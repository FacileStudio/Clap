//! The render handlers: thin wrappers that read the caller and the path.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use porte::Identity;
use tronc::Result;
use tronc::json::Json;

use super::service;
use super::types::{Render, RenderRequest, Renders};
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
