//! What a job's work and its status thread share: the progress, and why the work must stop.

use std::ops::ControlFlow;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, Ordering};
use std::time::{Duration, Instant};

use clap_render::{Error, Progress};

use crate::config::Limits;
use crate::status::{State, Status};
use crate::{files, health};

/// How often the status file is rewritten while a job runs.
const BEAT: Duration = Duration::from_millis(500);

pub(crate) const CANCELLED: u8 = 1;
pub(crate) const TIMED_OUT: u8 = 2;
pub(crate) const TOO_LONG: u8 = 3;

/// What the render thread and the status thread share.
#[derive(Default)]
pub(crate) struct Shared {
    pub frame: AtomicU32,
    pub frames: AtomicU32,
    /// Why the render must stop: 0 while it may go on.
    pub stop: AtomicU8,
    pub over: AtomicBool,
}

impl Shared {
    pub(crate) fn status(&self, state: State) -> Status {
        Status::new(state, self.frame.load(Ordering::Relaxed), self.frames.load(Ordering::Relaxed))
    }

    /// Rewrites the status twice a second until the render is over, and watches for the `cancel`
    /// file and the wall clock meanwhile.
    pub(crate) fn beat(&self, dir: &Path, wall: Duration) {
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

    pub(crate) fn progress(&self, progress: Progress, most: u32) -> ControlFlow<()> {
        self.frame.store(progress.frame, Ordering::Relaxed);
        self.frames.store(progress.frames, Ordering::Relaxed);
        if progress.frames > most {
            self.stop.store(TOO_LONG, Ordering::Relaxed);
        }
        if self.stop.load(Ordering::Relaxed) == 0 { ControlFlow::Continue(()) } else { ControlFlow::Break(()) }
    }

    pub(crate) fn verdict(&self, rendered: Result<(), Error>, limits: &Limits) -> Status {
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
