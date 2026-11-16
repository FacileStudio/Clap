//! A media job: what the API asks for, the files the worker answers with, and the order they
//! appear in.

use serde::{Deserialize, Serialize};

/// What a media job makes beside the probe, each only when the probe allows it: the poster,
/// the filmstrip and the proxy of a picture, the waveform of a sound. The API leaves out what
/// it already holds.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Make {
    /// A poster frame.
    #[serde(default)]
    pub poster: bool,
    /// A filmstrip.
    #[serde(default)]
    pub strip: bool,
    /// A waveform.
    #[serde(default)]
    pub peaks: bool,
    /// A 720p proxy.
    #[serde(default)]
    pub proxy: bool,
}

/// The steps of a media job, in the order the worker runs them. The worker's `status.json`
/// counts the steps finished in `frame`, out of [`Step::Proxy`] in `frames`: once `frame` has
/// reached a step, that step's file is whole or will never come.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Step {
    /// `probe.json`.
    Probe = 1,
    /// `poster.jpg`.
    Poster = 2,
    /// `strip.jpg`.
    Strip = 3,
    /// `peaks.json`.
    Peaks = 4,
    /// `proxy.mp4`, the last.
    Proxy = 5,
}

/// The files of a media job, relative to its directory.
pub mod files {
    /// The upload, a copy the API put in the bundle.
    pub const INPUT: &str = "bundle/input";
    /// What `ffprobe` printed.
    pub const PROBE: &str = "probe.json";
    /// One JPEG frame of the picture.
    pub const POSTER: &str = "poster.jpg";
    /// The filmstrip, one JPEG.
    pub const STRIP: &str = "strip.jpg";
    /// The waveform, `{"peaks": [...]}`.
    pub const PEAKS: &str = "peaks.json";
    /// The 720p H.264 proxy.
    pub const PROXY: &str = "proxy.mp4";
}

/// The largest file the API takes back from a media job, in bytes. The proxy's cap is the
/// asset cap, which the API knows.
pub mod max {
    /// `probe.json`.
    pub const PROBE: u64 = 64 << 10;
    /// `poster.jpg`.
    pub const POSTER: u64 = 2 << 20;
    /// `strip.jpg`: 60 frames 90 pixels high.
    pub const STRIP: u64 = 4 << 20;
    /// `peaks.json`.
    pub const PEAKS: u64 = 64 << 10;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_job_file_without_a_part_makes_nothing_of_it() {
        let make: Make = serde_json::from_str(r#"{"id": "m", "kind": "media", "proxy": true}"#).unwrap();
        assert_eq!(make, Make { proxy: true, ..Make::default() });
        assert!(Step::Probe < Step::Poster && Step::Peaks < Step::Proxy && Step::Proxy as u32 == 5);
    }
}
