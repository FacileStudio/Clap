//! What to render, and what a render answers.

use std::ops::Range;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::Error;

/// How frames leave Chrome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capture {
    /// `HeadlessExperimental.beginFrame` under `--deterministic-mode`: one seek, one composited
    /// frame. Linux only.
    BeginFrame,
    /// `Page.captureScreenshot` once two animation frames have landed after the seek.
    Screenshot,
}

impl Default for Capture {
    fn default() -> Self {
        if cfg!(target_os = "linux") { Capture::BeginFrame } else { Capture::Screenshot }
    }
}

/// One render: a bundle directory holding `index.html`, the picture format and the output file.
#[derive(Debug, Clone)]
pub struct Job {
    /// The directory holding `index.html`; the page may load files under it and nothing else.
    pub dir: PathBuf,
    /// Frame width in pixels, even.
    pub width: u32,
    /// Frame height in pixels, even.
    pub height: u32,
    /// Frames per second.
    pub fps: u32,
    /// The seconds to render; `None` is the whole scene.
    pub range: Option<Range<f64>>,
    /// The MP4 to write.
    pub output: PathBuf,
    /// How frames are captured.
    pub capture: Capture,
    /// The most bytes the frames extracted from the scene's videos may take on disk. They sit in
    /// a directory under `dir` for the length of the render.
    pub frame_budget: u64,
}

impl Job {
    /// The whole scene of `dir` at 1920x1080 and 30 fps, written to `output`.
    pub fn new(dir: impl Into<PathBuf>, output: impl Into<PathBuf>) -> Job {
        Job {
            dir: dir.into(),
            width: 1920,
            height: 1080,
            fps: 30,
            range: None,
            output: output.into(),
            capture: Capture::default(),
            frame_budget: 8 << 30,
        }
    }

    /// Refuses a size H.264 with 4:2:0 chroma cannot hold, a zero rate and a missing entry file.
    pub(crate) fn check(&self) -> Result<(), Error> {
        let even = |side: u32| (2..=8192).contains(&side) && side.is_multiple_of(2);
        if !even(self.width) || !even(self.height) {
            return Err(Error::Job(format!("{}x{} must be even and at most 8192", self.width, self.height)));
        }
        if !(1..=240).contains(&self.fps) {
            return Err(Error::Job(format!("{} fps must be between 1 and 240", self.fps)));
        }
        if !entry(&self.dir).is_file() {
            return Err(Error::Job(format!("{} has no index.html", self.dir.display())));
        }
        Ok(())
    }

    /// The frames to capture from a scene lasting `duration` seconds: first frame index, count.
    pub(crate) fn frames(&self, duration: f64) -> Result<(u64, u32), Error> {
        let range = self.range.clone().unwrap_or(0.0..duration);
        let valid = range.start >= 0.0 && range.end > range.start && range.end <= duration + 1e-6;
        if !valid {
            return Err(Error::Job(format!("range {range:?} is not inside the scene's {duration} s")));
        }
        let fps = f64::from(self.fps);
        let first = (range.start * fps).round();
        let count = (range.end * fps).round() - first;
        if count < 1.0 || count > f64::from(u32::MAX) {
            return Err(Error::Job(format!("range {range:?} holds no frame at {} fps", self.fps)));
        }
        Ok((first as u64, count as u32))
    }
}

/// The page a bundle opens with.
pub(crate) fn entry(dir: &Path) -> PathBuf {
    dir.join("index.html")
}

/// How far a render is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Progress {
    /// Frames captured so far.
    pub frame: u32,
    /// Frames the render will capture.
    pub frames: u32,
}

/// What a finished render produced.
#[derive(Debug, Clone)]
pub struct Report {
    /// Frames written to the MP4.
    pub frames: u32,
    /// Seconds of video: `frames / fps`.
    pub duration: f64,
    /// The hex SHA-256 of each captured frame, in order.
    pub frame_hashes: Vec<String>,
    /// The URLs the page asked for and was refused, at most the first hundred.
    pub blocked: Vec<String>,
    /// The capture path that ran.
    pub capture: Capture,
    /// How many `<video>` elements were swapped for extracted frames.
    pub videos: u32,
    /// How many sound tracks were mixed. Zero gives an MP4 without an audio stream.
    pub audio_tracks: u32,
    /// Wall time, Chrome launch to finished MP4.
    pub elapsed: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_are_counted_on_the_frame_grid() {
        let mut job = Job::new("/nonexistent", "out.mp4");
        assert_eq!(job.frames(2.0).unwrap(), (0, 60));
        job.range = Some(0.5..1.5);
        assert_eq!(job.frames(2.0).unwrap(), (15, 30));
        job.range = Some(1.0..1.0333334);
        assert_eq!(job.frames(2.0).unwrap(), (30, 1));
        for bad in [0.0..3.0, 1.0..1.0, -1.0..1.0, 0.0..0.001] {
            job.range = Some(bad.clone());
            assert!(job.frames(2.0).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn odd_sizes_and_silly_rates_are_refused() {
        let dir = tempfile::tempdir().unwrap();
        let mut job = Job::new(dir.path(), "out.mp4");
        assert!(job.check().unwrap_err().to_string().contains("index.html"));
        std::fs::write(entry(dir.path()), "").unwrap();
        assert!(job.check().is_ok());
        job.width = 1281;
        assert!(job.check().is_err());
        job.width = 1280;
        job.fps = 0;
        assert!(job.check().is_err());
    }
}
