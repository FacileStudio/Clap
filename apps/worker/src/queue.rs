//! The queue is the renders directory: one directory per job, queued while it has a `job.json`
//! and no `status.json`.

use std::path::{Path, PathBuf};

use time::OffsetDateTime;

use crate::config::Config;
use crate::files;
use crate::spec::{Head, Kind, named};
use crate::status::{State, Status};

/// What a job that was `running` when the worker started is told.
const STOPPED: &str = "the render worker stopped during this render; start it again";

fn job_dirs(renders: &Path) -> Vec<PathBuf> {
    let entries = std::fs::read_dir(renders).into_iter().flatten().flatten();
    let named = entries.filter(|entry| entry.file_name().to_str().is_some_and(named));
    named
        .map(|entry| entry.path())
        .filter(|path| files::is_dir(path) && files::exists(&path.join("job.json")))
        .collect()
}

/// Removes what a render that never finished left behind: its partial MP4 and its extracted
/// frames.
pub(crate) fn sweep(dir: &Path) {
    files::clear(&dir.join("out.mp4.tmp"));
    let bundle = dir.join("bundle");
    let entries = std::fs::read_dir(&bundle).into_iter().flatten().flatten().filter(|_| files::is_dir(&bundle));
    for entry in entries.filter(|entry| entry.file_name().to_string_lossy().starts_with(".clap-frames-")) {
        files::clear(&entry.path());
    }
}

/// Marks every job left `running`, or with a status that cannot be read, as failed. The worker
/// renders one job at a time, so between two jobs a `running` status is always a leftover.
pub(crate) fn recover(config: &Config) -> usize {
    let mut recovered = 0;
    for dir in job_dirs(&config.renders) {
        let stale = match Status::load(&dir) {
            Some(Ok(status)) => status.state == State::Running,
            Some(Err(_)) => true,
            None => false,
        };
        if stale {
            sweep(&dir);
            recovered += usize::from(Status::failed(STOPPED).save(&dir).is_ok());
        }
    }
    recovered
}

/// The queued job to run now: a media job before any render, then the one queued first. A job
/// whose `job.json` cannot be read sorts first, so it fails at once instead of waiting.
pub(crate) fn next(config: &Config) -> Option<PathBuf> {
    let queued = job_dirs(&config.renders).into_iter().filter(|dir| Status::load(dir).is_none());
    let dated = queued.map(|dir| match Head::load(&dir) {
        Ok((head, _)) => (head.kind, crate::clock::parse(&head.created_at), dir),
        Err(_) => (Kind::Media, Some(OffsetDateTime::UNIX_EPOCH), dir),
    });
    let dated = dated.filter(|(kind, _, _)| config.only.is_none_or(|only| only == *kind));
    dated.min().map(|(_, _, dir)| dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Limits;

    fn job(renders: &Path, id: &str, created: &str) -> PathBuf {
        let dir = renders.join(id);
        std::fs::create_dir_all(dir.join("bundle/.clap-frames-x")).unwrap();
        let json =
            format!(r#"{{"id":"{id}","width":320,"height":180,"fps":30,"range":null,"created_at":"{created}"}}"#);
        std::fs::write(dir.join("job.json"), json).unwrap();
        dir
    }

    #[test]
    fn the_oldest_queued_job_is_next_and_a_job_left_running_fails() {
        let renders = tempfile::tempdir().unwrap();
        let config = Config {
            renders: renders.path().to_owned(),
            poll: Default::default(),
            limits: Limits::default(),
            only: None,
        };
        assert_eq!(next(&config), None);
        let late = job(renders.path(), "late", "2026-11-16T10:00:05Z");
        let early = job(renders.path(), "early", "2026-11-16T12:00:00+02:00");
        let crashed = job(renders.path(), "crashed", "2026-11-16T09:00:00Z");
        let done = job(renders.path(), "done", "2026-11-16T08:00:00Z");
        std::fs::create_dir(renders.path().join(".hidden")).unwrap();
        std::os::unix::fs::symlink(&late, renders.path().join("linked")).unwrap();
        Status::new(State::Running, 3, 60).save(&crashed).unwrap();
        Status::new(State::Done, 60, 60).save(&done).unwrap();
        std::fs::write(crashed.join("out.mp4.tmp"), "half").unwrap();
        assert_eq!(recover(&config), 1);
        let status = Status::load(&crashed).unwrap().unwrap();
        assert_eq!((status.state, status.error.as_deref()), (State::Failed, Some(STOPPED)));
        assert!(!crashed.join("out.mp4.tmp").exists() && !crashed.join("bundle/.clap-frames-x").exists());
        assert_eq!(Status::load(&done).unwrap().unwrap().state, State::Done);
        assert_eq!(next(&config), Some(early.clone()));
        Status::new(State::Done, 1, 1).save(&early).unwrap();
        assert_eq!(next(&config), Some(late));
    }

    #[test]
    fn a_media_job_goes_before_an_older_render() {
        let renders = tempfile::tempdir().unwrap();
        let config = Config {
            renders: renders.path().to_owned(),
            poll: Default::default(),
            limits: Limits::default(),
            only: None,
        };
        let render = job(renders.path(), "render", "2026-11-16T10:00:00Z");
        let upload = renders.path().join("upload");
        std::fs::create_dir_all(&upload).unwrap();
        let media = r#"{"id":"upload","kind":"media","created_at":"2026-11-16T23:00:00Z"}"#;
        std::fs::write(upload.join("job.json"), media).unwrap();
        assert_eq!(next(&config), Some(upload.clone()));
        let renders_only = Config { only: Some(Kind::Render), ..config.clone() };
        assert_eq!(next(&renders_only), Some(render.clone()));
        Status::new(State::Done, 5, 5).save(&upload).unwrap();
        assert_eq!(next(&config), Some(render));
        assert_eq!(next(&Config { only: Some(Kind::Media), ..config }), None);
    }
}
