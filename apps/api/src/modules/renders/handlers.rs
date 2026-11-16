//! The render handlers: thin wrappers that read the caller and the path.

use axum::extract::{DefaultBodyLimit, Multipart, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{MethodRouter, post};
use porte::Identity;
use tronc::Result;
use tronc::json::Json;

use super::types::{Render, RenderRequest, Renders};
use super::{posted, service};
use crate::extract::id;
use crate::modules::projects::error::ProjectError;
use crate::state::AppState;

pub async fn create(
    State(state): State<AppState>,
    who: Identity,
    Path(raw): Path<String>,
    Json(body): Json<RenderRequest>,
) -> std::result::Result<(StatusCode, Json<Render>), ProjectError> {
    let render = service::enqueue(&state, who.user_id, id(&raw)?, &body).await?;
    Ok((StatusCode::ACCEPTED, Json(render)))
}

pub async fn list(State(state): State<AppState>, who: Identity, Path(raw): Path<String>) -> Result<Json<Renders>> {
    service::list(&state, who.user_id, id(&raw)?).await.map(Json)
}

pub async fn read(State(state): State<AppState>, who: Identity, Path(raw): Path<String>) -> Result<Json<Render>> {
    service::read(&state, who.user_id, id(&raw)?).await.map(Json)
}

pub async fn cancel(State(state): State<AppState>, who: Identity, Path(raw): Path<String>) -> Result<Response> {
    Ok(match service::cancel(&state, who.user_id, id(&raw)?).await? {
        Some(render) => (StatusCode::ACCEPTED, Json(render)).into_response(),
        None => StatusCode::NO_CONTENT.into_response(),
    })
}

async fn bundle(
    State(state): State<AppState>,
    headers: http::HeaderMap,
    multipart: Multipart,
) -> Result<(StatusCode, Json<Render>)> {
    let user_id = posted::caller(&state, &headers).await?;
    let length = headers.get(http::header::CONTENT_LENGTH).and_then(|value| value.to_str().ok()?.parse().ok());
    Ok((StatusCode::ACCEPTED, Json(posted::post(&state, user_id, length, multipart).await?)))
}

/// `POST /renders`: a bundle posted with a bearer token. It carries its own check, so it sits
/// outside `require_auth`, and its body streams to disk under the bundle cap.
pub fn posted() -> MethodRouter<AppState> {
    post(bundle).layer(DefaultBodyLimit::disable())
}
