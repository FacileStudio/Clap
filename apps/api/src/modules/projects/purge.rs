use std::time::Duration;

use tokio_util::sync::CancellationToken;

use crate::state::AppState;

/// How often the trash is purged.
pub const EVERY: Duration = Duration::from_secs(60 * 60);

/// Deletes for good every project trashed more than 30 days ago. The only path to permanent
/// deletion besides a manager's explicit delete.
pub async fn purge(state: &AppState) -> Result<u64, sqlx::Error> {
    let deleted =
        sqlx::query("DELETE FROM projects WHERE trashed_at < now() - interval '30 days'").execute(&state.pool).await?;
    Ok(deleted.rows_affected())
}

/// How many revisions of each project are kept: the newest.
pub const KEEP_REVISIONS: i64 = 50;

/// Deletes the revisions of each project past its newest [`KEEP_REVISIONS`].
pub async fn revisions(state: &AppState) -> Result<u64, sqlx::Error> {
    let deleted = sqlx::query(
        "DELETE FROM project_revisions WHERE id IN (SELECT id FROM (SELECT id, \
         row_number() OVER (PARTITION BY project_id ORDER BY rev DESC) AS n FROM project_revisions) ranked WHERE n > $1)",
    )
    .bind(KEEP_REVISIONS)
    .execute(&state.pool)
    .await?;
    Ok(deleted.rows_affected())
}

async fn hour(state: &AppState) {
    match purge(state).await {
        Ok(0) => {}
        Ok(count) => tracing::info!(count, "projects: purged the trash"),
        Err(error) => tracing::error!(%error, "projects: trash purge failed"),
    }
    match revisions(state).await {
        Ok(0) => {}
        Ok(count) => tracing::info!(count, "projects: pruned old revisions"),
        Err(error) => tracing::error!(%error, "projects: revision prune failed"),
    }
}

/// Purges every hour until `token` is cancelled. Run it in one replica.
pub async fn run(state: AppState, token: CancellationToken) {
    let mut tick = tokio::time::interval(EVERY);
    loop {
        tokio::select! {
            () = token.cancelled() => return,
            _ = tick.tick() => hour(&state).await,
        }
    }
}
