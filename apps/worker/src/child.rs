//! One `ffprobe` or `ffmpeg` child of a media job, with a deadline. It never outlives the call
//! that started it, and it reads nothing from the worker.

use std::ffi::OsString;
use std::fs::File;
use std::io::{Read, Seek};
use std::path::Path;
use std::process::{Child, ChildStdout, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

/// How often a running child is looked at.
const TICK: Duration = Duration::from_millis(50);

struct Running(Child);

impl Drop for Running {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn waited(child: &mut Child, name: &str, limit: Duration, stop: &dyn Fn() -> bool) -> Result<ExitStatus, String> {
    let deadline = Instant::now() + limit;
    loop {
        if let Some(status) = child.try_wait().map_err(|error| format!("{name} failed: {error}"))? {
            return Ok(status);
        }
        if Instant::now() > deadline {
            return Err(format!("{name} was stopped after {} s", limit.as_secs()));
        }
        if stop() {
            return Err(format!("{name} was stopped with the job"));
        }
        std::thread::sleep(TICK);
    }
}

fn said(log: &mut File) -> String {
    let mut text = String::new();
    let _ = log.rewind().and_then(|_| log.take(64 << 10).read_to_string(&mut text));
    let line = text.lines().rev().find(|line| !line.trim().is_empty()).unwrap_or("no output");
    line.trim().chars().take(300).collect()
}

/// Runs `program` with `args`, hands its standard output to `read` on its own thread, and
/// answers what `read` answered. The child is killed when `limit` passes or `stop` answers true;
/// a non-zero exit answers the last line it wrote to standard error.
pub(crate) fn run<T: Send>(
    program: &Path,
    args: &[OsString],
    limit: Duration,
    stop: &dyn Fn() -> bool,
    read: impl FnOnce(ChildStdout) -> T + Send,
) -> Result<T, String> {
    let name = program.file_name().unwrap_or(program.as_os_str()).to_string_lossy().into_owned();
    let unstarted = |error: std::io::Error| format!("{name} could not start: {error}");
    let mut log = tempfile::tempfile().map_err(unstarted)?;
    let mut command = Command::new(program);
    command.args(args).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(log.try_clone().map_err(unstarted)?);
    let mut running = Running(command.spawn().map_err(unstarted)?);
    let stdout = running.0.stdout.take().ok_or_else(|| format!("{name} has no output"))?;
    let (status, value) = std::thread::scope(|scope| {
        let reader = scope.spawn(move || read(stdout));
        let status = waited(&mut running.0, &name, limit, stop);
        if status.is_err() {
            let _ = running.0.kill();
        }
        (status, reader.join())
    });
    match (status?, value) {
        (status, Ok(value)) if status.success() => Ok(value),
        _ => Err(format!("{name} failed: {}", said(&mut log))),
    }
}

/// Reads `out` to its end and throws it away, so a child is never blocked on a full pipe.
pub(crate) fn drain(mut out: ChildStdout) {
    let _ = std::io::copy(&mut out, &mut std::io::sink());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(words: &[&str]) -> Vec<OsString> {
        words.iter().map(OsString::from).collect()
    }

    #[test]
    fn a_child_answers_its_output_and_is_stopped_at_its_limit() {
        let never = || false;
        let read = |mut out: ChildStdout| {
            let mut text = String::new();
            out.read_to_string(&mut text).map(|_| text).unwrap_or_default()
        };
        assert_eq!(run(Path::new("printf"), &words(&["a b"]), Duration::from_secs(5), &never, read).unwrap(), "a b");
        let slow = run(Path::new("sleep"), &words(&["5"]), Duration::from_millis(100), &never, drain).unwrap_err();
        assert!(slow.contains("sleep was stopped after 0 s"), "{slow}");
        let stopped = run(Path::new("sleep"), &words(&["5"]), Duration::from_secs(5), &|| true, drain).unwrap_err();
        assert!(stopped.contains("stopped with the job"), "{stopped}");
        let failed =
            run(Path::new("sh"), &words(&["-c", "echo nope >&2; exit 3"]), Duration::from_secs(5), &never, drain);
        assert_eq!(failed.unwrap_err(), "sh failed: nope");
        let missing = run(Path::new("/nonexistent/ffprobe"), &[], Duration::from_secs(1), &never, drain).unwrap_err();
        assert!(missing.contains("ffprobe could not start"), "{missing}");
    }
}
