use std::collections::HashMap;

use axum::Router;
use axum::extract::{Path, State};
use axum::http::{HeaderValue, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use porte::Identity;
use porte::session::Manager;
use serde::Deserialize;
use tronc::Error;
use uuid::Uuid;

use super::{TTL, now};
use crate::extract::{Params, id};
use crate::modules::assets::{name, service as assets};
use crate::modules::projects::access::{self, Access};
use crate::modules::projects::error::ProjectError;
use crate::modules::projects::service::fetch;
use crate::state::AppState;

/// The `Content-Security-Policy` of every frame. `sandbox` without `allow-same-origin` gives the
/// document an opaque origin: it runs the author's scripts, and cannot read the app's cookies,
/// storage or API. No `connect-src`, so nothing in it reaches the network from script.
pub const POLICY: &str = "sandbox allow-scripts; default-src 'none'; img-src 'self' data: blob:; \
    media-src 'self' blob:; style-src 'unsafe-inline'; script-src 'unsafe-inline'; font-src 'self' data:; \
    base-uri 'none'; form-action 'none'; frame-ancestors 'self'";

#[derive(Deserialize)]
struct FrameQuery {
    scene: Option<usize>,
}

async fn compiled(state: &AppState, id: Uuid, only: Option<usize>) -> Result<String, ProjectError> {
    let row = fetch(state, id).await?;
    let parsed = scene::parse(&row.source).map_err(ProjectError::Invalid)?;
    let catalog = assets::catalog(state, id).await?;
    let proxied: HashMap<&str, bool> = catalog.iter().map(|asset| (asset.name.as_str(), asset.has_proxy)).collect();
    let token = state.frame_key.token(id, now() + TTL.as_secs());
    let resolve = |path: &str| {
        let (name, fragment) = name::referenced(path);
        let variant = if *proxied.get(name)? { "&variant=proxy" } else { "" };
        Some(format!("{}?frame={token}{variant}{fragment}", assets::url(id, name)))
    };
    let options = scene::Options::new().assets(&resolve);
    match only {
        None => Ok(scene::compile(&parsed, &options)),
        Some(index) => scene::compile_scene(&parsed, index, &options)
            .ok_or_else(|| Error::not_found(format!("the project has no scene {index}")).into()),
    }
}

async fn frame(
    State(state): State<AppState>,
    who: Identity,
    Path(raw): Path<String>,
    Params(query): Params<FrameQuery>,
) -> Result<Response, ProjectError> {
    let project = id(&raw)?;
    access::check(&state, who.user_id, project, Access::Viewer).await?;
    let html = compiled(&state, project, query.scene).await?;
    let headers = [
        (header::CONTENT_TYPE, HeaderValue::from_static("text/html; charset=utf-8")),
        (header::CONTENT_SECURITY_POLICY, HeaderValue::from_static(POLICY)),
        (header::X_FRAME_OPTIONS, HeaderValue::from_static("SAMEORIGIN")),
        (header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff")),
        (header::CACHE_CONTROL, HeaderValue::from_static("no-store")),
        (header::REFERRER_POLICY, HeaderValue::from_static("no-referrer")),
    ];
    Ok((headers, html).into_response())
}

async fn cookieless(mut response: Response) -> Response {
    response.headers_mut().remove(header::SET_COOKIE);
    response
}

/// `GET /projects/{id}/frame?scene=n`, behind `require_auth`: the project compiled to one HTML
/// document, or its scene `n` (from 0), with [`POLICY`]. Each asset the source names loads through
/// a signed link, a video through its 720p proxy once that exists. The only route that answers user HTML
/// as HTML. The response never carries a cookie, whatever a layer inside it set.
pub fn routes(sessions: &Manager) -> Router<AppState> {
    Router::new()
        .route("/projects/{id}/frame", get(frame).route_layer(sessions.require_auth()))
        .layer(axum::middleware::map_response(cookieless))
}
