//! Where the binaries the renderer shells out to live.

use std::path::{Path, PathBuf};

use crate::Error;

/// The binaries one render needs.
#[derive(Debug, Clone)]
pub struct Tools {
    /// `chrome-headless-shell`.
    pub chrome: PathBuf,
    /// `ffmpeg`.
    pub ffmpeg: PathBuf,
    /// Whether Chrome keeps its own sandbox. Off only where the host cannot grant it one.
    pub sandbox: bool,
}

impl Tools {
    /// Finds both binaries: `CLAP_CHROME` and `CLAP_FFMPEG`, then `PATH`, then, for Chrome, the
    /// newest build in Playwright's cache. `CLAP_CHROME_SANDBOX=off` turns Chrome's sandbox off.
    pub fn locate() -> Result<Tools, Error> {
        let chrome = chrome().ok_or(Error::MissingTool("chrome-headless-shell", "CLAP_CHROME"))?;
        let ffmpeg = ffmpeg().ok_or(Error::MissingTool("ffmpeg", "CLAP_FFMPEG"))?;
        let sandbox = std::env::var("CLAP_CHROME_SANDBOX").map_or(true, |value| value != "off");
        Ok(Tools { chrome, ffmpeg, sandbox })
    }
}

fn on_path(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path).map(|dir| dir.join(name)).find(|file| file.is_file())
}

fn named(key: &str, name: &str) -> Option<PathBuf> {
    match std::env::var_os(key).filter(|value| !value.is_empty()) {
        Some(path) => Some(PathBuf::from(path)).filter(|file| file.is_file()),
        None => on_path(name),
    }
}

fn revision(dir: &Path) -> u64 {
    let name = dir.file_name().and_then(|name| name.to_str()).unwrap_or_default();
    name.rsplit('-').next().and_then(|digits| digits.parse().ok()).unwrap_or(0)
}

fn playwright_chrome() -> Option<PathBuf> {
    let cache = match std::env::var_os("PLAYWRIGHT_BROWSERS_PATH") {
        Some(dir) => PathBuf::from(dir),
        None => PathBuf::from(std::env::var_os("HOME")?).join(".cache/ms-playwright"),
    };
    let mut builds: Vec<PathBuf> = std::fs::read_dir(cache)
        .ok()?
        .filter_map(|entry| Some(entry.ok()?.path()))
        .filter(|path| revision(path) > 0 && path.to_string_lossy().contains("chromium_headless_shell-"))
        .collect();
    builds.sort_by_key(|path| std::cmp::Reverse(revision(path)));
    builds.iter().find_map(|build| {
        let platforms = std::fs::read_dir(build).ok()?;
        let mut shells = platforms.filter_map(|entry| Some(entry.ok()?.path().join("chrome-headless-shell")));
        shells.find(|file| file.is_file())
    })
}

/// `chrome-headless-shell`: `CLAP_CHROME` alone when it is set, else `PATH`, then Playwright's cache.
pub fn chrome() -> Option<PathBuf> {
    let unset = std::env::var_os("CLAP_CHROME").is_none_or(|value| value.is_empty());
    named("CLAP_CHROME", "chrome-headless-shell").or_else(|| unset.then(playwright_chrome).flatten())
}

/// `ffmpeg`: `CLAP_FFMPEG`, then `PATH`.
pub fn ffmpeg() -> Option<PathBuf> {
    named("CLAP_FFMPEG", "ffmpeg")
}

/// `ffprobe`: `CLAP_FFPROBE`, then `PATH`.
pub fn ffprobe() -> Option<PathBuf> {
    named("CLAP_FFPROBE", "ffprobe")
}
