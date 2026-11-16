//! Reads the worker's `status.json` of every unfinished job and mirrors it into `renders`.

use std::time::Duration;

use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use super::jobfs;
use super::status::{OUTPUT_MAX_BYTES, Status, output, status};
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

/// Closes the render `id` as `outcome` (`failed` or `cancelled`). Its job directory stays for
/// [`sweep::RETENTION`], for whoever looks into the failure; the sweep removes it then.
pub async fn close(state: &AppState, id: Uuid, outcome: &str, error: Option<&str>) -> Result<bool, sqlx::Error> {
    let closed = sqlx::query(
        "UPDATE renders SET state = $2, error = $3, finished_at = now() WHERE id = $1 AND state IN ('queued', 'running')",
    )
    .bind(id)
    .bind(outcome)
    .bind(error)
    .execute(&state.pool)
    .await?;
    Ok(closed.rows_affected() > 0)
}

async fn kept(state: &AppState, id: Uuid) -> std::io::Result<u64> {
    let (data_dir, target) = (state.data_dir.to_path_buf(), output(&state.data_dir, id));
    tokio::fs::create_dir_all(target.parent().unwrap_or(&data_dir)).await?;
    let copy = move || jobfs::copy_out(&data_dir, &format!("{id}/out.mp4"), &target, OUTPUT_MAX_BYTES);
    tokio::task::spawn_blocking(copy).await.map_err(std::io::Error::other)?
}

async fn done(state: &AppState, id: Uuid, status: &Status) -> Result<bool, sqlx::Error> {
    let size = match kept(state, id).await {
        Ok(size) if size > 0 => size,
        Ok(_) => {
            return close(state, id, "failed", Some("the worker reported a finished render and wrote an empty file"))
                .await;
        }
        Err(error) => {
            tracing::warn!(render = %id, %error, "renders: refused the worker's file");
            return close(state, id, "failed", Some("the worker reported a finished render and wrote no usable file"))
                .await;
        }
    };
    let stored = sqlx::query_as(
        "UPDATE renders SET state = 'done', frame = greatest($2, frame), frames = greatest($2, frames), error = NULL, \
         output = $3, size = $4, started_at = coalesce(started_at, now()), finished_at = now() \
         WHERE id = $1 AND state IN ('queued', 'running') RETURNING frames, fps",
    );
    let stored = stored.bind(id).bind(status.frames).bind(format!("outputs/{id}.mp4")).bind(size as i64);
    let kept: Option<(i64, i32)> = stored.fetch_optional(&state.pool).await?;
    tokio::fs::remove_dir_all(bundle::dir(&state.data_dir, id)).await.ok();
    if let Some((frames, fps)) = kept {
        crate::modules::public::poster::make(state, id, frames * 1000 / i64::from(fps.max(1))).await;
    }
    Ok(kept.is_some())
}

/// Mirrors the status file of the unfinished render `id`, and answers whether its row moved. On
/// `done` the file moves to durable storage, the job directory goes (the worker never deletes
/// one) and the poster is made beside the file.
pub async fn mirror(state: &AppState, id: Uuid) -> Result<bool, sqlx::Error> {
    let Some(status) = status(&state.data_dir, id).await else { return Ok(false) };
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
