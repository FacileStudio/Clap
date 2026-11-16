//! Clap's render worker.
//!
//! It polls one directory for jobs and runs one at a time: a render with `clap-render`, or a
//! media job that probes an upload and makes its poster, filmstrip, waveform and proxy with
//! `clap-media`'s commands. It answers through files in the job's own directory. It opens no socket and reads no database: the
//! service runs with no network, and the directory is all it shares with the API. See
//! `docs/rendering.md`.
#![deny(unused, dead_code, unsafe_code)]

mod child;
mod clock;
mod config;
mod decode;
mod files;
mod health;
mod media;
mod queue;
mod run;
mod shared;
mod spec;
mod status;

use std::process::ExitCode;

use clap_render::Tools;
use serde_json::json;

use config::Config;
use decode::Media;
use spec::{Head, Kind};

fn log(level: &str, message: &str, detail: serde_json::Value) {
    eprintln!("{}", json!({"time": clock::now(), "level": level, "msg": message, "detail": detail}));
}

fn work(dir: &std::path::Path, config: &Config, tools: &Tools, media: &Media) -> status::Status {
    match Head::load(dir) {
        Ok((head, _)) if head.kind == Kind::Media => media::run(dir, media, &config.limits),
        _ => run::run(dir, tools, &config.limits),
    }
}

fn serve(config: &Config, tools: &Tools) -> ! {
    let media = Media { ffmpeg: tools.ffmpeg.clone(), ffprobe: clap_render::tools::ffprobe() };
    let recovered = queue::recover(config);
    log(
        "info",
        "render worker started",
        json!({"renders": config.renders, "recovered": recovered, "sandbox": tools.sandbox, "ffprobe": media.ffprobe}),
    );
    loop {
        health::beat(&config.renders);
        let Some(dir) = queue::next(config) else {
            std::thread::sleep(config.poll);
            continue;
        };
        let job = dir.file_name().unwrap_or_default().to_string_lossy().into_owned();
        log("info", "job started", json!({"job": job}));
        let status = work(&dir, config, tools, &media);
        log(
            "info",
            "job finished",
            json!({"job": job, "state": status.state, "frames": status.frame, "error": status.error}),
        );
    }
}

/// The worker's file creation mask: what it creates in a job directory stays writable by the
/// API's group, which removes the directory once the job is over.
const UMASK: u32 = 0o002;

fn main() -> ExitCode {
    rustix::process::umask(rustix::fs::Mode::from_raw_mode(UMASK));
    let config = Config::from_env();
    if std::env::args().nth(1).as_deref() == Some("healthcheck") {
        return if health::alive(&config.renders) { ExitCode::SUCCESS } else { ExitCode::FAILURE };
    }
    if !files::is_dir(&config.renders) {
        log("error", "the renders directory is missing", json!({"renders": config.renders}));
        return ExitCode::FAILURE;
    }
    match Tools::locate() {
        Ok(tools) => serve(&config, &tools),
        Err(error) => {
            log("error", "a binary the renderer needs is missing", json!({"error": error.to_string()}));
            ExitCode::FAILURE
        }
    }
}
