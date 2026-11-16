use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

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
