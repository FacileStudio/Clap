use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

/// The columns every render answer reads, as a literal for `concat!`.
macro_rules! columns {
    () => {
        "id, project_id, rev, width, height, fps, range_start, range_end, state, frame, frames, error, size, \
         created_at, started_at, finished_at"
    };
}
pub(crate) use columns;

/// Whether `state` is one a render never leaves: `done`, `failed` or `cancelled`.
pub fn is_final(state: &str) -> bool {
    matches!(state, "done" | "failed" | "cancelled")
}

/// One `renders` row.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RenderRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub rev: i64,
    pub width: i32,
    pub height: i32,
    pub fps: i32,
    pub range_start: Option<f64>,
    pub range_end: Option<f64>,
    pub state: String,
    pub frame: i64,
    pub frames: i64,
    pub error: Option<String>,
    pub size: Option<i64>,
    pub created_at: OffsetDateTime,
    pub started_at: Option<OffsetDateTime>,
    pub finished_at: Option<OffsetDateTime>,
}

/// A render as the API answers it.
#[derive(Debug, Clone, Serialize)]
pub struct Render {
    pub id: Uuid,
    pub project_id: Uuid,
    /// The project revision that was rendered.
    pub rev: i64,
    pub width: i32,
    pub height: i32,
    pub fps: i32,
    /// `[start, end]` in seconds, or `null` for the whole project.
    pub range: Option<[f64; 2]>,
    /// `queued`, `running`, `done`, `failed` or `cancelled`.
    pub state: String,
    /// The frames captured so far.
    pub frame: i64,
    /// The frames the render has in all.
    pub frames: i64,
    pub error: Option<String>,
    /// The size of the file in bytes, once done.
    pub size: Option<i64>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub started_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub finished_at: Option<OffsetDateTime>,
    /// Where a signed-in caller downloads the MP4, once done.
    pub download_url: Option<String>,
}

/// The route that serves the file of the render `id`.
pub fn file_url(id: Uuid) -> String {
    format!("/api/renders/{id}/file")
}

impl From<RenderRow> for Render {
    fn from(row: RenderRow) -> Self {
        Render {
            download_url: (row.state == "done").then(|| file_url(row.id)),
            range: row.range_start.zip(row.range_end).map(|(start, end)| [start, end]),
            id: row.id,
            project_id: row.project_id,
            rev: row.rev,
            width: row.width,
            height: row.height,
            fps: row.fps,
            state: row.state,
            frame: row.frame,
            frames: row.frames,
            error: row.error,
            size: row.size,
            created_at: row.created_at,
            started_at: row.started_at,
            finished_at: row.finished_at,
        }
    }
}

/// What `GET /projects/{id}/renders` answers.
#[derive(Debug, Serialize)]
pub struct Renders {
    pub renders: Vec<Render>,
}

/// `POST /projects/{id}/renders`. Every field is optional: the size defaults to the project's
/// own, the frame rate to 30, the range to the whole project.
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RenderRequest {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub fps: Option<u32>,
    /// `[start, end]` in seconds.
    pub range: Option<[f64; 2]>,
}
