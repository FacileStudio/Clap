//! What the worker reads from its environment, once, at start.

use std::path::PathBuf;
use std::time::Duration;

/// The caps the worker enforces on every job, whatever the job file asks.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Limits {
    /// `CLAP_RENDER_MAX_WIDTH`: the longer side of the frame, 1920 by default.
    pub long_side: u32,
    /// `CLAP_RENDER_MAX_HEIGHT`: the shorter side of the frame, 1080 by default.
    pub short_side: u32,
    /// `CLAP_RENDER_MAX_FPS`, 30 by default.
    pub fps: u32,
    /// `CLAP_RENDER_MAX_SECONDS`: the length of the rendered video, 600 by default.
    pub seconds: f64,
    /// `CLAP_RENDER_TIMEOUT_SECONDS`: the wall clock one job gets, 3600 by default.
    pub wall: Duration,
    /// `CLAP_RENDER_FRAMES_MAX_BYTES`: the disk the extracted video frames of one job may take,
    /// 8 GiB by default.
    pub frame_bytes: u64,
}

/// The worker's settings.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Config {
    /// `CLAP_RENDERS_DIR`, `/data/renders` by default: one directory per job.
    pub renders: PathBuf,
    /// `CLAP_RENDER_POLL_MS`: the pause between two looks at an empty queue, 1000 by default.
    pub poll: Duration,
    pub limits: Limits,
}

fn number<T: std::str::FromStr>(key: &str, default: T) -> T {
    std::env::var(key).ok().and_then(|value| value.trim().parse().ok()).unwrap_or(default)
}

impl Default for Limits {
    fn default() -> Self {
        Limits {
            long_side: 1920,
            short_side: 1080,
            fps: 30,
            seconds: 600.0,
            wall: Duration::from_secs(3600),
            frame_bytes: 8 << 30,
        }
    }
}

impl Config {
    /// Reads the environment. A value that is not a number keeps the default.
    pub(crate) fn from_env() -> Config {
        let default = Limits::default();
        let limits = Limits {
            long_side: number("CLAP_RENDER_MAX_WIDTH", default.long_side),
            short_side: number("CLAP_RENDER_MAX_HEIGHT", default.short_side),
            fps: number("CLAP_RENDER_MAX_FPS", default.fps),
            seconds: number("CLAP_RENDER_MAX_SECONDS", default.seconds),
            wall: Duration::from_secs(number("CLAP_RENDER_TIMEOUT_SECONDS", default.wall.as_secs())),
            frame_bytes: number("CLAP_RENDER_FRAMES_MAX_BYTES", default.frame_bytes),
        };
        let renders = std::env::var_os("CLAP_RENDERS_DIR").filter(|value| !value.is_empty());
        Config {
            renders: renders.map_or_else(|| PathBuf::from("/data/renders"), PathBuf::from),
            poll: Duration::from_millis(number("CLAP_RENDER_POLL_MS", 1000)),
            limits,
        }
    }
}
