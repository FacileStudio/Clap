//! What the timeline draws of an asset: the filmstrip of a video and the waveform of a sound,
//! taken from the media job after the poster and before the proxy.

use clap_media::{Plan, Step, files as made, max};

use super::facts::Facts;
use super::files;
use super::media::{MediaJob, Seen};
use super::worker::Job;
use crate::state::AppState;

async fn filmstrip(state: &AppState, job: &Job, media: &MediaJob, duration_ms: Option<i64>) -> Option<(String, Plan)> {
    let plan = clap_media::plan(duration_ms)?;
    let relative = files::derived(&job.sha256, "strip.jpg");
    let kept = media.kept(&state.data_dir, made::STRIP, relative, max::STRIP).await;
    if kept.is_none() {
        tracing::warn!(asset = %job.name, "assets: no filmstrip");
    }
    Some((kept?, plan))
}

async fn waveform(state: &AppState, job: &Job, media: &MediaJob) -> Option<String> {
    let relative = files::derived(&job.sha256, "peaks.json");
    let target = state.data_dir.join(&relative);
    if tokio::fs::try_exists(&target).await.unwrap_or(false) {
        return Some(relative);
    }
    let read = media.read(&state.data_dir, made::PEAKS, max::PEAKS).await.ok_or("the media worker wrote no waveform");
    let checked = read.map_err(str::to_owned).and_then(|json| clap_media::checked(&json));
    let written = match checked {
        Ok(json) => written(&target, &json).await,
        Err(error) => Err(error),
    };
    match written {
        Ok(()) => Some(relative),
        Err(error) => {
            tracing::warn!(asset = %job.name, %error, "assets: no waveform");
            None
        }
    }
}

async fn written(target: &std::path::Path, json: &[u8]) -> Result<(), String> {
    let scratch = target.with_extension("part");
    tokio::fs::create_dir_all(target.parent().unwrap_or(target)).await.map_err(|error| error.to_string())?;
    let moved = match tokio::fs::write(&scratch, json).await {
        Ok(()) => tokio::fs::rename(&scratch, target).await.map_err(|error| error.to_string()),
        Err(error) => Err(error.to_string()),
    };
    if moved.is_err() {
        tokio::fs::remove_file(&scratch).await.ok();
    }
    moved
}

/// Takes the filmstrip of a video and the waveform of anything the probe heard sound in from
/// the media job once it is past them, stores them on the row the job still names, and tells
/// open editors. A part that is missing is logged and left empty; the asset stays `ready`.
pub async fn make(state: &AppState, job: &Job, facts: &Facts, media: &MediaJob) {
    if let Seen::Lost(reason) = media.until(&state.data_dir, Step::Peaks).await {
        return tracing::warn!(asset = %job.name, reason, "assets: no filmstrip or waveform");
    }
    let strip = if facts.kind == "video" { filmstrip(state, job, media, facts.probe.duration_ms).await } else { None };
    let sound = facts.kind == "audio" || (facts.kind == "video" && facts.probe.has_audio);
    let peaks = if sound { waveform(state, job, media).await } else { None };
    if strip.is_none() && peaks.is_none() {
        return;
    }
    let query = "UPDATE assets SET strip = coalesce($4, strip), strip_tiles = coalesce($5, strip_tiles), \
                 strip_step = coalesce($6, strip_step), peaks = coalesce($7, peaks) \
                 WHERE project_id = $1 AND name = $2 AND sha256 = $3";
    let (path, plan) = strip.map_or((None, None), |(path, plan)| (Some(path), Some(plan)));
    let stored = sqlx::query(query)
        .bind(job.project)
        .bind(&job.name)
        .bind(&job.sha256)
        .bind(path)
        .bind(plan.map(|plan| plan.tiles as i32))
        .bind(plan.map(Plan::step))
        .bind(peaks)
        .execute(&state.pool)
        .await;
    match stored {
        Ok(done) if done.rows_affected() > 0 => state.events.asset(job.project, &job.name, "ready"),
        Ok(_) => {}
        Err(error) => tracing::error!(asset = %job.name, %error, "assets: could not store a filmstrip or a waveform"),
    }
}
