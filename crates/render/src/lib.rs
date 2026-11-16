//! Renders a seekable HTML scene to an H.264 MP4.
//!
//! A scene is a directory holding an `index.html` that exposes
//! `window.__scene = { duration, seek(t) }`. The renderer loads it in `chrome-headless-shell`
//! over the DevTools Protocol with every request outside the directory refused, seeks to each
//! frame time, captures the frame and pipes it to `ffmpeg`. The page never plays. See
//! `docs/rendering.md`.
#![deny(unused, dead_code, unsafe_code)]

mod cdp;
mod chrome;
mod encode;
mod error;
mod guard;
mod job;
mod page;
mod pipe;
pub mod tools;

use std::ops::ControlFlow;
use std::time::Instant;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use sha2::{Digest, Sha256};

pub use error::Error;
pub use job::{Capture, Job, Progress, Report};
pub use tools::Tools;

use cdp::Browser;
use encode::Encoder;
use page::Page;

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
    let filmed = film(&mut browser, job, tools, progress);
    let blocked = browser.guard.refused().to_vec();
    drop(browser);
    let frame_hashes = filmed.inspect_err(|_| _ = std::fs::remove_file(&job.output))?;
    let frames = frame_hashes.len() as u32;
    let duration = f64::from(frames) / f64::from(job.fps);
    Ok(Report { frames, duration, frame_hashes, blocked, capture: job.capture, elapsed: started.elapsed() })
}

fn film(browser: &mut Browser, job: &Job, tools: &Tools, progress: OnProgress) -> Result<Vec<String>, Error> {
    let mut page = Page::open(browser, job)?;
    let duration = page.load()?;
    let (first, count) = job.frames(duration)?;
    let mut encoder = Encoder::start(tools, job)?;
    let hashes = frames(&mut page, job.fps, first..first + u64::from(count), &mut encoder, progress)?;
    encoder.finish()?;
    Ok(hashes)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// The capture loop: seek, shoot, encode. A frame the compositor reports unchanged repeats the
/// one before. Swapping `<video>` elements for pre-extracted frames belongs between the seek
/// and the shot.
fn frames(
    page: &mut Page,
    fps: u32,
    indexes: std::ops::Range<u64>,
    encoder: &mut Encoder,
    progress: OnProgress,
) -> Result<Vec<String>, Error> {
    let total = (indexes.end - indexes.start) as u32;
    let mut hashes = Vec::with_capacity(total as usize);
    let mut frame = Vec::new();
    for index in indexes {
        page.seek(index as f64 / f64::from(fps))?;
        if let Some(data) = page.shot(index)? {
            frame = STANDARD.decode(data).map_err(|error| Error::Chrome(format!("bad frame data: {error}")))?;
        }
        if frame.is_empty() {
            return Err(Error::Chrome("the first frame came back empty".to_owned()));
        }
        hashes.push(hex(&Sha256::digest(&frame)));
        encoder.write(&frame)?;
        let done = Progress { frame: hashes.len() as u32, frames: total };
        if progress(done).is_break() {
            return Err(Error::Cancelled);
        }
    }
    Ok(hashes)
}
