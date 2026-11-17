use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, patch};
use porte::Identity;
use tronc::Result;
use tronc::json::Json;

use super::service::{self, Change, Marker, Markers, NewMarker};
use crate::extract::id;
use crate::state::AppState;

async fn list(State(state): State<AppState>, who: Identity, Path(raw): Path<String>) -> Result<Json<Markers>> {
    service::list(&state, who.user_id, id(&raw)?).await.map(Json)
}

async fn create(
    State(state): State<AppState>,
    who: Identity,
    Path(raw): Path<String>,
    Json(body): Json<NewMarker>,
) -> Result<(StatusCode, Json<Marker>)> {
    let marker = service::create(&state, who.user_id, id(&raw)?, body).await?;
    Ok((StatusCode::CREATED, Json(marker)))
}

async fn change(
    State(state): State<AppState>,
    who: Identity,
    Path((raw, marker)): Path<(String, String)>,
    Json(body): Json<Change>,
) -> Result<Json<Marker>> {
    service::change(&state, who.user_id, id(&raw)?, id(&marker)?, body).await.map(Json)
}

async fn remove(
    State(state): State<AppState>,
    who: Identity,
    Path((raw, marker)): Path<(String, String)>,
) -> Result<StatusCode> {
    service::remove(&state, who.user_id, id(&raw)?, id(&marker)?).await.map(|()| StatusCode::NO_CONTENT)
}

/// `GET` and `POST /projects/{id}/markers`, `PATCH` and `DELETE /projects/{id}/markers/{marker}`,
/// behind `require_auth`.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/projects/{id}/markers", get(list).post(create))
        .route("/projects/{id}/markers/{marker}", patch(change).delete(remove))
}
