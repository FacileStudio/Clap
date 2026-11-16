//! Chrome's `--remote-debugging-pipe`: NUL-terminated JSON, commands on the child's fd 3,
//! answers and events on its fd 4. No port is opened, so nothing else on the host can attach.

use std::fs::File;
use std::io::{BufRead, BufReader, PipeReader, PipeWriter};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};

use command_fds::{CommandFdExt, FdMapping};
use serde_json::Value;

use crate::Error;

/// A started Chrome: the process, where to write commands, and what it says back.
pub(crate) struct Piped {
    pub(crate) child: Child,
    pub(crate) writer: PipeWriter,
    pub(crate) inbox: Receiver<Value>,
}

fn read_messages(reader: PipeReader, outbox: Sender<Value>) {
    let mut reader = BufReader::new(reader);
    let mut frame = Vec::new();
    while reader.read_until(0, &mut frame).is_ok_and(|read| read > 0) {
        let body = frame.strip_suffix(&[0]).unwrap_or(&frame);
        let sent = serde_json::from_slice(body).ok().map(|message| outbox.send(message));
        if matches!(sent, Some(Err(_))) {
            return;
        }
        frame.clear();
    }
}

/// Starts `chrome` with `flags`, its profile and its stderr log under `profile`. A thread reads
/// its messages until the pipe closes.
pub(crate) fn spawn(chrome: &Path, flags: Vec<String>, profile: &Path) -> Result<Piped, Error> {
    let (commands, writer) = std::io::pipe()?;
    let (reader, answers) = std::io::pipe()?;
    let mut command = Command::new(chrome);
    command
        .args(flags)
        .arg(format!("--user-data-dir={}", profile.display()))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(File::create(profile.join("stderr.log"))?);
    let pipes = vec![
        FdMapping { parent_fd: commands.into(), child_fd: 3 },
        FdMapping { parent_fd: answers.into(), child_fd: 4 },
    ];
    command.fd_mappings(pipes).map_err(|error| Error::Chrome(error.to_string()))?;
    let child = command.spawn()?;
    drop(command);
    let (outbox, inbox) = mpsc::channel();
    std::thread::spawn(move || read_messages(reader, outbox));
    Ok(Piped { child, writer, inbox })
}
