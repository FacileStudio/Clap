use serde::Serialize;
use tronc::{Error, Result};
use uuid::Uuid;

use super::poster;
use crate::extract::db;
use crate::modules::share::service::resolve;
use crate::state::AppState;

/// The render a live link shows: the latest finished render of its project.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Shared {
    /// The render id. It never leaves the server.
    pub id: Uuid,
    /// The project title.
    pub title: String,
    pub width: i32,
    pub height: i32,
    pub fps: i32,
    pub frames: i64,
}

impl Shared {
    /// The length of the video in milliseconds.
    pub fn duration_ms(&self) -> i64 {
        self.frames * 1000 / i64::from(self.fps.max(1))
    }
}

/// What `GET /public/renders/{token}` answers: enough to play the video, and nothing else. No
/// source, no project id, no render id, no space.
#[derive(Debug, Serialize)]
pub struct PublicRender {
    pub title: String,
    pub duration_ms: i64,
    pub width: i32,
    pub height: i32,
    /// The route of the poster frame, or `null` while none exists.
    pub poster: Option<String>,
    /// The route of the MP4.
    pub video: String,
}

/// The public route of the video behind the link `token`.
pub fn video(token: &str) -> String {
    format!("/api/public/renders/{token}/video.mp4")
}

/// The latest finished render of the project the live link `token` names. `404` when the link
/// is not live, the project is in the trash or no render of it has finished.
pub async fn shared(state: &AppState, token: &str) -> Result<Shared> {
    let project = resolve(state, token).await?;
    sqlx::query_as::<_, Shared>(
        "SELECT r.id, p.title, r.width, r.height, r.fps, r.frames FROM renders r JOIN projects p ON p.id = r.project_id \
         WHERE r.project_id = $1 AND r.state = 'done' ORDER BY r.finished_at DESC, r.id LIMIT 1",
    )
    .bind(project)
    .fetch_optional(&state.pool)
    .await
    .map_err(db)?
    .ok_or_else(|| Error::not_found("not found"))
}

/// `GET /public/renders/{token}`.
pub async fn render(state: &AppState, token: &str) -> Result<PublicRender> {
    let shared = shared(state, token).await?;
    let pictured = poster::exists(state, shared.id).await;
    Ok(PublicRender {
        duration_ms: shared.duration_ms(),
        title: shared.title,
        width: shared.width,
        height: shared.height,
        poster: pictured.then(|| poster::route(token)),
        video: video(token),
    })
}
