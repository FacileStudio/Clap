//! What a media job's probe says an asset is, read back from the worker as untrusted.

use clap_media::{Probe, Step, files as made, max};

use super::files;
use super::media::{MediaJob, Seen};
use super::worker::Job;
use crate::state::AppState;

/// What the probe found, and the poster of a video.
pub struct Facts {
    pub probe: Probe,
    pub kind: &'static str,
    pub poster: Option<String>,
}

/// Why an asset has no facts: what to tell its owner, and whether the worker was lost, in which
/// case an asset that was already `ready` is left as it was.
pub struct Fault {
    pub reason: String,
    pub lost: bool,
}

impl From<String> for Fault {
    fn from(reason: String) -> Self {
        Fault { reason, lost: false }
    }
}

async fn probed(state: &AppState, media: &MediaJob) -> Result<Result<Probe, String>, Fault> {
    match media.until(&state.data_dir, Step::Poster).await {
        Seen::Lost(reason) => Err(Fault { reason: reason.to_owned(), lost: true }),
        Seen::Ended(error) => Ok(Err(error.unwrap_or_else(|| "the media worker wrote no probe".to_owned()))),
        Seen::Reached => {
            let report = media.read(&state.data_dir, made::PROBE, max::PROBE).await;
            Ok(report
                .ok_or_else(|| "the media worker wrote no usable probe".to_owned())
                .and_then(|r| clap_media::parse(&r)))
        }
    }
}

async fn poster(state: &AppState, job: &Job, media: &MediaJob) -> Option<String> {
    let kept = media.kept(&state.data_dir, made::POSTER, files::poster(&job.sha256), max::POSTER).await;
    if kept.is_none() {
        tracing::warn!(asset = %job.name, "assets: no poster");
    }
    kept
}

/// Waits for the probe and the poster of the media job on `job`, then answers what the asset of
/// `kind` turns out to be: a video with no picture is a sound, a sound has no size, a picture
/// `ffprobe` cannot read is still a picture.
pub async fn inspect(state: &AppState, job: &Job, kind: &str, media: &MediaJob) -> Result<Facts, Fault> {
    let probe = match (kind, probed(state, media).await?) {
        (_, Ok(probe)) => probe,
        ("image", Err(_)) => return Ok(Facts { probe: Probe::default(), kind: "image", poster: None }),
        (_, Err(error)) => return Err(error.into()),
    };
    match (kind, probe.has_video) {
        ("video", true) => Ok(Facts { poster: poster(state, job, media).await, probe, kind: "video" }),
        ("video", false) if probe.has_audio => Ok(Facts { probe, kind: "audio", poster: None }),
        ("video", false) => Err("the file has no picture and no sound".to_owned().into()),
        ("audio", _) if !probe.has_audio => Err("the file has no sound".to_owned().into()),
        ("audio", _) => {
            Ok(Facts { probe: Probe { width: None, height: None, fps: None, ..probe }, kind: "audio", poster: None })
        }
        _ => Ok(Facts { probe: Probe { duration_ms: None, fps: None, ..probe }, kind: "image", poster: None }),
    }
}
