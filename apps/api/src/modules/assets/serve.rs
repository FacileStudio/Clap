use std::path::PathBuf;

use axum::body::Body;
use axum::extract::{Path, Request, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::Response;
use porte::Identity;
use serde::Deserialize;
use tower::ServiceExt;
use tower_http::services::ServeFile;
use tronc::{Error, Result};
use uuid::Uuid;

use super::service::Asset;
use super::{files, service};
use crate::extract::{Params, id};
use crate::modules::frame;
use crate::modules::projects::access::{self, Access};
use crate::state::AppState;

/// The policy of every asset response: opened on its own, an SVG gets an opaque origin and runs
/// no script.
pub const POLICY: &str = "sandbox; default-src 'none'; style-src 'unsafe-inline'";

/// Which file of an asset a request reads.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Variant {
    /// The bytes as uploaded.
    #[default]
    Original,
    /// The poster frame of a video, a JPEG.
    Poster,
    /// The 720p H.264 proxy of a video.
    Proxy,
}

/// `?frame=`: the token a frame's asset links carry. `?variant=`: `poster` or `proxy`.
#[derive(Deserialize)]
pub struct AssetQuery {
    frame: Option<String>,
    #[serde(default)]
    variant: Variant,
}

async fn allowed(state: &AppState, who: Option<Identity>, project: Uuid, token: Option<&str>) -> Result<bool> {
    match (token, who) {
        (Some(token), _) if state.frame_key.allows(project, token, frame::now()) => Ok(true),
        (Some(_), _) => Err(Error::not_found("asset not found")),
        (None, Some(who)) => access::check(state, who.user_id, project, Access::Viewer).await.map(|_| false),
        (None, None) => Err(Error::unauthorized("missing auth")),
    }
}

fn chosen(state: &AppState, asset: &Asset, variant: Variant) -> Result<(PathBuf, String)> {
    let missing = || Error::not_found("the asset has no such variant yet");
    match variant {
        Variant::Original => Ok((files::path(&state.data_dir, &asset.sha256), asset.mime.clone())),
        Variant::Poster if asset.has_poster => {
            Ok((state.data_dir.join(files::poster(&asset.sha256)), "image/jpeg".to_owned()))
        }
        Variant::Proxy if asset.has_proxy => {
            Ok((state.data_dir.join(files::proxy(&asset.sha256)), "video/mp4".to_owned()))
        }
        _ => Err(missing()),
    }
}

fn dress(response: &mut Response, mime: &str, framed: bool) {
    let headers = response.headers_mut();
    if let Ok(mime) = HeaderValue::from_str(mime) {
        headers.insert(header::CONTENT_TYPE, mime);
    }
    headers.insert(header::CONTENT_SECURITY_POLICY, HeaderValue::from_static(POLICY));
    headers.insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("private, max-age=3600"));
    headers.insert(header::CONTENT_DISPOSITION, HeaderValue::from_static("inline"));
    headers.remove(header::SET_COOKIE);
    if framed {
        headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
        headers.insert("cross-origin-resource-policy", HeaderValue::from_static("cross-origin"));
    }
}

/// `GET /projects/{id}/assets/{name}`: the bytes, with range requests so a video seeks. A frame
/// link is answered with `Access-Control-Allow-Origin: *`, which a font loaded from the frame's
/// opaque origin needs; the link itself is the right to read.
pub async fn serve(
    State(state): State<AppState>,
    who: Option<Identity>,
    Path((raw, name)): Path<(String, String)>,
    Params(query): Params<AssetQuery>,
    request: Request,
) -> Result<Response> {
    let project = id(&raw)?;
    let framed = allowed(&state, who, project, query.frame.as_deref()).await?;
    let asset = service::find(&state, project, &name).await?;
    let (path, mime) = chosen(&state, &asset, query.variant)?;
    let answer = ServeFile::new(path).oneshot(request).await;
    let answer = answer.map_err(|error| Error::internal("failed to read an asset", error))?;
    if answer.status() == StatusCode::NOT_FOUND {
        return Err(Error::internal("an asset file is missing", std::io::Error::other(asset.sha256)));
    }
    let mut response = answer.map(Body::new);
    dress(&mut response, &mime, framed);
    Ok(response)
}
