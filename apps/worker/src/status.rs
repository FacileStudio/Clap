//! `status.json`: what the API reads to follow a job.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{clock, files};

/// Where a job is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum State {
    Running,
    Done,
    Failed,
    Cancelled,
}

/// The content of `status.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Status {
    pub state: State,
    /// Frames captured so far.
    pub frame: u32,
    /// Frames the render will capture; 0 until the scene has loaded.
    pub frames: u32,
    /// Why the job failed.
    pub error: Option<String>,
    /// When this file was written, RFC 3339.
    pub updated_at: String,
}

impl Status {
    /// A status written now.
    pub(crate) fn new(state: State, frame: u32, frames: u32) -> Status {
        Status { state, frame, frames, error: None, updated_at: clock::now() }
    }

    /// A failure written now.
    pub(crate) fn failed(reason: impl Into<String>) -> Status {
        Status { error: Some(reason.into()), ..Status::new(State::Failed, 0, 0) }
    }

    /// Writes the status of the job in `dir`, atomically.
    pub(crate) fn save(&self, dir: &Path) -> std::io::Result<()> {
        files::write(&dir.join("status.json"), &serde_json::to_string(self).unwrap_or_default())
    }

    /// The status of the job in `dir`: `None` without a file, an error for a file that is not one.
    pub(crate) fn load(dir: &Path) -> Option<Result<Status, String>> {
        let path = dir.join("status.json");
        files::exists(&path)
            .then(|| files::read(&path).and_then(|text| serde_json::from_str(&text).map_err(|error| error.to_string())))
    }
}
