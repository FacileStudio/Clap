//! Renaming an asset: the row takes the new name and the source follows through `rename_media`,
//! in one transaction and one revision. The bytes and the derived files are stored by digest, so
//! nothing moves on disk.

use clap_timeline::Op;
use serde::{Deserialize, Serialize};
use sqlx::PgConnection;
use tronc::Error;
use uuid::Uuid;

use super::service::{Asset, find};
use super::worker::{self, Job, unfinished};
use crate::extract::db;
use crate::modules::projects::access::{self, Access};
use crate::modules::projects::error::ProjectError;
use crate::modules::projects::service::fetch;
use crate::modules::projects::source;
use crate::modules::projects::types::{Author, SourceRequest};
use crate::modules::projects::write::{advance, refusal};
use crate::state::AppState;

/// `PATCH /projects/{id}/assets/{name}`: the new name, and the `rev` the caller read.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RenameRequest {
    pub name: String,
    pub rev: i64,
}

/// What a rename answers: the asset under its new name, and the project's new `rev`.
#[derive(Debug, Serialize)]
pub struct Renamed {
    pub asset: Asset,
    pub rev: i64,
}

fn refused(error: clap_timeline::Error) -> ProjectError {
    match error {
        clap_timeline::Error::NameTaken(_) => Error::conflict(error.to_string()).into(),
        error => ProjectError::Timeline(error),
    }
}

async fn relabel(conn: &mut PgConnection, project: Uuid, from: &str, to: &str) -> Result<Option<Job>, ProjectError> {
    let moved = sqlx::query_as::<_, (Uuid, String, String, bool)>(concat!(
        "UPDATE assets SET name = $3 WHERE project_id = $1 AND name = $2 RETURNING project_id, name, sha256, ",
        unfinished!()
    ))
    .bind(project)
    .bind(from)
    .bind(to)
    .fetch_optional(conn)
    .await;
    match moved {
        Ok(Some((project, name, sha256, open))) => Ok(open.then_some(Job { project, name, sha256 })),
        Ok(None) => Err(Error::not_found("asset not found").into()),
        Err(error) if error.as_database_error().is_some_and(|found| found.is_unique_violation()) => {
            Err(Error::conflict(format!("the project already has an asset named {to}")).into())
        }
        Err(error) => Err(db(error).into()),
    }
}

/// The source with `from` renamed to `to`, once the caller may edit, the project is live and at
/// `rev`, and the asset exists.
async fn renamed_source(
    state: &AppState,
    author: &Author,
    project: Uuid,
    (from, to): (&str, &str),
    rev: i64,
) -> Result<String, ProjectError> {
    access::check(state, author.user_id, project, Access::Editor).await?;
    let row = fetch(state, project).await?;
    if row.trashed_at.is_some() {
        return Err(Error::failed("the project is in the trash").into());
    }
    if row.rev != rev {
        return Err(ProjectError::Stale { rev: row.rev, sent: rev });
    }
    find(state, project, from).await?;
    let op = Op::RenameMedia { from: from.to_owned(), to: to.to_owned() };
    let source = clap_timeline::apply(&row.source, &[op]).map_err(refused)?;
    source::checked(&source, state.limits.source)?;
    Ok(source)
}

/// Tells open editors what moved, and starts the media job again under the new name when it
/// had not finished.
async fn announce(
    state: &AppState,
    project: Uuid,
    (from, to): (&str, &str),
    job: Option<Job>,
) -> Result<Asset, ProjectError> {
    state.events.asset(project, from, "deleted");
    let asset = find(state, project, to).await?;
    let now = match asset.state.as_str() {
        "probing" => "probing",
        "failed" => "failed",
        _ => "ready",
    };
    state.events.asset(project, to, now);
    if let Some(job) = job {
        worker::spawn(state.clone(), job);
    }
    Ok(asset)
}

/// `PATCH /projects/{id}/assets/{name}`, for an editor: renames the asset `from` to `request.name`
/// and applies `rename_media` to the source, as one revision at `request.rev + 1`. `409` on a
/// stale `rev` or a name already taken, by an asset or by a reference in the source; `404` for an
/// unknown asset; `422` with `timeline_error` for a name that is not an asset name. A media job
/// still running on the old name is started again on the new one.
pub async fn rename(
    state: &AppState,
    author: &Author,
    project: Uuid,
    from: &str,
    request: RenameRequest,
) -> Result<Renamed, ProjectError> {
    let to = request.name.trim();
    let source = renamed_source(state, author, project, (from, to), request.rev).await?;
    let mut tx = state.pool.begin().await.map_err(db)?;
    let job = relabel(&mut tx, project, from, to).await?;
    let written = SourceRequest { source, rev: request.rev };
    let Some(saved) = advance(&mut tx, author, project, &written).await.map_err(db)? else {
        return Err(refusal(state, project, request.rev).await);
    };
    tx.commit().await.map_err(db)?;
    state.events.changed(project, saved.rev);
    let asset = announce(state, project, (from, to), job).await?;
    Ok(Renamed { asset, rev: saved.rev })
}
