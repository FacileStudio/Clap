//! What the timeline draws of an asset: the filmstrip of a video and the waveform of a sound,
//! made after the probe and before the proxy, inside the same media slot.

use super::probe::Probe;
use super::strip::{self, Plan};
use super::worker::Job;
use super::{files, peaks, tools};
use crate::state::AppState;

async fn filmstrip(state: &AppState, job: &Job, duration_ms: Option<i64>) -> Option<(String, Plan)> {
    let plan = strip::plan(duration_ms)?;
    let relative = files::derived(&job.sha256, "strip.jpg");
    let input = files::path(&state.data_dir, &job.sha256);
    match strip::strip(&state.tools, &input, plan, duration_ms, &state.data_dir.join(&relative)).await {
        Ok(()) => Some((relative, plan)),
        Err(error) => {
            tracing::warn!(asset = %job.name, %error, "assets: no filmstrip");
            None
        }
    }
}

async fn waveform(state: &AppState, job: &Job, duration_ms: Option<i64>) -> Option<String> {
    let relative = files::derived(&job.sha256, "peaks.json");
    let input = files::path(&state.data_dir, &job.sha256);
    let limit = tools::proxy_limit(duration_ms);
    match peaks::peaks(&state.tools, &input, &state.data_dir.join(&relative), limit).await {
        Ok(()) => Some(relative),
        Err(error) => {
            tracing::warn!(asset = %job.name, %error, "assets: no waveform");
            None
        }
    }
}

/// Makes the filmstrip of a `video` and the waveform of anything `probe` heard sound in, stores
/// them on the row the job still names, and tells open editors. A part that fails is logged and
/// left empty; the asset stays `ready`.
pub async fn make(state: &AppState, job: &Job, kind: &str, probe: &Probe) {
    let strip = if kind == "video" { filmstrip(state, job, probe.duration_ms).await } else { None };
    let sound = kind == "audio" || (kind == "video" && probe.has_audio);
    let peaks = if sound { waveform(state, job, probe.duration_ms).await } else { None };
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
