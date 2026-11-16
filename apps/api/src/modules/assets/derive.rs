//! The one decode left in the API: the poster of a finished render, cut with the same
//! arguments the worker uses for an upload's poster.

use std::ffi::OsString;
use std::path::Path;
use std::time::Duration;

use super::tools::{self, Tools};

/// Runs `ffmpeg` with `args`, whose last element is the output, within `limit`, writing to a
/// scratch file renamed onto `output` once whole.
async fn written(tools: &Tools, args: Vec<OsString>, limit: Duration, output: &Path) -> Result<(), String> {
    let scratch = output.with_extension("part");
    let mut args = args;
    args.pop();
    args.push(scratch.clone().into());
    if let Some(dir) = output.parent() {
        tokio::fs::create_dir_all(dir).await.map_err(|error| error.to_string())?;
    }
    let done = tools::run(&tools.ffmpeg, &args, limit).await;
    let moved = match done {
        Ok(_) => tokio::fs::rename(&scratch, output).await.map_err(|error| error.to_string()),
        Err(error) => Err(error),
    };
    if moved.is_err() {
        tokio::fs::remove_file(&scratch).await.ok();
    }
    moved
}

/// Writes the poster of `input` to `output`, unless it is already there. The file appears whole
/// or not at all.
pub async fn poster(tools: &Tools, input: &Path, at_ms: i64, output: &Path) -> Result<(), String> {
    if tokio::fs::try_exists(output).await.unwrap_or(false) {
        return Ok(());
    }
    written(tools, clap_media::poster_args(input, at_ms, output), clap_media::POSTER_TIMEOUT, output).await
}
