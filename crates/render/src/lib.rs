//! Renders a seekable HTML scene to an H.264 MP4.
//!
//! A scene is a directory holding an `index.html` that exposes
//! `window.__scene = { duration, seek(t) }`. The renderer loads it in `chrome-headless-shell`
//! over the DevTools Protocol with every request outside the directory refused, seeks to each
//! frame time, captures the frame and pipes it to `ffmpeg`. The page never plays: a `<video>` is
//! shown as frames `ffmpeg` extracted beforehand, and sound is mixed by `ffmpeg` from the files.
//! See `docs/rendering.md`.
#![deny(unused, dead_code, unsafe_code)]

mod audio;
mod capture;
mod cdp;
mod chrome;
mod encode;
mod error;
mod guard;
mod job;
mod media;
mod page;
mod pipe;
mod proc;
mod stock;
pub mod tools;

use std::ops::ControlFlow;
use std::time::Instant;

pub use error::Error;
pub use job::{Capture, Job, Progress, Report};
pub use tools::Tools;

use cdp::Browser;

/// What a caller answers to each [`Progress`]: `Break` cancels the render.
pub type OnProgress<'a> = &'a mut dyn FnMut(Progress) -> ControlFlow<()>;

/// Renders `job` to its output file. Blocks until the MP4 is complete.
pub fn render(job: &Job, tools: &Tools) -> Result<Report, Error> {
    render_with(job, tools, &mut |_| ControlFlow::Continue(()))
}

/// [`render`], telling `progress` after every frame. A cancelled or failed render leaves no
/// output file and no Chrome or ffmpeg process.
pub fn render_with(job: &Job, tools: &Tools, progress: OnProgress) -> Result<Report, Error> {
    let started = Instant::now();
    job.check()?;
    let guard = guard::Guard::new(&job.dir)?;
    let mut browser = Browser::launch(tools, chrome::flags(job, tools), guard)?;
    let filmed = capture::film(&mut browser, job, tools, progress);
    let blocked = browser.guard.refused().to_vec();
    drop(browser);
    let film = filmed.inspect_err(|_| _ = std::fs::remove_file(&job.output))?;
    let frames = film.hashes.len() as u32;
    Ok(Report {
        frames,
        duration: f64::from(frames) / f64::from(job.fps),
        frame_hashes: film.hashes,
        blocked,
        capture: job.capture,
        videos: film.videos,
        audio_tracks: film.audio_tracks,
        elapsed: started.elapsed(),
    })
}
