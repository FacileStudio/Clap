use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::Value;

use super::render::mirrored;
use super::{App, Person};

pub struct Worker(Child);

impl Drop for Worker {
    fn drop(&mut self) {
        self.0.kill().ok();
        self.0.wait().ok();
    }
}

fn binary() -> Option<PathBuf> {
    let named = std::env::var_os("CLAP_WORKER_BIN").map(PathBuf::from);
    let beside = std::env::current_exe().ok()?.parent()?.parent()?.join("worker");
    named.or(Some(beside)).filter(|path| path.is_file())
}

pub fn ready() -> Option<(PathBuf, PathBuf)> {
    let found = binary().zip(clap_render::tools::chrome());
    if found.is_none() || clap_render::tools::ffmpeg().is_none() {
        eprintln!(
            "\n=== SKIPPED: the end-to-end render needs the worker binary (cargo build -p worker, or \
             CLAP_WORKER_BIN), chrome-headless-shell (CLAP_CHROME) and ffmpeg ===\n"
        );
    }
    found.filter(|_| clap_render::tools::ffmpeg().is_some())
}

pub fn start(binary: &Path, chrome: &Path, renders: &Path) -> Worker {
    let child = Command::new(binary)
        .env("CLAP_RENDERS_DIR", renders)
        .env("CLAP_CHROME", chrome)
        .env("CLAP_CHROME_SANDBOX", "off")
        .env("CLAP_RENDER_POLL_MS", "200")
        .stdin(Stdio::null())
        .spawn()
        .unwrap();
    Worker(child)
}

pub async fn alive(app: &App) {
    let heartbeat = app.state.data_dir.join("renders/.worker");
    std::fs::remove_file(&heartbeat).ok();
    let started = Instant::now();
    while !heartbeat.is_file() {
        assert!(started.elapsed() < Duration::from_secs(30), "the worker wrote no heartbeat in 30 s");
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}

pub async fn settled(app: &App, who: &Person, render: &Value) -> Value {
    let started = Instant::now();
    loop {
        let seen = mirrored(app, who, render).await;
        if ["done", "failed", "cancelled"].contains(&seen["state"].as_str().unwrap_or_default()) {
            return seen;
        }
        assert!(started.elapsed() < Duration::from_secs(300), "the render did not finish in 5 minutes: {seen}");
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

/// A worker over `app`'s renders directory, alive, for the tests whose uploads need their media
/// job run. `None`, with the banner, when the worker binary, Chrome or ffmpeg is missing.
pub async fn media(app: &App) -> Option<Worker> {
    let (binary, chrome) = ready()?;
    let worker = start(&binary, &chrome, &app.state.data_dir.join("renders"));
    alive(app).await;
    Some(worker)
}
