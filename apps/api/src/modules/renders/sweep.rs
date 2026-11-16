//! Clears what a crash, a restart or a dead worker leaves behind.

use std::collections::HashSet;
use std::path::Path;
use std::time::Duration;

use uuid::Uuid;

use super::{bundle, mirror, status};
use crate::state::AppState;

/// How often the sweep runs after the one at boot.
pub const EVERY: Duration = Duration::from_secs(10 * 60);

/// How long a queued job may wait while the worker is down before it is given up: six hours.
/// While the worker is alive a queued job waits its turn, however long the queue.
pub const STALE: Duration = Duration::from_secs(6 * 60 * 60);

/// How long a running job may go without a status write before it is given up. The worker
/// writes one twice a second.
pub const SILENT: Duration = Duration::from_secs(10 * 60);

/// How long a new job is left alone: its directory may still be on its way to disk.
pub const GRACE: Duration = Duration::from_secs(60);

type SweepError = Box<dyn std::error::Error + Send + Sync>;

async fn open(state: &AppState) -> Result<Vec<(Uuid, String, f64)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, state, extract(epoch FROM now() - created_at)::float8 FROM renders WHERE state IN ('queued', 'running')",
    )
    .fetch_all(&state.pool)
    .await
}

async fn verdict(state: &AppState, id: Uuid, row_state: &str, age: f64) -> Option<&'static str> {
    let lost = !tokio::fs::try_exists(bundle::dir(&state.data_dir, id).join("job.json")).await.unwrap_or(false);
    if lost && age > GRACE.as_secs_f64() {
        return Some("the render job was lost before the worker finished it");
    }
    if row_state == "running" {
        let silence = status::status(&state.data_dir, id).await.and_then(|status| status.silence());
        let silent = silence.is_none_or(|seconds| seconds > SILENT.as_secs() as i64);
        return silent.then_some("the worker stopped reporting progress and the render was given up");
    }
    let down = age > STALE.as_secs_f64() && !status::worker_alive(&state.data_dir).await;
    down.then_some("the render service was down too long and the render was given up")
}

async fn given_up(state: &AppState) -> Result<HashSet<Uuid>, SweepError> {
    let mut waiting = HashSet::new();
    for (id, row_state, age) in open(state).await? {
        let Some(reason) = verdict(state, id, &row_state, age).await else {
            waiting.insert(id);
            continue;
        };
        tokio::fs::remove_dir_all(bundle::dir(&state.data_dir, id)).await.ok();
        if mirror::close(state, id, "failed", Some(reason)).await? {
            state.events.render(id);
        }
    }
    Ok(waiting)
}

fn named(file_name: &std::ffi::OsStr) -> Option<Uuid> {
    Uuid::parse_str(Path::new(file_name).file_stem()?.to_str()?).ok()
}

async fn entries(dir: &Path) -> Vec<(Uuid, std::path::PathBuf)> {
    let mut found = Vec::new();
    let Ok(mut read) = tokio::fs::read_dir(dir).await else { return found };
    while let Ok(Some(entry)) = read.next_entry().await {
        if let Some(id) = named(&entry.file_name()) {
            found.push((id, entry.path()));
        }
    }
    found
}

async fn kept(state: &AppState) -> Result<HashSet<Uuid>, sqlx::Error> {
    let done = sqlx::query_scalar("SELECT id FROM renders WHERE state = 'done'").fetch_all(&state.pool).await?;
    Ok(done.into_iter().collect())
}

/// Fails every unfinished render whose job directory is gone or that waited past [`STALE`],
/// then removes the job directories and the output files no row names, and the directories of
/// renders that failed or were cancelled more than [`RETENTION`](super::retention::RETENTION) ago. A render given up here
/// loses its directory at once, so a worker that comes back cannot pick it up. The directories are
/// listed before the rows are read, and a row is always written before its directory, so a job
/// queued meanwhile is never taken. Answers how many directories and files went.
pub async fn swept(state: &AppState) -> Result<usize, SweepError> {
    let jobs = entries(&bundle::root(&state.data_dir)).await;
    let files = entries(&state.data_dir.join("outputs")).await;
    super::retention::retire(state).await?;
    let (mut waiting, kept) = (given_up(state).await?, kept(state).await?);
    waiting.extend(super::retention::retained(state).await?);
    let mut removed = 0;
    for (_, path) in jobs.into_iter().filter(|(id, _)| !waiting.contains(id)) {
        removed += usize::from(tokio::fs::remove_dir_all(&path).await.is_ok());
    }
    for (_, path) in files.into_iter().filter(|(id, _)| !kept.contains(id)) {
        removed += usize::from(tokio::fs::remove_file(&path).await.is_ok());
    }
    Ok(removed)
}

/// [`swept`], with a failure logged instead of answered.
pub async fn sweep(state: &AppState) {
    match swept(state).await {
        Ok(0) => {}
        Ok(removed) => tracing::info!(removed, "renders: swept stale jobs"),
        Err(error) => tracing::error!(%error, "renders: sweep failed"),
    }
}
