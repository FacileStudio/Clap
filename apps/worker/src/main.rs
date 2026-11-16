//! Clap's render worker.
//!
//! It polls one directory for jobs, renders one at a time with `clap-render`, and answers
//! through files in the job's own directory. It opens no socket and reads no database: the
//! service runs with no network, and the directory is all it shares with the API. See
//! `docs/rendering.md`.
#![deny(unused, dead_code, unsafe_code)]

mod clock;
mod config;
mod files;
mod health;
mod queue;
mod run;
mod spec;
mod status;

use std::process::ExitCode;

use clap_render::Tools;
use serde_json::json;

use config::Config;

fn log(level: &str, message: &str, detail: serde_json::Value) {
    eprintln!("{}", json!({"time": clock::now(), "level": level, "msg": message, "detail": detail}));
}

fn serve(config: &Config, tools: &Tools) -> ! {
    let recovered = queue::recover(config);
    log(
        "info",
        "render worker started",
        json!({"renders": config.renders, "recovered": recovered, "sandbox": tools.sandbox}),
    );
    loop {
        health::beat(&config.renders);
        let Some(dir) = queue::next(config) else {
            std::thread::sleep(config.poll);
            continue;
        };
        let job = dir.file_name().unwrap_or_default().to_string_lossy().into_owned();
        log("info", "render started", json!({"job": job}));
        let status = run::run(&dir, tools, &config.limits);
        log(
            "info",
            "render finished",
            json!({"job": job, "state": status.state, "frames": status.frame, "error": status.error}),
        );
    }
}

fn main() -> ExitCode {
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
