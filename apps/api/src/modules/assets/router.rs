use axum::Router;
use axum::extract::{DefaultBodyLimit, Multipart, Path, State};
use axum::http::StatusCode;
use axum::routing::{MethodRouter, delete, get};
use porte::Identity;
use porte::session::Manager;
use tronc::Result;
use tronc::json::Json;

use super::serve::serve;
use super::service::{self, Asset, Assets};
use crate::extract::id;
use crate::state::AppState;

async fn upload(
    State(state): State<AppState>,
    who: Identity,
    Path(raw): Path<String>,
    headers: http::HeaderMap,
    multipart: Multipart,
) -> Result<(StatusCode, Json<Asset>)> {
    let length = headers.get(http::header::CONTENT_LENGTH).and_then(|value| value.to_str().ok()?.parse().ok());
    let asset = service::upload(&state, who.user_id, id(&raw)?, service::Body { multipart, length }).await?;
    Ok((StatusCode::CREATED, Json(asset)))
}

async fn list(State(state): State<AppState>, who: Identity, Path(raw): Path<String>) -> Result<Json<Assets>> {
    service::list(&state, who.user_id, id(&raw)?).await.map(Json)
}

async fn remove(
    State(state): State<AppState>,
    who: Identity,
    Path((raw, name)): Path<(String, String)>,
) -> Result<StatusCode> {
    service::remove(&state, who.user_id, id(&raw)?, &name).await.map(|()| StatusCode::NO_CONTENT)
}

fn delete_one(sessions: &Manager) -> MethodRouter<AppState> {
    delete(remove).route_layer(sessions.require_auth())
}

/// The asset routes. Upload, list and delete sit behind `require_auth`. The upload body has no
/// framework limit: it streams to disk and the stream itself stops at `ASSET_MAX_BYTES`. Reading
/// an asset takes a session or a frame link, so it sits behind the optional session layer.
pub fn routes(sessions: &Manager) -> Router<AppState> {
    let collection = get(list).post(upload).layer(DefaultBodyLimit::disable()).route_layer(sessions.require_auth());
    let one = get(serve).route_layer(sessions.optional()).merge(delete_one(sessions));
    Router::new().route("/projects/{id}/assets", collection).route("/projects/{id}/assets/{*name}", one)
}
