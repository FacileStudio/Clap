use serde::Deserialize;
use serde_json::Value;
use tronc::{Error, Result};
use uuid::Uuid;

use super::access::{self, Access};
use super::error::ProjectError;
use super::service::fetch;
use super::types::{Author, Project, SourceRequest};
use super::write;
use crate::state::AppState;

/// The timeline operations, as their `"op"` tag spells them.
pub const OPS: [&str; 9] = [
    "move_clip",
    "trim_clip",
    "split_clip",
    "delete_clip",
    "add_clip",
    "set_scene_duration",
    "reorder_scene",
    "delete_scene",
    "add_scene",
];

/// `POST /projects/{id}/edits`: timeline operations, applied in order to the source read at `rev`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EditsRequest {
    pub rev: i64,
    pub ops: Vec<Value>,
}

fn tagged(op: &Value) -> Result<&str> {
    let tag = op.get("op").and_then(Value::as_str).unwrap_or_default();
    match OPS.contains(&tag) {
        true => Ok(tag),
        false => Err(Error::invalid(format!("an op is an object whose `op` is one of {}", OPS.join(", ")))),
    }
}

/// The one place the timeline engine is called: `source` with `ops` applied in order. The
/// engine (crate `clap-timeline`) is not part of this build yet, so well-formed operations are
/// answered `503 unavailable` and nothing changes.
pub fn apply(source: &str, ops: &[Value]) -> Result<String> {
    if ops.is_empty() {
        return Err(Error::invalid("ops is empty: give at least one operation"));
    }
    let tags = ops.iter().map(tagged).collect::<Result<Vec<&str>>>()?;
    Err(Error::unavailable(format!(
        "timeline edits are not available yet: {} operation(s) on a {}-byte source were not applied. \
         Edit the source text instead",
        tags.len(),
        source.len()
    )))
}

/// Applies timeline operations to the project's source and saves the result as [`write::save`]
/// does: the same `rev` rule, the same revision, the same event.
pub async fn edits(
    state: &AppState,
    author: &Author,
    id: Uuid,
    request: EditsRequest,
) -> std::result::Result<Project, ProjectError> {
    access::check(state, author.user_id, id, Access::Editor).await?;
    let row = fetch(state, id).await?;
    if row.rev != request.rev {
        return Err(ProjectError::Stale { rev: row.rev, sent: request.rev });
    }
    let source = apply(&row.source, &request.ops)?;
    write::save(state, author, id, SourceRequest { source, rev: request.rev }).await
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn ops_are_checked_then_answered_unavailable() {
        assert_eq!(apply("x", &[]).unwrap_err().code, tronc::Code::InvalidArgument);
        assert_eq!(apply("x", &[json!({"op": "explode"})]).unwrap_err().code, tronc::Code::InvalidArgument);
        assert_eq!(apply("x", &[json!("move_clip")]).unwrap_err().code, tronc::Code::InvalidArgument);
        let known = json!({"op": "move_clip", "clip": "c1", "start": 2});
        let error = apply("x", &[known]).unwrap_err();
        assert_eq!(error.code, tronc::Code::Unavailable);
        assert!(error.message.contains("not available yet"), "{}", error.message);
    }
}
