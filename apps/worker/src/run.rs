//! One job, start to final status.

use std::ops::ControlFlow;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, Ordering};
use std::time::{Duration, Instant};

use clap_render::{Error, Job, Progress, Tools, render_with};

use crate::config::Limits;
use crate::spec::Spec;
use crate::status::{State, Status};
use crate::{files, health, queue};

/// How often the status file is rewritten while a job runs.
const BEAT: Duration = Duration::from_millis(500);

const CANCELLED: u8 = 1;
const TIMED_OUT: u8 = 2;
const TOO_LONG: u8 = 3;

/// What the render thread and the status thread share.
#[derive(Default)]
struct Shared {
    frame: AtomicU32,
    frames: AtomicU32,
    /// Why the render must stop: 0 while it may go on.
    stop: AtomicU8,
    over: AtomicBool,
}

impl Shared {
    fn status(&self, state: State) -> Status {
        Status::new(state, self.frame.load(Ordering::Relaxed), self.frames.load(Ordering::Relaxed))
    }

    /// Rewrites the status twice a second until the render is over, and watches for the `cancel`
    /// file and the wall clock meanwhile.
    fn beat(&self, dir: &Path, wall: Duration) {
        let deadline = Instant::now() + wall;
        while !self.over.load(Ordering::Relaxed) {
            let _ = self.status(State::Running).save(dir);
            health::beat(dir.parent().unwrap_or(dir));
            let stop = if files::exists(&dir.join("cancel")) {
                CANCELLED
            } else {
                u8::from(Instant::now() > deadline) * TIMED_OUT
            };
            let _ = self.stop.compare_exchange(0, stop, Ordering::Relaxed, Ordering::Relaxed);
            std::thread::sleep(BEAT);
        }
    }

    fn progress(&self, progress: Progress, most: u32) -> ControlFlow<()> {
        self.frame.store(progress.frame, Ordering::Relaxed);
        self.frames.store(progress.frames, Ordering::Relaxed);
        if progress.frames > most {
            self.stop.store(TOO_LONG, Ordering::Relaxed);
        }
        if self.stop.load(Ordering::Relaxed) == 0 { ControlFlow::Continue(()) } else { ControlFlow::Break(()) }
    }

    fn verdict(&self, rendered: Result<(), Error>, limits: &Limits) -> Status {
        let failed = |reason: String| Status { error: Some(reason), ..self.status(State::Failed) };
        match (rendered, self.stop.load(Ordering::Relaxed)) {
            (Ok(()), _) => self.status(State::Done),
            (Err(Error::Cancelled), CANCELLED) => self.status(State::Cancelled),
            (Err(Error::Cancelled), TIMED_OUT) => {
                failed(format!("the render passed its {} s time limit", limits.wall.as_secs()))
            }
            (Err(Error::Cancelled), TOO_LONG) => {
                failed(format!("the scene is longer than the {} s a render may last", limits.seconds))
            }
            (Err(error), _) => failed(error.to_string()),
        }
    }
}

fn render(dir: &Path, spec: &Spec, tools: &Tools, limits: &Limits) -> Status {
    let partial = dir.join("out.mp4.tmp");
    let job = Job {
        width: spec.width,
        height: spec.height,
        fps: spec.fps,
        range: spec.range.map(|(start, end)| start..end),
        frame_budget: limits.frame_bytes,
        ..Job::new(dir.join("bundle"), &partial)
    };
    let shared = Shared::default();
    let most = (limits.seconds * f64::from(spec.fps)).round() as u32;
    let rendered = std::thread::scope(|scope| {
        scope.spawn(|| shared.beat(dir, limits.wall));
        let rendered = render_with(&job, tools, &mut |progress| shared.progress(progress, most));
        shared.over.store(true, Ordering::Relaxed);
        rendered
    });
    let moved = rendered.map_err(Some).and_then(|_| std::fs::rename(&partial, dir.join("out.mp4")).map_err(|_| None));
    match moved {
        Ok(()) => shared.verdict(Ok(()), limits),
        Err(Some(error)) => shared.verdict(Err(error), limits),
        Err(None) => Status::failed("the finished video could not be moved into place"),
    }
}

