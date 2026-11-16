//! The timeline of a project: the model `clap-timeline` reads from its source, and the edit ops
//! applied to it under the `rev` rule.

use clap_timeline::{Op, Timeline};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tronc::{Error, Result};
use uuid::Uuid;

use super::access::{self, Access};
use super::error::ProjectError;
use super::service::fetch;
use super::types::{Author, Project, SourceRequest};
use super::write;
use crate::state::AppState;

/// The most operations one edit applies.
pub const MAX_OPS: usize = 500;

/// `POST /projects/{id}/edits`: timeline operations, applied in order to the source read at `rev`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EditsRequest {
    pub rev: i64,
    pub ops: Vec<Value>,
}

/// What `GET /projects/{id}/timeline` answers.
#[derive(Debug, Serialize)]
pub struct TimelineAnswer {
    pub rev: i64,
    pub timeline: Timeline,
}

/// A project after timeline operations.
#[derive(Debug)]
pub struct Edited {
    /// The project as saved.
    pub project: Project,
    /// Its timeline now.
    pub timeline: Timeline,
    /// The ids, as they are now, of the clips the operations named or added.
    pub clips: Vec<String>,
}

/// `ops` as the engine reads them: `400` naming the first one that is not an operation.
pub fn ops(raw: &[Value]) -> Result<Vec<Op>> {
    if raw.is_empty() {
        return Err(Error::invalid("ops is empty: give at least one operation"));
    }
    if raw.len() > MAX_OPS {
        return Err(Error::invalid(format!("ops holds {} operations: send at most {MAX_OPS} at once", raw.len())));
    }
    let parsed = |(index, op): (usize, &Value)| {
        Op::deserialize(op)
            .map_err(|error| Error::invalid(format!("ops[{index}] is not a timeline operation: {error}")))
    };
    raw.iter().enumerate().map(parsed).collect()
}

fn named(op: &Op) -> Vec<String> {
    match op {
        Op::MoveClip { clip, .. }
        | Op::TrimClip { clip, .. }
        | Op::DeleteClip { clip }
        | Op::UpdateClip { clip, .. } => {
            vec![clip.clone()]
        }
        Op::SplitClip { clip, .. } => vec![clip.clone(), format!("{clip}-b")],
        _ => Vec::new(),
    }
}

fn ids(timeline: &Timeline) -> Vec<&str> {
    timeline.scenes.iter().flat_map(|scene| scene.clips.iter().map(|clip| clip.id.as_str())).collect()
}

/// The clips of `after` that `ops` named, and those that were not in `before`.
pub fn affected(before: &Timeline, after: &Timeline, ops: &[Op]) -> Vec<String> {
    let (old, named): (Vec<&str>, Vec<String>) = (ids(before), ops.iter().flat_map(named).collect());
    let touched = |id: &&str| named.iter().any(|name| name == id) || !old.contains(id);
    ids(after).into_iter().filter(touched).map(str::to_owned).collect()
}

/// `GET /projects/{id}/timeline`: the scenes and clips of the source as it is saved, with the
/// `rev` an edit must name.
pub async fn read(state: &AppState, user_id: i64, id: Uuid) -> std::result::Result<TimelineAnswer, ProjectError> {
    access::check(state, user_id, id, Access::Viewer).await?;
    let row = fetch(state, id).await?;
    let timeline = clap_timeline::read(&row.source).map_err(ProjectError::Timeline)?;
    Ok(TimelineAnswer { rev: row.rev, timeline })
}

/// Applies timeline operations to the project's source and saves the result as [`write::save`]
/// does: the same `rev` rule, the same revision, the same event. Nothing is saved unless every
/// operation applies. The engine runs on a blocking thread, at most [`MAX_OPS`] operations.
pub async fn edits(
    state: &AppState,
    author: &Author,
    id: Uuid,
    request: EditsRequest,
) -> std::result::Result<Edited, ProjectError> {
    access::check(state, author.user_id, id, Access::Editor).await?;
    let row = fetch(state, id).await?;
    if row.rev != request.rev {
        return Err(ProjectError::Stale { rev: row.rev, sent: request.rev });
    }
    let ops = ops(&request.ops)?;
    let edit = move || -> std::result::Result<_, clap_timeline::Error> {
        let before = clap_timeline::read(&row.source)?;
        let source = clap_timeline::apply(&row.source, &ops)?;
        let timeline = clap_timeline::read(&source)?;
        Ok((affected(&before, &timeline, &ops), source, timeline))
    };
    let applied =
        tokio::task::spawn_blocking(edit).await.map_err(|error| Error::internal("an edit did not finish", error))?;
    let (clips, source, timeline) = applied.map_err(ProjectError::Timeline)?;
    let project = write::save(state, author, id, SourceRequest { source, rev: request.rev }).await?;
    Ok(Edited { clips, project, timeline })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    const SOURCE: &str = "---\nlayout: raw\nduration: 4\n---\n\
<p id=\"a\" data-start=\"0\" data-duration=\"2\">Hi</p>\n<p data-start=\"2\" data-duration=\"1\">Bye</p>\n";

    #[test]
    fn ops_are_read_strictly_and_the_first_bad_one_is_named() {
        assert!(ops(&[]).unwrap_err().message.contains("empty"));
        let bad = ops(&[json!({"op": "delete_clip", "clip": "a"}), json!({"op": "explode"})]).unwrap_err();
        assert!(bad.message.starts_with("ops[1] is not a timeline operation"), "{}", bad.message);
        assert_eq!(ops(&[json!({"op": "move_clip", "clip": "a", "start": 1})]).unwrap().len(), 1);
    }

    #[test]
    fn the_affected_clips_are_those_named_and_those_added() {
        let ops =
            ops(&[json!({"op": "split_clip", "clip": "a", "at": 1}), json!({"op": "delete_clip", "clip": "clip-0-2"})]);
        let ops = ops.unwrap();
        let after = clap_timeline::apply(SOURCE, &ops).unwrap();
        let (before, after) = (clap_timeline::read(SOURCE).unwrap(), clap_timeline::read(&after).unwrap());
        assert_eq!(affected(&before, &after, &ops), ["a", "a-b"]);
    }
}
