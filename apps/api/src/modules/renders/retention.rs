//! What the sweep keeps: finished renders, and the job directories of renders that did not finish.

use std::collections::HashSet;
use std::time::Duration;

use uuid::Uuid;

use crate::state::AppState;

/// How many finished renders of a project are kept: the newest, which a share link shows, and
/// the two before it.
pub const KEEP_DONE: i64 = 3;

/// How long the job directory of a failed or cancelled render is kept.
pub const RETENTION: Duration = Duration::from_secs(24 * 60 * 60);

/// Deletes the finished renders past [`KEEP_DONE`] per project, and those of posted bundles a
/// week after they finished. Their files go in the same sweep, since no row names them.
pub async fn retire(state: &AppState) -> Result<u64, sqlx::Error> {
    let gone = sqlx::query(
        "DELETE FROM renders WHERE id IN (SELECT id FROM (SELECT id, project_id, finished_at, \
         row_number() OVER (PARTITION BY project_id ORDER BY finished_at DESC, id) AS n FROM renders \
         WHERE state = 'done') ranked WHERE (project_id IS NOT NULL AND n > $1) \
         OR (project_id IS NULL AND finished_at < now() - interval '7 days'))",
    )
    .bind(KEEP_DONE)
    .execute(&state.pool)
    .await?;
    Ok(gone.rows_affected())
}

/// The renders whose job directory is still kept: failed or cancelled less than [`RETENTION`] ago.
pub async fn retained(state: &AppState) -> Result<HashSet<Uuid>, sqlx::Error> {
    let recent = sqlx::query_scalar(
        "SELECT id FROM renders WHERE state IN ('failed', 'cancelled') AND finished_at > now() - make_interval(secs => $1)",
    )
    .bind(RETENTION.as_secs_f64())
    .fetch_all(&state.pool)
    .await?;
    Ok(recent.into_iter().collect())
}