/// Runs the job in `dir` and writes its final status. Nothing outside `dir` is written.
pub(crate) fn run(dir: &Path, tools: &Tools, limits: &Limits) -> Status {
    let status = match Spec::load(dir, limits) {
        Err(reason) => Status::failed(reason),
        Ok(_) if files::exists(&dir.join("cancel")) => Status::new(State::Cancelled, 0, 0),
        Ok(_) if !files::is_dir(&dir.join("bundle")) => Status::failed("the job has no bundle directory"),
        Ok(spec) => {
            queue::sweep(dir);
            files::clear(&dir.join("out.mp4"));
            let _ = Status::new(State::Running, 0, 0).save(dir);
            render(dir, &spec, tools, limits)
        }
    };
    queue::sweep(dir);
    let _ = status.save(dir);
    status
}

#[cfg(test)]
mod tests {
    use super::*;

    const SKIP: &str = "\n================ SKIPPED: the worker's render test needs chrome-headless-shell and ffmpeg \
        (CLAP_CHROME, CLAP_FFMPEG or PATH) ================\n";

    fn job(renders: &Path, id: &str, range: &str) -> std::path::PathBuf {
        let dir = renders.join(id);
        std::fs::create_dir_all(dir.join("bundle")).unwrap();
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../crates/render/fixtures/title/index.html");
        std::fs::copy(fixture, dir.join("bundle/index.html")).unwrap();
        let json = format!(
            r#"{{"id":"{id}","width":320,"height":180,"fps":30,"range":{range},"created_at":"2026-11-16T10:00:00Z"}}"#
        );
        std::fs::write(dir.join("job.json"), json).unwrap();
        dir
    }

    #[test]
    fn a_job_ends_with_a_video_or_a_reason() {
        let Ok(tools) = Tools::locate() else { return eprintln!("{SKIP}") };
        let renders = tempfile::tempdir().unwrap();
        let limits = Limits::default();
        let good = job(renders.path(), "good", "[0,0.2]");
        let done = run(&good, &tools, &limits);
        assert_eq!((done.state, done.frame, done.frames, done.error.clone()), (State::Done, 6, 6, None));
        assert_eq!(Status::load(&good).unwrap().unwrap(), done);
        assert!(good.join("out.mp4").metadata().unwrap().len() > 1000 && !good.join("out.mp4.tmp").exists());
        assert!(health::alive(renders.path()));
        let stopped = job(renders.path(), "stopped", "null");
        std::fs::write(stopped.join("cancel"), "").unwrap();
        assert_eq!(run(&stopped, &tools, &limits).state, State::Cancelled);
        let long = run(&job(renders.path(), "long", "null"), &tools, &Limits { seconds: 1.0, ..limits.clone() });
        assert!(long.error.as_deref().is_some_and(|error| error.contains("longer than the 1 s")), "{long:?}");
        let slow =
            run(&job(renders.path(), "slow", "null"), &tools, &Limits { wall: Duration::ZERO, ..limits.clone() });
        assert!(slow.error.as_deref().is_some_and(|error| error.contains("time limit")), "{slow:?}");
        let empty = job(renders.path(), "empty", "null");
        std::fs::remove_dir_all(empty.join("bundle")).unwrap();
        assert_eq!(run(&empty, &tools, &limits).error.as_deref(), Some("the job has no bundle directory"));
        let files: Vec<_> =
            ["long", "slow", "stopped"].iter().map(|id| renders.path().join(id).join("out.mp4")).collect();
        assert!(files.iter().all(|file| !file.exists()), "a job that did not finish left a video");
    }
}
