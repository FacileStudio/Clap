//! The background task of one asset: `ffprobe`, then a poster and a 720p proxy for a video.

use uuid::Uuid;

use super::probe::{self, Probe};
use super::{derive, files};
use crate::state::AppState;

/// One asset to look at: the row it is, and the bytes it had when the task was started.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Job {
    /// The project.
    #[sqlx(rename = "project_id")]
    pub project: Uuid,
    /// The asset name.
    pub name: String,
    /// The digest of the bytes. A row uploaded again since holds another, and is left alone.
    pub sha256: String,
}

struct Facts {
    probe: Probe,
    kind: &'static str,
    poster: Option<String>,
}

async fn kind_of(state: &AppState, job: &Job) -> Option<String> {
    sqlx::query_scalar("SELECT kind FROM assets WHERE project_id = $1 AND name = $2 AND sha256 = $3")
        .bind(job.project)
        .bind(&job.name)
        .bind(&job.sha256)
        .fetch_optional(&state.pool)
        .await
        .ok()
        .flatten()
}

async fn poster(state: &AppState, job: &Job, probe: &Probe) -> Option<String> {
    let relative = files::poster(&job.sha256);
    let at = probe.duration_ms.map_or(0, |duration| (duration / 2).min(1000));
    let input = files::path(&state.data_dir, &job.sha256);
    match derive::poster(&state.tools, &input, at, &state.data_dir.join(&relative)).await {
        Ok(()) => Some(relative),
        Err(error) => {
            tracing::warn!(asset = %job.name, %error, "assets: no poster");
            None
        }
    }
}

async fn inspect(state: &AppState, job: &Job, kind: &str) -> Result<Facts, String> {
    let probed = probe::probe(&state.tools, &files::path(&state.data_dir, &job.sha256)).await;
    let probe = match (kind, probed) {
        (_, Ok(probe)) => probe,
        ("image", Err(_)) => return Ok(Facts { probe: Probe::default(), kind: "image", poster: None }),
        (_, Err(error)) => return Err(error),
    };
    match (kind, probe.has_video) {
        ("video", true) => Ok(Facts { poster: poster(state, job, &probe).await, probe, kind: "video" }),
        ("video", false) if probe.has_audio => Ok(Facts { probe, kind: "audio", poster: None }),
        ("video", false) => Err("the file has no picture and no sound".to_owned()),
        ("audio", _) if !probe.has_audio => Err("the file has no sound".to_owned()),
        ("audio", _) => {
            Ok(Facts { probe: Probe { width: None, height: None, fps: None, ..probe }, kind: "audio", poster: None })
        }
        _ => Ok(Facts { probe: Probe { duration_ms: None, fps: None, ..probe }, kind: "image", poster: None }),
    }
}

async fn settle(state: &AppState, job: &Job, facts: &Result<Facts, String>) -> Result<u64, sqlx::Error> {
    let query = "UPDATE assets SET state = $4, error = $5, kind = coalesce($6, kind), duration_ms = $7, \
                 width = $8, height = $9, fps = $10, has_audio = $11, poster = $12 \
                 WHERE project_id = $1 AND name = $2 AND sha256 = $3";
    let bound = sqlx::query(query).bind(job.project).bind(&job.name).bind(&job.sha256);
    let none = Probe::default();
    let (probe, bound) = match facts {
        Ok(facts) => (&facts.probe, bound.bind("ready").bind(None::<String>).bind(Some(facts.kind))),
        Err(error) => (&none, bound.bind("failed").bind(Some(error.clone())).bind(None::<&str>)),
    };
    let poster = facts.as_ref().ok().and_then(|facts| facts.poster.clone());
    let bound = bound.bind(probe.duration_ms).bind(probe.width).bind(probe.height).bind(probe.fps);
    Ok(bound.bind(probe.has_audio).bind(poster).execute(&state.pool).await?.rows_affected())
}

async fn proxy(state: &AppState, job: &Job) {
    let relative = files::proxy(&job.sha256);
    let input = files::path(&state.data_dir, &job.sha256);
    if let Err(error) = derive::proxy(&state.tools, &input, &state.data_dir.join(&relative)).await {
        return tracing::warn!(asset = %job.name, %error, "assets: no proxy");
    }
    let stored = sqlx::query("UPDATE assets SET proxy = $4 WHERE project_id = $1 AND name = $2 AND sha256 = $3")
        .bind(job.project)
        .bind(&job.name)
        .bind(&job.sha256)
        .bind(relative)
        .execute(&state.pool)
        .await;
    if stored.is_ok_and(|done| done.rows_affected() > 0) {
        state.events.asset(job.project, &job.name, "ready");
    }
}

/// Looks at one asset, one at a time across the process: probes it, grabs the poster of a
/// video, stores what it found and tells open editors, then encodes the proxy of a video and
/// tells them again. An asset replaced or deleted meanwhile is left alone.
pub async fn run(state: AppState, job: Job) {
    let Ok(_slot) = state.media.clone().acquire_owned().await else { return };
    let Some(kind) = kind_of(&state, &job).await else { return };
    let facts = inspect(&state, &job, &kind).await;
    match settle(&state, &job, &facts).await {
        Ok(0) => return,
        Ok(_) => state.events.asset(job.project, &job.name, if facts.is_ok() { "ready" } else { "failed" }),
        Err(error) => return tracing::error!(asset = %job.name, %error, "assets: could not store a probe"),
    }
    if facts.is_ok_and(|facts| facts.kind == "video") {
        proxy(&state, &job).await;
    }
}

/// Starts [`run`] for `job` on its own task.
pub fn spawn(state: AppState, job: Job) {
    tokio::spawn(run(state, job));
}

/// Starts again, at boot, every asset a previous process left `probing` or without its proxy.
pub async fn resume(state: &AppState) {
    files::clear_incoming(&state.data_dir).await;
    let left = sqlx::query_as::<_, Job>(
        "SELECT project_id, name, sha256 FROM assets \
         WHERE state = 'probing' OR (state = 'ready' AND kind = 'video' AND proxy IS NULL) ORDER BY created_at",
    )
    .fetch_all(&state.pool)
    .await;
    match left {
        Ok(jobs) => jobs.into_iter().for_each(|job| spawn(state.clone(), job)),
        Err(error) => tracing::error!(%error, "assets: could not list unfinished assets"),
    }
}
