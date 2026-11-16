//! The background task of one asset: a media job in the render worker, whose probe, poster,
//! filmstrip, waveform and proxy the API stores as they come.

use clap_media::{Make, Probe, Step, files as made};
use uuid::Uuid;

use super::facts::{Facts, Fault, inspect};
use super::files;
use super::media::{MediaJob, Seen};
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

/// What a media job on an asset of `kind` makes: what the asset can have and the API does not
/// hold yet under `derived/`.
async fn wanted(data_dir: &std::path::Path, sha256: &str, kind: &str) -> Make {
    let missing = |suffix: &str| {
        let path = data_dir.join(files::derived(sha256, suffix));
        async move { !tokio::fs::try_exists(path).await.unwrap_or(false) }
    };
    let (video, sound) = (kind == "video", kind == "video" || kind == "audio");
    Make {
        poster: video && missing("poster.jpg").await,
        strip: video && missing("strip.jpg").await,
        peaks: sound && missing("peaks.json").await,
        proxy: video && missing("proxy.mp4").await,
    }
}

async fn kind_of(state: &AppState, job: &Job) -> Option<(String, String)> {
    sqlx::query_as("SELECT kind, state FROM assets WHERE project_id = $1 AND name = $2 AND sha256 = $3")
        .bind(job.project)
        .bind(&job.name)
        .bind(&job.sha256)
        .fetch_optional(&state.pool)
        .await
        .ok()
        .flatten()
}

async fn settle(state: &AppState, job: &Job, facts: &Result<Facts, Fault>) -> Result<u64, sqlx::Error> {
    let query = "UPDATE assets SET state = $4, error = $5, kind = coalesce($6, kind), duration_ms = $7, \
                 width = $8, height = $9, fps = $10, has_audio = $11, poster = $12 \
                 WHERE project_id = $1 AND name = $2 AND sha256 = $3";
    let bound = sqlx::query(query).bind(job.project).bind(&job.name).bind(&job.sha256);
    let none = Probe::default();
    let (probe, bound) = match facts {
        Ok(facts) => (&facts.probe, bound.bind("ready").bind(None::<String>).bind(Some(facts.kind))),
        Err(fault) => (&none, bound.bind("failed").bind(Some(fault.reason.clone())).bind(None::<&str>)),
    };
    let poster = facts.as_ref().ok().and_then(|facts| facts.poster.clone());
    let bound = bound.bind(probe.duration_ms).bind(probe.width).bind(probe.height).bind(probe.fps);
    Ok(bound.bind(probe.has_audio).bind(poster).execute(&state.pool).await?.rows_affected())
}

async fn proxy(state: &AppState, job: &Job, media: &MediaJob) {
    if let Seen::Lost(reason) = media.until(&state.data_dir, Step::Proxy).await {
        return tracing::warn!(asset = %job.name, reason, "assets: no proxy");
    }
    let Some(relative) = media.kept(&state.data_dir, made::PROXY, files::proxy(&job.sha256), state.limits.asset).await
    else {
        return tracing::warn!(asset = %job.name, "assets: no proxy");
    };
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

async fn decoded(state: &AppState, job: &Job, (kind, was): (&str, &str), media: &MediaJob) {
    let facts = inspect(state, job, kind, media).await;
    if facts.as_ref().is_err_and(|fault| fault.lost && was != "probing") {
        return;
    }
    match settle(state, job, &facts).await {
        Ok(0) => return,
        Ok(_) => state.events.asset(job.project, &job.name, if facts.is_ok() { "ready" } else { "failed" }),
        Err(error) => return tracing::error!(asset = %job.name, %error, "assets: could not store a probe"),
    }
    let Ok(facts) = facts else { return };
    super::previews::make(state, job, &facts, media).await;
    if facts.kind == "video" {
        proxy(state, job, media).await;
    }
}

/// Looks at one asset, one media job at a time across the process:
/// hands it to the render worker, which probes it and makes what it lacks, then stores the probe
/// and the poster of a video and tells open editors, takes the filmstrip and the waveform and
/// tells them again, then takes the proxy of a video and tells them once more. The API decodes
/// nothing. An asset replaced or deleted meanwhile is left alone.
pub async fn run(state: AppState, job: Job) {
    let Ok(_slot) = state.media.clone().acquire_owned().await else { return };
    let Some((kind, was)) = kind_of(&state, &job).await else { return };
    let make = wanted(&state.data_dir, &job.sha256, &kind).await;
    match MediaJob::publish(&state.data_dir, &job.sha256, make).await {
        Ok(media) => {
            decoded(&state, &job, (&kind, &was), &media).await;
            media.close(&state.data_dir).await;
        }
        Err(error) => {
            tracing::error!(asset = %job.name, %error, "assets: could not queue a media job");
            let fault = Fault { reason: "the file could not be handed to the media worker".to_owned(), lost: true };
            if was == "probing" && settle(&state, &job, &Err(fault)).await.is_ok_and(|rows| rows > 0) {
                state.events.asset(job.project, &job.name, "failed");
            }
        }
    }
}

/// Starts [`run`] for `job` on its own task.
pub fn spawn(state: AppState, job: Job) {
    tokio::spawn(run(state, job));
}

/// Starts again, at boot, every asset a previous process left `probing`, or `ready` without its
/// proxy, its filmstrip or its waveform.
pub async fn resume(state: &AppState) {
    super::leftovers::clear(&state.data_dir).await;
    let data_dir = state.data_dir.to_path_buf();
    match tokio::task::spawn_blocking(move || super::seal::seal(&data_dir)).await {
        Ok(Ok(0)) => {}
        Ok(Ok(sealed)) => tracing::info!(sealed, "assets: made stored files read-only"),
        Ok(Err(error)) => tracing::error!(%error, "assets: could not make stored files read-only"),
        Err(error) => tracing::error!(%error, "assets: the read-only pass did not finish"),
    }
    let left = sqlx::query_as::<_, Job>(
        "SELECT project_id, name, sha256 FROM assets \
         WHERE state = 'probing' OR (state = 'ready' AND kind = 'video' AND (proxy IS NULL OR strip IS NULL)) \
         OR (state = 'ready' AND peaks IS NULL AND (kind = 'audio' OR (kind = 'video' AND has_audio))) \
         ORDER BY created_at",
    )
    .fetch_all(&state.pool)
    .await;
    match left {
        Ok(jobs) => jobs.into_iter().for_each(|job| spawn(state.clone(), job)),
        Err(error) => tracing::error!(%error, "assets: could not list unfinished assets"),
    }
}
