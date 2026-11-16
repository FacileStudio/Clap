use axum::Router;
use axum::body::Body;
use axum::extract::{Path, Request, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::Response;
use axum::routing::get;
use tower::ServiceExt;
use tower_http::services::ServeFile;
use tronc::http::rate_limit;
use tronc::json::Json;
use tronc::{Error, Result};

use super::guard::limiter;
use super::info::{self, PublicRender, shared};
use crate::modules::frame::cookieless;
use crate::modules::renders::status::output;
use crate::state::AppState;

async fn render(State(state): State<AppState>, Path(token): Path<String>) -> Result<Json<PublicRender>> {
    info::render(&state, &token).await.map(Json)
}

fn dress(response: &mut Response) {
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("video/mp4"));
    headers.insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("private, max-age=300"));
    headers.insert(header::CONTENT_DISPOSITION, HeaderValue::from_static("inline"));
    headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
    headers.insert("cross-origin-resource-policy", HeaderValue::from_static("cross-origin"));
}

async fn video(State(state): State<AppState>, Path(token): Path<String>, request: Request) -> Result<Response> {
    let shared = shared(&state, &token).await?;
    let answer = ServeFile::new(output(&state.data_dir, shared.id)).oneshot(request).await;
    let answer = answer.map_err(|error| Error::internal("failed to read a render", error))?;
    if answer.status() == StatusCode::NOT_FOUND {
        return Err(Error::internal("a render file is missing", std::io::Error::other(shared.id.to_string())));
    }
    let mut response = answer.map(Body::new);
    dress(&mut response);
    Ok(response)
}

/// The public routes under `/api`: the facts of the shared render, rate limited to 5 requests a
/// second per client with a burst of 60, and its MP4 with range requests. A player asks for many
/// ranges, so the video sits outside the limit, as Diapo's public assets do; the share token is
/// the right to read it. No session, no cookie.
pub fn api_routes() -> Router<AppState> {
    Router::new()
        .route("/public/renders/{token}", get(render))
        .route_layer(axum::middleware::from_fn_with_state(limiter(), rate_limit))
        .route("/public/renders/{token}/video.mp4", get(video))
        .layer(axum::middleware::map_response(cookieless))
}
