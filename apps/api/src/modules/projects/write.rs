use sqlx::PgConnection;
use tronc::Error;
use uuid::Uuid;

use super::access::{self, Access};
use super::error::ProjectError;
use super::service::{answer, fetch};
use super::source;
use super::types::{Author, Project, ProjectRow, SourceRequest, columns};
use crate::extract::db;
use crate::state::AppState;

/// Keeps the source `row` holds as the revision `row.rev`, with its author and origin.
pub async fn revision(conn: &mut PgConnection, row: &ProjectRow, author: &Author) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO project_revisions (project_id, rev, source, created_by, origin) \
         VALUES ($1, $2, $3, (SELECT u.id FROM porte_users u WHERE u.id = $4), $5)",
    )
    .bind(row.id)
    .bind(row.rev)
    .bind(&row.source)
    .bind(author.user_id)
    .bind(author.origin.name())
    .execute(conn)
    .await
    .map(|_| ())
}

async fn store(
    state: &AppState,
    author: &Author,
    id: Uuid,
    request: &SourceRequest,
) -> Result<ProjectRow, ProjectError> {
    let mut tx = state.pool.begin().await.map_err(db)?;
    let row = sqlx::query_as::<_, ProjectRow>(concat!(
        "UPDATE projects SET source = $2, rev = rev + 1, updated_at = now(), updated_by = $4 \
         WHERE id = $1 AND rev = $3 AND trashed_at IS NULL RETURNING ",
        columns!()
    ))
    .bind(id)
    .bind(&request.source)
    .bind(request.rev)
    .bind(author.user_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db)?;
    let Some(row) = row else { return Err(refusal(state, id, request.rev).await) };
    revision(&mut tx, &row, author).await.map_err(db)?;
    tx.commit().await.map_err(db)?;
    Ok(row)
}

async fn refusal(state: &AppState, id: Uuid, sent: i64) -> ProjectError {
    match fetch(state, id).await {
        Ok(row) if row.trashed_at.is_some() => Error::failed("the project is in the trash").into(),
        Ok(row) => ProjectError::Stale { rev: row.rev, sent },
        Err(error) => error.into(),
    }
}

/// `PUT /projects/{id}/source`, and every other door that writes a source: checks the caller may
/// edit, that the source parses, then stores it only if the project is still at `request.rev`. The
/// new source is kept as a revision and open editors are told. A stale `rev` changes nothing.
pub async fn save(
    state: &AppState,
    author: &Author,
    id: Uuid,
    request: SourceRequest,
) -> Result<Project, ProjectError> {
    let have = access::check(state, author.user_id, id, Access::Editor).await?;
    source::checked(&request.source, state.limits.source)?;
    let row = store(state, author, id, &request).await?;
    state.events.changed(row.id, row.rev);
    Ok(answer(row, have))
}
