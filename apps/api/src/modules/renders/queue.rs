//! Queues a render row under the instance's and the person's caps.

use tronc::{Error, Result};
use uuid::Uuid;

use super::spec::Spec;
use super::types::{RenderRow, columns};
use crate::extract::db;
use crate::state::AppState;

/// Queues one row, unless the instance already has `RENDER_QUEUE_MAX` renders waiting or
/// running, or the person `RENDER_USER_MAX`: `429`, saying which. `project` is the project and its revision, or `(None, 0)` for a posted bundle.
pub async fn insert(state: &AppState, user_id: i64, project: (Option<Uuid>, i64), spec: &Spec) -> Result<RenderRow> {
    let inserted = sqlx::query_as::<_, RenderRow>(concat!(
        "INSERT INTO renders (project_id, requested_by, rev, width, height, fps, range_start, range_end, frames) \
         SELECT $1, $2, $3, $4, $5, $6, $7, $8, $9 \
         WHERE (SELECT count(*) FROM renders WHERE state IN ('queued', 'running')) < $10 \
         AND (SELECT count(*) FROM renders WHERE state IN ('queued', 'running') AND requested_by = $2) < $11 RETURNING ",
        columns!()
    ))
    .bind(project.0)
    .bind(user_id)
    .bind(project.1)
    .bind(spec.width as i32)
    .bind(spec.height as i32)
    .bind(spec.fps as i32)
    .bind(spec.range.map(|range| range.0))
    .bind(spec.range.map(|range| range.1))
    .bind(spec.frames)
    .bind(i64::from(state.limits.render.queue))
    .bind(i64::from(state.limits.render.user))
    .fetch_optional(&state.pool)
    .await
    .map_err(db)?;
    match inserted {
        Some(row) => Ok(row),
        None => Err(refusal(state, user_id).await),
    }
}

async fn refusal(state: &AppState, user_id: i64) -> Error {
    let mine: std::result::Result<i64, sqlx::Error> =
        sqlx::query_scalar("SELECT count(*) FROM renders WHERE state IN ('queued', 'running') AND requested_by = $1")
            .bind(user_id)
            .fetch_one(&state.pool)
            .await;
    match mine {
        Ok(count) if count >= i64::from(state.limits.render.user) => Error::rate_limited(format!(
            "you already have {count} renders waiting or running: try again when one has finished"
        )),
        Ok(_) => Error::rate_limited("too many renders are waiting: try again when one has finished"),
        Err(error) => db(error),
    }
}
