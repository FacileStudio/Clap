//! `POST /renders`: another suite app, acting for a person, posts a compiled scene bundle and
//! gets the same render job a project gets.

use std::path::{Path, PathBuf};

use axum::extract::Multipart;
use axum::extract::multipart::{Field, MultipartError};
use http::HeaderMap;
use porte::mcpauth::VerifyError;
use tokio::io::AsyncWriteExt;
use tronc::{Error, Result};

use super::bundle::JobFile;
use super::spec;
use super::types::{Render, RenderRequest};
use crate::limits::RENDER_BUNDLE_MAX_FILES;
use crate::modules::assets::inflight::Ask;
use crate::modules::assets::name;
use crate::modules::assets::temporary::Temporary;
use crate::state::AppState;

/// The scope a bearer of `POST /renders` must hold.
pub const SCOPE: &str = "clap.write";

/// The account a bearer token acts for: a Registre token for this app, verified by porte, whose
/// account is created on a first call. `401` without a valid token, `403` without [`SCOPE`].
pub async fn caller(state: &AppState, headers: &HeaderMap) -> Result<i64> {
    let header = headers.get(http::header::AUTHORIZATION).and_then(|value| value.to_str().ok()).unwrap_or_default();
    let token = header.get(..7).filter(|scheme| scheme.eq_ignore_ascii_case("bearer ")).map(|_| header[7..].trim());
    let Some(token) = token.filter(|token| !token.is_empty()) else {
        return Err(Error::unauthorized("missing bearer token"));
    };
    let verifier = state.machine.as_ref().ok_or_else(|| Error::unauthorized("invalid token"))?;
    let info = match verifier.verify(token).await {
        Ok(info) => info,
        Err(VerifyError::InvalidToken) => return Err(Error::unauthorized("invalid token")),
        Err(VerifyError::Unavailable) => return Err(Error::unavailable("the token could not be checked")),
    };
    if !info.scopes.iter().any(|scope| scope == SCOPE) {
        return Err(Error::forbidden("insufficient scope"));
    }
    info.user_id.parse().map_err(|error| Error::internal("the token names no local account", error))
}

struct Intake {
    bundle: PathBuf,
    max: u64,
    spec: Option<RenderRequest>,
    files: usize,
    bytes: u64,
}

fn invalid(error: MultipartError) -> Error {
    Error::invalid(error.body_text())
}

fn io(error: std::io::Error) -> Error {
    Error::internal("failed to store a bundle", error)
}

async fn described(mut field: Field<'_>) -> Result<RenderRequest> {
    let mut text = Vec::new();
    while let Some(chunk) = field.chunk().await.map_err(invalid)? {
        text.extend_from_slice(&chunk);
        if text.len() > 4096 {
            return Err(Error::invalid("the `spec` part is too long"));
        }
    }
    serde_json::from_slice(&text)
        .map_err(|error| Error::invalid(format!("the `spec` part is not a render request: {error}")))
}

async fn stored(intake: &mut Intake, mut field: Field<'_>) -> Result<()> {
    let path = field.file_name().unwrap_or_default().to_owned();
    if !name::valid(&path) || intake.files >= RENDER_BUNDLE_MAX_FILES {
        return Err(Error::invalid(format!(
            "a bundle holds at most {RENDER_BUNDLE_MAX_FILES} files, each named by a relative path of letters, \
             digits, `.`, `_` and `-`: {path:?} is refused"
        )));
    }
    let target = intake.bundle.join(&path);
    tokio::fs::create_dir_all(target.parent().unwrap_or(&intake.bundle)).await.map_err(io)?;
    let mut file = tokio::fs::File::create(&target).await.map_err(io)?;
    while let Some(chunk) = field.chunk().await.map_err(invalid)? {
        intake.bytes += chunk.len() as u64;
        if intake.bytes > intake.max {
            return Err(Error::too_large(format!("a bundle holds at most {} MiB", intake.max >> 20)));
        }
        file.write_all(&chunk).await.map_err(io)?;
    }
    intake.files += 1;
    file.flush().await.map_err(io)
}

async fn received(staging: &Path, mut multipart: Multipart, max: u64) -> Result<Intake> {
    let mut intake = Intake { bundle: staging.join("bundle"), max, spec: None, files: 0, bytes: 0 };
    while let Some(field) = multipart.next_field().await.map_err(invalid)? {
        match field.name() {
            Some("spec") => intake.spec = Some(described(field).await?),
            Some("file") => stored(&mut intake, field).await?,
            _ => {}
        }
    }
    Ok(intake)
}

/// Queues a render of a posted bundle for `user_id`. The multipart body carries a `spec` part,
/// the JSON `{width, height, fps?, range?}`, and one `file` part per file of the bundle, whose
/// file name is its relative path; `index.html` is required. The files stream to disk under the
/// same caps as a project render, then the bundle moves into a job directory and `job.json` is
/// written last.
pub async fn post(state: &AppState, user_id: i64, length: Option<u64>, multipart: Multipart) -> Result<Render> {
    super::status::serving(state).await?;
    let (per_user, space) = (state.limits.uploads, format!("bundles:{user_id}"));
    let room = state.limits.bundle.saturating_mul(per_user as u64);
    let ask = Ask { user: user_id, per_user, space: &space, length, file_max: state.limits.bundle, room };
    let slot = state.inflight.enter(ask)?;
    let staging = crate::modules::assets::files::incoming(&state.data_dir).join(uuid::Uuid::new_v4().to_string());
    tokio::fs::create_dir_all(&staging).await.map_err(io)?;
    let guard = Temporary::at(staging.clone());
    let intake = received(&staging, multipart, slot.bytes()).await?;
    let request = intake.spec.ok_or_else(|| Error::invalid("the multipart body has no `spec` part"))?;
    let spec = spec::posted(&request, &state.limits.render)?;
    if !tokio::fs::try_exists(intake.bundle.join("index.html")).await.unwrap_or(false) {
        return Err(Error::invalid("the bundle has no `index.html`"));
    }
    let row = super::queue::insert(state, user_id, (None, 0), &spec).await?;
    let moved = super::stage::adopt(&state.data_dir, &intake.bundle, &JobFile::of(&row, &spec)).await;
    if let Err(error) = moved {
        super::service::discard(state, row.id).await;
        return Err(Error::internal("failed to write a render job", error));
    }
    drop(guard);
    state.events.render(row.id);
    Ok(row.into())
}
