//! Reads the worker's `status.json` of every unfinished job and mirrors it into `renders`.

use std::time::Duration;

use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use super::status::{Status, output, regular, status};
use super::{bundle, sweep};
use crate::state::AppState;

/// How often the status files are read.
pub const EVERY: Duration = Duration::from_secs(1);

async fn progress(state: &AppState, id: Uuid, status: &Status) -> Result<bool, sqlx::Error> {
    let moved = sqlx::query(
        "UPDATE renders SET state = 'running', frame = $2, frames = CASE WHEN $3 > 0 THEN $3 ELSE frames END, \
         started_at = coalesce(started_at, now()) WHERE id = $1 AND state IN ('queued', 'running') \
         AND (state, frame) IS DISTINCT FROM ('running', $2)",
    )
    .bind(id)
    .bind(status.frame)
    .bind(status.frames)
    .execute(&state.pool)
    .await?;
    Ok(moved.rows_affected() > 0)
}

/// Closes the render `id` as `outcome` (`failed` or `cancelled`) and removes its job directory.
pub async fn close(state: &AppState, id: Uuid, outcome: &str, error: Option<&str>) -> Result<bool, sqlx::Error> {
    let closed = sqlx::query(
        "UPDATE renders SET state = $2, error = $3, finished_at = now() WHERE id = $1 AND state IN ('queued', 'running')",
    )
    .bind(id)
    .bind(outcome)
    .bind(error)
    .execute(&state.pool)
    .await?;
    tokio::fs::remove_dir_all(bundle::dir(&state.data_dir, id)).await.ok();
    Ok(closed.rows_affected() > 0)
}

async fn done(state: &AppState, id: Uuid, status: &Status) -> Result<bool, sqlx::Error> {
    let (written, kept) = (bundle::dir(&state.data_dir, id).join("out.mp4"), output(&state.data_dir, id));
    let Some(size) = regular(&written).await.filter(|size| *size > 0) else {
        return close(state, id, "failed", Some("the worker reported a finished render and wrote no file")).await;
    };
    let moved = match tokio::fs::create_dir_all(state.data_dir.join("outputs")).await {
        Ok(()) => tokio::fs::rename(&written, &kept).await,
        Err(error) => Err(error),
    };
    if let Err(error) = moved {
        tracing::error!(render = %id, %error, "renders: could not keep a finished file");
        return close(state, id, "failed", Some("the finished file could not be stored")).await;
    }
    let stored = sqlx::query(
        "UPDATE renders SET state = 'done', frame = greatest($2, frame), frames = greatest($2, frames), error = NULL, \
         output = $3, size = $4, started_at = coalesce(started_at, now()), finished_at = now() \
         WHERE id = $1 AND state IN ('queued', 'running')",
    );
    let stored = stored.bind(id).bind(status.frames).bind(format!("outputs/{id}.mp4")).bind(size as i64);
    let stored = stored.execute(&state.pool).await?;
    tokio::fs::remove_dir_all(bundle::dir(&state.data_dir, id)).await.ok();
    Ok(stored.rows_affected() > 0)
}

/// Mirrors the status file of the unfinished render `id`, and answers whether its row moved. On
/// `done` the file moves to durable storage and the job directory goes.
pub async fn mirror(state: &AppState, id: Uuid) -> Result<bool, sqlx::Error> {
    let Some(status) = status(&bundle::dir(&state.data_dir, id)).await else { return Ok(false) };
    let moved = match status.state.as_str() {
        "running" => progress(state, id, &status).await?,
        "done" => done(state, id, &status).await?,
        outcome => close(state, id, outcome, status.error.as_deref()).await?,
    };
    if moved {
        state.events.render(id);
    }
    Ok(moved)
}

/// Mirrors every unfinished render once, and answers how many rows moved.
pub async fn tick(state: &AppState) -> Result<usize, sqlx::Error> {
    let open: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM renders WHERE state IN ('queued', 'running')")
        .fetch_all(&state.pool)
        .await?;
    let mut moved = 0;
    for id in open {
        moved += usize::from(mirror(state, id).await?);
    }
    Ok(moved)
}

/// Sweeps the stale jobs at boot, then mirrors every second, and sweeps again every
/// [`sweep::EVERY`], until `token` is cancelled. Run it in one replica.
pub async fn run(state: AppState, token: CancellationToken) {
    let mut every = tokio::time::interval(EVERY);
    let mut ticks = 0u64;
    loop {
        tokio::select! {
            () = token.cancelled() => return,
            _ = every.tick() => {}
        }
        if ticks.is_multiple_of(sweep::EVERY.as_secs()) {
            sweep::sweep(&state).await;
        }
        ticks += 1;
        if let Err(error) = tick(&state).await {
            tracing::error!(%error, "renders: could not mirror the job statuses");
        }
    }
}
