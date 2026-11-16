//! One render inside a started browser: load, extract the videos, then seek, swap, shoot, encode.

use std::ops::Range;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use sha2::{Digest, Sha256};

use crate::cdp::Browser;
use crate::encode::Encoder;
use crate::job::{Job, Progress};
use crate::page::Page;
use crate::stock::Stock;
use crate::tools::Tools;
use crate::{Error, OnProgress, audio};

/// What the capture of one job produced.
pub(crate) struct Film {
    /// The hex SHA-256 of each captured frame.
    pub hashes: Vec<String>,
    pub videos: u32,
    pub audio_tracks: u32,
}

/// Renders `job` in `browser`. Every ffmpeg child and the extracted frames are gone when this
/// returns, whatever it returns.
pub(crate) fn film(browser: &mut Browser, job: &Job, tools: &Tools, progress: OnProgress) -> Result<Film, Error> {
    let mut page = Page::open(browser, job)?;
    let duration = page.load()?;
    let (first, count) = job.frames(duration)?;
    let span = first..first + u64::from(count);
    let mut waiting = || match progress(Progress { frame: 0, frames: count }).is_break() {
        true => Err(Error::Cancelled),
        false => Ok(()),
    };
    waiting()?;
    let media = page.media()?;
    let stock = Stock::extract(tools, job, (&media, span.clone()), &mut waiting)?;
    let fps = f64::from(job.fps);
    let tracks = audio::tracks(tools, &media, span.start as f64 / fps, span.end as f64 / fps)?;
    let mut encoder = Encoder::start(tools, job, &tracks, count)?;
    let hashes = frames(&mut page, (fps, span), &stock, &mut encoder, progress)?;
    encoder.finish()?;
    Ok(Film { hashes, videos: stock.len() as u32, audio_tracks: tracks.len() as u32 })
}

fn decode(data: &str) -> Result<Vec<u8>, Error> {
    STANDARD.decode(data).map_err(|error| Error::Chrome(format!("bad frame data: {error}")))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// The capture loop: seek, hand each video its still, shoot, encode. A frame the compositor
/// reports unchanged repeats the one before; a primed frame drawn before the loop stands in
/// for the one before the first.
fn frames(
    page: &mut Page,
    (fps, indexes): (f64, Range<u64>),
    stock: &Stock,
    encoder: &mut Encoder,
    progress: OnProgress,
) -> Result<Vec<String>, Error> {
    let total = (indexes.end - indexes.start) as u32;
    let mut hashes = Vec::with_capacity(total as usize);
    let mut frame = match page.shot(None)? {
        Some(data) => decode(&data)?,
        None => Vec::new(),
    };
    for index in indexes {
        page.seek(index as f64 / fps)?;
        page.show(&stock.at(index))?;
        if let Some(data) = page.shot(Some(index))? {
            frame = decode(&data)?;
        }
        if frame.is_empty() {
            return Err(Error::Chrome("the first frame came back empty".to_owned()));
        }
        hashes.push(hex(&Sha256::digest(&frame)));
        encoder.write(&frame)?;
        if progress(Progress { frame: hashes.len() as u32, frames: total }).is_break() {
            return Err(Error::Cancelled);
        }
    }
    Ok(hashes)
}
