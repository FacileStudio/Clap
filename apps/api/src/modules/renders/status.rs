//! What the worker's side of a job directory holds, read without trusting it.

use std::path::{Path, PathBuf};

use serde::Deserialize;
use tronc::{Error, Result};
use uuid::Uuid;

use crate::state::AppState;

/// The largest `status.json` the API reads.
pub const STATUS_MAX_BYTES: u64 = 64 << 10;

/// The longest error kept from a status file, in characters.
pub const ERROR_MAX_CHARS: usize = 2000;

/// `status.json`, as the worker writes it.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Status {
    /// `running`, `done`, `failed` or `cancelled`.
    pub state: String,
    #[serde(default)]
    pub frame: i64,
    #[serde(default)]
    pub frames: i64,
    #[serde(default)]
    pub error: Option<String>,
}

/// `DATA_DIR/outputs/<id>.mp4`: where a finished render is kept.
pub fn output(data_dir: &Path, id: Uuid) -> PathBuf {
    data_dir.join("outputs").join(format!("{id}.mp4"))
}

/// The size of `path` when it is a regular file, never a link the worker's side could have
/// pointed somewhere else.
pub async fn regular(path: &Path) -> Option<u64> {
    let metadata = tokio::fs::symlink_metadata(path).await.ok()?;
    metadata.file_type().is_file().then_some(metadata.len())
}

/// The status of the job in `dir`, or `None` while the worker has written none, or none the
/// protocol knows.
pub async fn status(dir: &Path) -> Option<Status> {
    let path = dir.join("status.json");
    if regular(&path).await? > STATUS_MAX_BYTES {
        return None;
    }
    let status: Status = serde_json::from_slice(&tokio::fs::read(&path).await.ok()?).ok()?;
    let known = matches!(status.state.as_str(), "running" | "done" | "failed" | "cancelled");
    let error = status.error.map(|error| error.chars().take(ERROR_MAX_CHARS).collect());
    known.then(|| Status { frame: status.frame.max(0), frames: status.frames.max(0), error, ..status })
}

/// The oldest heartbeat a live worker has, in seconds.
pub const HEARTBEAT_FRESH_SECONDS: i64 = 30;

/// Whether the render worker wrote its heartbeat, `renders/.worker`, in the last half minute.
/// The file holds an RFC 3339 timestamp the worker rewrites every second.
pub async fn worker_alive(data_dir: &Path) -> bool {
    let path = data_dir.join("renders").join(".worker");
    if regular(&path).await.is_none_or(|size| size > 64) {
        return false;
    }
    let Ok(text) = tokio::fs::read_to_string(&path).await else { return false };
    let stamp = time::format_description::well_known::Rfc3339;
    let Ok(written) = time::OffsetDateTime::parse(text.trim(), &stamp) else { return false };
    (time::OffsetDateTime::now_utc() - written).whole_seconds().abs() <= HEARTBEAT_FRESH_SECONDS
}

/// Refuses with `503` while the worker's heartbeat is stale.
pub async fn serving(state: &AppState) -> Result<()> {
    match worker_alive(&state.data_dir).await {
        true => Ok(()),
        false => Err(Error::unavailable("the render service is not running: nothing was queued, try again later")),
    }
}
