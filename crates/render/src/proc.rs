//! One ffmpeg child with a deadline. It never outlives the call that started it.

use std::io::{Read, Seek};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use crate::Error;

/// How often a running child is looked at.
const TICK: Duration = Duration::from_millis(100);

/// What a finished child said.
pub(crate) struct Finished {
    pub ok: bool,
    /// Its standard error.
    pub said: String,
}

struct Running(Child);

impl Drop for Running {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Runs `command` to its end, calling `tick` ten times a second. The child is killed when `tick`
/// fails, when `limit` passes, and whenever this function returns early.
pub(crate) fn run(
    command: &mut Command,
    limit: Duration,
    tick: &mut dyn FnMut() -> Result<(), Error>,
) -> Result<Finished, Error> {
    let mut log = tempfile::tempfile()?;
    command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(log.try_clone()?);
    let mut running = Running(command.spawn()?);
    let deadline = Instant::now() + limit;
    let status = loop {
        if let Some(status) = running.0.try_wait()? {
            break status;
        }
        if Instant::now() > deadline {
            return Err(Error::Timeout("ffmpeg".to_owned(), limit));
        }
        tick()?;
        std::thread::sleep(TICK);
    };
    let mut said = String::new();
    log.rewind()?;
    let _ = log.read_to_string(&mut said);
    Ok(Finished { ok: status.success(), said: said.trim().to_owned() })
}
