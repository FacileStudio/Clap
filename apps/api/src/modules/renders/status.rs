//! What the worker's side of a job directory holds, read without trusting it.

use std::path::{Path, PathBuf};

use serde::Deserialize;
use tronc::{Error, Result};
use uuid::Uuid;

use super::jobfs;
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

/// The largest `out.mp4` the API keeps: 8 GiB.
pub const OUTPUT_MAX_BYTES: u64 = 8 << 30;

/// The status of the job `id`, or `None` while the worker has written none, or none the
/// protocol knows. The file is read through [`jobfs::regular`]: never a link, never past 64 KiB.
pub async fn status(data_dir: &Path, id: Uuid) -> Option<Status> {
    let (data_dir, relative) = (data_dir.to_owned(), format!("{id}/status.json"));
    let bytes =
        tokio::task::spawn_blocking(move || jobfs::read(&data_dir, &relative, STATUS_MAX_BYTES)).await.ok()??;
    let status: Status = serde_json::from_slice(&bytes).ok()?;
    let known = matches!(status.state.as_str(), "running" | "done" | "failed" | "cancelled");
    let error = status.error.map(|error| error.chars().take(ERROR_MAX_CHARS).collect());
    known.then(|| Status { frame: status.frame.max(0), frames: status.frames.max(0), error, ..status })
}

/// The oldest heartbeat a live worker has, in seconds.
pub const HEARTBEAT_FRESH_SECONDS: i64 = 30;

/// Whether the render worker wrote its heartbeat, `renders/.worker`, in the last half minute.
/// The file holds an RFC 3339 timestamp the worker rewrites every second.
pub async fn worker_alive(data_dir: &Path) -> bool {
    let data_dir = data_dir.to_owned();
    let read = tokio::task::spawn_blocking(move || jobfs::read(&data_dir, ".worker", 64)).await;
    let Some(text) = read.ok().flatten().and_then(|bytes| String::from_utf8(bytes).ok()) else { return false };
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
