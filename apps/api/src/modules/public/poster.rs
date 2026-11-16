//! The poster of a finished render: one JPEG frame, made by `ffmpeg` once the render is kept.

use std::path::{Path, PathBuf};

use axum::body::Body;
use axum::http::{HeaderValue, header};
use axum::response::Response;
use tronc::{Error, Result};
use uuid::Uuid;

use crate::state::AppState;

/// The widest poster, in pixels; a narrower render keeps its own width.
pub const MAX_WIDTH: i32 = 640;

/// `DATA_DIR/outputs/<render id>.jpg`, beside the MP4.
pub fn path(data_dir: &Path, render: Uuid) -> PathBuf {
    data_dir.join("outputs").join(format!("{render}.jpg"))
}

/// The public route of the poster behind the link `token`.
pub fn route(token: &str) -> String {
    format!("/v/{token}/thumbnail.jpg")
}

/// The poster size of a render of `width` by `height`, as `ffmpeg` scales it: at most
/// [`MAX_WIDTH`] wide, the height rounded to an even number.
pub fn size(width: i32, height: i32) -> (i32, i32) {
    let shown = width.clamp(1, MAX_WIDTH);
    let tall = (f64::from(height) * f64::from(shown) / f64::from(width.max(1)) / 2.0).round() as i32 * 2;
    (shown, tall.max(2))
}

/// Whether the render has its poster.
pub async fn exists(state: &AppState, render: Uuid) -> bool {
    tokio::fs::try_exists(path(&state.data_dir, render)).await.unwrap_or(false)
}

/// The poster of `render`, or `404` while none exists.
pub async fn serve(state: &AppState, render: Uuid) -> Result<Response> {
    let bytes = tokio::fs::read(path(&state.data_dir, render)).await.map_err(|_| Error::not_found("not found"))?;
    let mut response = Response::new(Body::from(bytes));
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("image/jpeg"));
    headers.insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("public, max-age=300"));
    headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
    Ok(response)
}

/// Makes the poster of the kept render `render`, a frame from its middle, at most one second
/// in. A failure is logged: the share page then has no picture.
pub async fn make(state: &AppState, render: Uuid, duration_ms: i64) {
    let video = crate::modules::renders::status::output(&state.data_dir, render);
    let at = (duration_ms / 2).min(1000);
    let made = crate::modules::assets::derive::poster(&state.tools, &video, at, &path(&state.data_dir, render)).await;
    if let Err(error) = made {
        tracing::warn!(%render, %error, "renders: no poster");
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_poster_keeps_the_render_ratio_at_most_640_wide() {
        assert_eq!(super::size(1920, 1080), (640, 360));
        assert_eq!(super::size(1080, 1920), (640, 1138));
        assert_eq!(super::size(320, 180), (320, 180));
    }
}
