use axum::extract::Multipart;
use serde::Serialize;
use time::OffsetDateTime;
use tronc::{Error, Result};
use uuid::Uuid;

use super::upload::Stored;
use super::{files, intake, name, quota, sniff, worker};
use crate::extract::db;
use crate::modules::projects::access::{self, Access};
use crate::modules::projects::service::fetch;
use crate::state::AppState;

/// The columns every asset answer reads, as a literal for `concat!`.
macro_rules! columns {
    () => {
        "name, mime, size, sha256, kind, duration_ms, width, height, fps, has_audio, state, error, created_at, \
         poster IS NOT NULL AS has_poster, proxy IS NOT NULL AS has_proxy"
    };
}

/// One stored asset, as the API answers it.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Asset {
    /// The relative path the project's source writes to load it.
    pub name: String,
    pub mime: String,
    pub size: i64,
    pub sha256: String,
    /// `video`, `audio`, `image` or `other`.
    pub kind: String,
    pub duration_ms: Option<i64>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub fps: Option<f64>,
    pub has_audio: bool,
    /// `probing` until the background task has looked at the file, then `ready` or `failed`.
    pub state: String,
    pub error: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(skip)]
    pub has_poster: bool,
    #[serde(skip)]
    pub has_proxy: bool,
    /// Where a signed-in caller reads the bytes as uploaded.
    #[sqlx(skip)]
    pub url: String,
    /// Where the poster frame is, once there is one.
    #[sqlx(skip)]
    pub poster_url: Option<String>,
    /// Where the 720p proxy is, once there is one.
    #[sqlx(skip)]
    pub proxy_url: Option<String>,
}

/// What `GET /projects/{id}/assets` answers.
#[derive(Debug, Serialize)]
pub struct Assets {
    pub assets: Vec<Asset>,
}

/// The route that serves the asset `name` of `project`.
pub fn url(project: Uuid, name: &str) -> String {
    format!("/api/projects/{project}/assets/{name}")
}

fn located(project: Uuid, asset: Asset) -> Asset {
    let base = url(project, &asset.name);
    Asset {
        poster_url: asset.has_poster.then(|| format!("{base}?variant=poster")),
        proxy_url: asset.has_proxy.then(|| format!("{base}?variant=proxy")),
        url: base,
        ..asset
    }
}

async fn insert(state: &AppState, user_id: i64, project: Uuid, name: &str, stored: &Stored) -> Result<Asset> {
    let kind = sniff::kind(stored.mime);
    sqlx::query_as::<_, Asset>(concat!(
        "INSERT INTO assets (project_id, name, mime, size, sha256, kind, state, created_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) ON CONFLICT (project_id, name) DO UPDATE SET \
         mime = EXCLUDED.mime, size = EXCLUDED.size, sha256 = EXCLUDED.sha256, kind = EXCLUDED.kind, \
         state = EXCLUDED.state, created_by = EXCLUDED.created_by, created_at = now(), duration_ms = NULL, \
         width = NULL, height = NULL, fps = NULL, has_audio = false, poster = NULL, proxy = NULL, error = NULL \
         RETURNING ",
        columns!()
    ))
    .bind(project)
    .bind(name)
    .bind(stored.mime)
    .bind(stored.size as i64)
    .bind(&stored.sha256)
    .bind(kind)
    .bind(if kind == "other" { "ready" } else { "probing" })
    .bind(user_id)
    .fetch_one(&state.pool)
    .await
    .map_err(db)
}

/// `POST /projects/{id}/assets`: streams the file to disk, types it by its content, stores it
/// once under the name asked for or one made from the file name, then probes it in the
/// background. The same name again replaces the asset.
pub async fn upload(state: &AppState, user_id: i64, project: Uuid, multipart: Multipart) -> Result<Asset> {
    access::check(state, user_id, project, Access::Editor).await?;
    if fetch(state, project).await?.trashed_at.is_some() {
        return Err(Error::failed("the project is in the trash"));
    }
    quota::room(state, project, "", 1).await?;
    let intake = intake::read(state, multipart).await?;
    let Some((file_name, stored)) = intake.file else {
        return Err(Error::invalid("the multipart body has no `file` part"));
    };
    let name = name::chosen(intake.name.as_deref(), &file_name)?;
    quota::room(state, project, &name, stored.size).await?;
    let adopted = files::adopt(&state.data_dir, stored.file.path(), &stored.sha256).await;
    adopted.map_err(|error| Error::internal("failed to store a file", error))?;
    let asset = insert(state, user_id, project, &name, &stored).await?;
    stored.file.keep();
    state.events.asset(project, &asset.name, if asset.state == "ready" { "ready" } else { "probing" });
    if asset.state == "probing" {
        worker::spawn(state.clone(), worker::Job { project, name: asset.name.clone(), sha256: asset.sha256.clone() });
    }
    Ok(located(project, asset))
}

/// `GET /projects/{id}/assets`: the assets of a project the caller may read, by name.
pub async fn list(state: &AppState, user_id: i64, project: Uuid) -> Result<Assets> {
    access::check(state, user_id, project, Access::Viewer).await?;
    Ok(Assets { assets: catalog(state, project).await? })
}

/// Every asset of `project`, by name, whoever asks: callers check access first.
pub async fn catalog(state: &AppState, project: Uuid) -> Result<Vec<Asset>> {
    let rows =
        sqlx::query_as::<_, Asset>(concat!("SELECT ", columns!(), " FROM assets WHERE project_id = $1 ORDER BY name"))
            .bind(project)
            .fetch_all(&state.pool)
            .await
            .map_err(db)?;
    Ok(rows.into_iter().map(|asset| located(project, asset)).collect())
}

/// `DELETE /projects/{id}/assets/{name}`: forgets the asset; the sweep removes its bytes, its
/// poster and its proxy once no project names them.
pub async fn remove(state: &AppState, user_id: i64, project: Uuid, name: &str) -> Result<()> {
    access::check(state, user_id, project, Access::Editor).await?;
    let deleted = sqlx::query("DELETE FROM assets WHERE project_id = $1 AND name = $2")
        .bind(project)
        .bind(name)
        .execute(&state.pool)
        .await
        .map_err(db)?;
    if deleted.rows_affected() == 0 {
        return Err(Error::not_found("asset not found"));
    }
    state.events.asset(project, name, "deleted");
    Ok(())
}

/// The asset `name` of `project`, whoever asks: callers check access first.
pub async fn find(state: &AppState, project: Uuid, name: &str) -> Result<Asset> {
    let query = concat!("SELECT ", columns!(), " FROM assets WHERE project_id = $1 AND name = $2");
    let found = sqlx::query_as::<_, Asset>(query).bind(project).bind(name).fetch_optional(&state.pool).await;
    found.map_err(db)?.map(|asset| located(project, asset)).ok_or_else(|| Error::not_found("asset not found"))
}
