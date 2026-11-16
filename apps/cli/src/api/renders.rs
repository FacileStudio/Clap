//! Server renders: queue one, follow it, stop it, fetch the MP4.

use std::io::Write;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::transfer::OnBytes;
use super::{ApiError, Client, Payload, text};
use crate::url::encode;

/// A render as the server answers it.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Render {
    /// The render id.
    #[serde(deserialize_with = "text")]
    pub id: String,
    /// `queued`, `running`, `done`, `failed` or `cancelled`.
    pub state: String,
    /// Frames captured so far.
    #[serde(default)]
    pub frame: u64,
    /// Frames the render will capture, 0 until it starts.
    #[serde(default)]
    pub frames: u64,
    /// Why the render failed.
    #[serde(default)]
    pub error: Option<String>,
}

impl Render {
    /// Whether the render will not change any more.
    pub fn is_final(&self) -> bool {
        matches!(self.state.as_str(), "done" | "failed" | "cancelled")
    }
}

impl Client {
    /// `POST /api/projects/{id}/renders`: queues a render of the project as it is saved. `spec`
    /// holds any of `width`, `height`, `fps` and `range`.
    pub fn start_render(&self, project: &str, spec: &Value) -> Result<Render, ApiError> {
        self.json("POST", &format!("/projects/{}/renders", encode(project)), Payload::Json(spec))
    }

    /// `GET /api/renders/{id}`.
    pub fn render(&self, id: &str) -> Result<Render, ApiError> {
        self.json("GET", &format!("/renders/{}", encode(id)), Payload::Empty)
    }

    /// `DELETE /api/renders/{id}`: asks a waiting or running render to stop.
    pub fn cancel_render(&self, id: &str) -> Result<(), ApiError> {
        self.call("DELETE", &format!("/renders/{}", encode(id)), Payload::Empty).map(drop)
    }

    /// `GET /api/renders/{id}/file`: the MP4 of a finished render, written into `sink`.
    pub fn download_render(&self, id: &str, sink: &mut dyn Write, progress: OnBytes<'_>) -> Result<u64, ApiError> {
        self.fetch(&format!("/renders/{}/file", encode(id)), sink, progress)
    }
}
