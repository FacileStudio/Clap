use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use tronc::env::Vars;

/// The media binaries the API runs: never a shell, always an argument vector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tools {
    /// `CLAP_FFMPEG`, or `ffmpeg` on `PATH`.
    pub ffmpeg: PathBuf,
    /// `CLAP_FFPROBE`, or `ffprobe` on `PATH`.
    pub ffprobe: PathBuf,
}

impl Tools {
    /// Reads `CLAP_FFMPEG` and `CLAP_FFPROBE`; a blank one means the name looked up on `PATH`.
    pub fn from_vars(vars: &Vars) -> Self {
        let named = |key: &str, name: &str| {
            let raw = vars.string(key, "");
            PathBuf::from(if raw.trim().is_empty() { name } else { raw.trim() })
        };
        Tools { ffmpeg: named("CLAP_FFMPEG", "ffmpeg"), ffprobe: named("CLAP_FFPROBE", "ffprobe") }
    }
}

fn last_line(stderr: &[u8]) -> String {
    let text = String::from_utf8_lossy(stderr);
    let line = text.lines().rev().find(|line| !line.trim().is_empty()).unwrap_or("no output");
    line.trim().chars().take(300).collect()
}

/// Runs `program` with `args` and answers its standard output. The child reads nothing, is
/// killed when `limit` passes or when the caller is dropped, and a non-zero exit answers the
/// last line it wrote to standard error.
pub async fn run(program: &Path, args: &[OsString], limit: Duration) -> Result<Vec<u8>, String> {
    let name = program.file_name().unwrap_or(program.as_os_str()).to_string_lossy().into_owned();
    let child = tokio::process::Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|error| format!("{name} could not start: {error}"))?;
    match tokio::time::timeout(limit, child.wait_with_output()).await {
        Err(_) => Err(format!("{name} was stopped after {} s", limit.as_secs())),
        Ok(Err(error)) => Err(format!("{name} failed: {error}")),
        Ok(Ok(output)) if output.status.success() => Ok(output.stdout),
        Ok(Ok(output)) => Err(format!("{name} failed: {}", last_line(&output.stderr))),
    }
}

/// `words` as an argument vector, each word one whole element.
pub fn argv(words: &[&str]) -> Vec<OsString> {
    words.iter().map(OsString::from).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_child_is_stopped_at_its_limit_and_a_missing_binary_is_named() {
        let slow = run(Path::new("sleep"), &argv(&["5"]), Duration::from_millis(50)).await.unwrap_err();
        assert!(slow.contains("sleep was stopped"), "{slow}");
        let missing = run(Path::new("/nonexistent/ffprobe"), &[], Duration::from_secs(1)).await.unwrap_err();
        assert!(missing.contains("ffprobe could not start"), "{missing}");
        assert_eq!(run(Path::new("printf"), &argv(&["a b"]), Duration::from_secs(5)).await.unwrap(), b"a b");
    }
}
