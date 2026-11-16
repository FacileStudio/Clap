//! The API's half of a media job: it hands an upload to the render worker and reads back what
//! the worker made. The job directory sits in `DATA_DIR/renders`, which a compromised worker
//! controls, so every file comes back through [`jobfs`]: no link, one hard link, a size cap.
//!
//! ```text
//! DATA_DIR/renders/media-<uuid>/bundle/input   a copy of the upload, written by the API
//! DATA_DIR/renders/media-<uuid>/job.json       {"kind": "media", "poster": true, ...}, last
//! DATA_DIR/renders/media-<uuid>/status.json    `frame` counts the steps finished
//! DATA_DIR/renders/media-<uuid>/probe.json, poster.jpg, strip.jpg, peaks.json, proxy.mp4
//! ```

use std::path::Path;
use std::time::{Duration, Instant};

use clap_media::{Make, Step};
use serde::Serialize;
use time::format_description::well_known::Rfc3339;

use super::files;
use crate::modules::renders::{bundle, jobfs, stage, status};

/// How often the API looks at a media job.
pub const POLL: Duration = Duration::from_millis(250);

/// How long a running media job may go without a status write: the worker writes one twice a
/// second.
pub const SILENT: Duration = Duration::from_secs(120);

/// How long a media job waits for a worker whose heartbeat is stale before it is given up.
pub const UNSERVED: Duration = Duration::from_secs(10 * 60);

/// The prefix of a media job's directory. A render's directory is a bare UUID, so the render
/// sweep never takes a media job for a render nobody names.
pub const PREFIX: &str = "media-";

#[derive(Serialize)]
struct JobFile<'a> {
    id: &'a str,
    kind: &'static str,
    created_at: String,
    #[serde(flatten)]
    make: Make,
}

/// Where a media job stands for the step the caller waits for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Seen {
    /// The step is over: its file is whole, or will never come.
    Reached,
    /// The job ended before the step, with the worker's reason.
    Ended(Option<String>),
    /// The worker stopped answering, or the job directory went: nothing more will come.
    Lost(&'static str),
}

/// One media job, named by its directory under `DATA_DIR/renders`.
#[derive(Debug)]
pub struct MediaJob {
    pub name: String,
}

fn seen(found: Option<status::Status>, step: Step) -> Option<Seen> {
    let found = found?;
    match found.state.as_str() {
        "done" => Some(Seen::Reached),
        _ if found.frame >= step as i64 => Some(Seen::Reached),
        "running" if found.silence().is_none_or(|seconds| seconds > SILENT.as_secs() as i64) => {
            Some(Seen::Lost("the media worker stopped reporting progress"))
        }
        "running" => None,
        _ => Some(Seen::Ended(found.error)),
    }
}

impl MediaJob {
    /// Queues a media job on the upload `sha256`: copies it into a bundle built out of the
    /// worker's sight, then publishes the bundle with a `job.json` asking for `make`.
    pub async fn publish(data_dir: &Path, sha256: &str, make: Make) -> std::io::Result<MediaJob> {
        let name = format!("{PREFIX}{}", uuid::Uuid::new_v4());
        let staged = stage::staging(data_dir).join(&name);
        let bundle = staged.join("bundle");
        let created_at = time::OffsetDateTime::now_utc().format(&Rfc3339).map_err(std::io::Error::other)?;
        let job = JobFile { id: &name, kind: "media", created_at, make };
        let built = async {
            tokio::fs::create_dir_all(&bundle).await?;
            tokio::fs::copy(files::path(data_dir, sha256), bundle.join("input")).await?;
            stage::publish(data_dir, &bundle, &name, &job).await
        };
        let done = built.await;
        tokio::fs::remove_dir_all(&staged).await.ok();
        done.map(|()| MediaJob { name })
    }

    /// Waits until the worker is past `step`, the job ends, or the worker is lost: its status
    /// goes silent, the job directory goes, or no live worker takes the job for [`UNSERVED`].
    pub async fn until(&self, data_dir: &Path, step: Step) -> Seen {
        let mut served = Instant::now();
        loop {
            if let Some(seen) = seen(status::status(data_dir, &self.name).await, step) {
                return seen;
            }
            if !tokio::fs::try_exists(bundle::root(data_dir).join(&self.name).join("job.json")).await.unwrap_or(false) {
                return Seen::Lost("the media job was removed");
            }
            if status::worker_alive(data_dir).await {
                served = Instant::now();
            } else if served.elapsed() > UNSERVED {
                return Seen::Lost("the render service is not running: upload the file again later");
            }
            tokio::time::sleep(POLL).await;
        }
    }

    /// The bytes of the small file `file` of the job, or `None` when it is missing or refused.
    pub async fn read(&self, data_dir: &Path, file: &str, max: u64) -> Option<Vec<u8>> {
        let (data_dir, relative) = (data_dir.to_owned(), format!("{}/{file}", self.name));
        tokio::task::spawn_blocking(move || jobfs::read(&data_dir, &relative, max)).await.ok()?
    }

    /// The derived file at `relative` (to `DATA_DIR`): kept when it is already there, else
    /// copied from the job's `file` through [`jobfs::copy_out`]. `None` when there is neither.
    pub async fn kept(&self, data_dir: &Path, file: &str, relative: String, max: u64) -> Option<String> {
        let target = data_dir.join(&relative);
        if tokio::fs::try_exists(&target).await.unwrap_or(false) {
            return Some(relative);
        }
        tokio::fs::create_dir_all(target.parent()?).await.ok()?;
        tokio::fs::remove_file(target.with_extension("part")).await.ok();
        let (data_dir, source) = (data_dir.to_owned(), format!("{}/{file}", self.name));
        let copied = tokio::task::spawn_blocking(move || jobfs::copy_out(&data_dir, &source, &target, max)).await;
        match copied.map_err(std::io::Error::other).and_then(|copied| copied) {
            Ok(_) => Some(relative),
            Err(error) => {
                tracing::debug!(job = %self.name, file, %error, "assets: nothing taken from the media job");
                None
            }
        }
    }

    /// Removes the job directory: the worker never deletes one.
    pub async fn close(self, data_dir: &Path) {
        tokio::fs::remove_dir_all(bundle::root(data_dir).join(&self.name)).await.ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(state: &str, frame: i64, written: &str) -> Option<status::Status> {
        let json =
            serde_json::json!({"state": state, "frame": frame, "frames": 5, "error": "why", "updated_at": written});
        serde_json::from_value(json).ok()
    }

    #[test]
    fn a_step_is_reached_by_the_count_or_the_end_and_a_silent_worker_is_lost() {
        let now = time::OffsetDateTime::now_utc().format(&Rfc3339).unwrap();
        assert_eq!(seen(None, Step::Probe), None);
        assert_eq!(seen(at("running", 1, &now), Step::Poster), None);
        assert_eq!(seen(at("running", 2, &now), Step::Poster), Some(Seen::Reached));
        assert_eq!(seen(at("done", 0, &now), Step::Proxy), Some(Seen::Reached));
        assert_eq!(seen(at("failed", 4, &now), Step::Peaks), Some(Seen::Reached));
        assert_eq!(seen(at("failed", 0, &now), Step::Poster), Some(Seen::Ended(Some("why".to_owned()))));
        let old = seen(at("running", 1, "2026-01-01T00:00:00Z"), Step::Poster);
        assert!(matches!(old, Some(Seen::Lost(_))), "{old:?}");
        assert_eq!(files::derived("ab".repeat(32).as_str(), "poster.jpg"), files::poster(&"ab".repeat(32)));
    }
}
