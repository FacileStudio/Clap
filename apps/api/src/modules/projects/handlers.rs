//! The project handlers: thin wrappers that read the caller and the path.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use porte::Identity;
use tronc::Result;
use tronc::json::{Json, JsonLimit};

use super::error::ProjectError;
use super::source::BODY_MAX_BYTES;
use super::timeline::{self, EditsRequest};
use super::types::{Author, CreateRequest, ListQuery, PatchRequest, Project, ProjectEntry, SourceRequest};
use super::{patch, service, trash, write};
use crate::extract::{Params, id};
use crate::state::AppState;

type Body<T> = JsonLimit<T, BODY_MAX_BYTES>;

pub async fn list(
    State(state): State<AppState>,
    who: Identity,
    Params(query): Params<ListQuery>,
) -> Result<Json<Vec<ProjectEntry>>> {
    service::list(&state, who.user_id, &query).await.map(Json)
}

pub async fn create(
    State(state): State<AppState>,
    who: Identity,
    JsonLimit(body): Body<CreateRequest>,
) -> std::result::Result<(StatusCode, Json<Project>), ProjectError> {
    Ok((StatusCode::CREATED, Json(service::create(&state, &Author::of(&who), body).await?)))
}

pub async fn read(State(state): State<AppState>, who: Identity, Path(raw): Path<String>) -> Result<Json<Project>> {
    service::read(&state, who.user_id, id(&raw)?).await.map(Json)
}

pub async fn save(
    State(state): State<AppState>,
    who: Identity,
    Path(raw): Path<String>,
    JsonLimit(body): Body<SourceRequest>,
) -> std::result::Result<Json<Project>, ProjectError> {
    write::save(&state, &Author::of(&who), id(&raw)?, body).await.map(Json)
}

pub async fn update(
    State(state): State<AppState>,
    who: Identity,
    Path(raw): Path<String>,
    Json(body): Json<PatchRequest>,
) -> Result<Json<Project>> {
    patch::patch(&state, who.user_id, id(&raw)?, body).await.map(Json)
}

pub async fn remove(State(state): State<AppState>, who: Identity, Path(raw): Path<String>) -> Result<StatusCode> {
    trash::permanent(&state, who.user_id, id(&raw)?).await.map(|()| StatusCode::NO_CONTENT)
}

pub async fn edits(
    State(state): State<AppState>,
    who: Identity,
    Path(raw): Path<String>,
    JsonLimit(body): Body<EditsRequest>,
) -> std::result::Result<Json<Project>, ProjectError> {
    timeline::edits(&state, &Author::of(&who), id(&raw)?, body).await.map(Json)
}
