//! The Chrome DevTools Protocol: one command at a time, events handled while waiting.

use std::io::Write;
use std::sync::mpsc::RecvTimeoutError;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use tempfile::TempDir;

use crate::Error;
use crate::guard::Guard;
use crate::pipe::{self, Piped};
use crate::tools::Tools;

/// How long one command may take.
const CALL_TIMEOUT: Duration = Duration::from_secs(30);
/// How long Chrome gets to exit after `Browser.close`.
const EXIT_GRACE: Duration = Duration::from_secs(3);

/// One Chrome process. Dropping it closes Chrome and removes its profile.
pub(crate) struct Browser {
    chrome: Piped,
    next_id: u64,
    profile: TempDir,
    /// Decides every request Chrome pauses.
    pub(crate) guard: Guard,
}

fn answer(method: &str, mut message: Value) -> Result<Value, Error> {
    match message.get("error") {
        Some(error) => Err(Error::Chrome(format!("{method}: {}", error["message"].as_str().unwrap_or("failed")))),
        None => Ok(message["result"].take()),
    }
}

impl Browser {
    /// Starts Chrome with `flags` and a fresh profile.
    pub(crate) fn launch(tools: &Tools, flags: Vec<String>, guard: Guard) -> Result<Browser, Error> {
        let profile = tempfile::Builder::new().prefix("clap-chrome-").tempdir()?;
        let chrome = pipe::spawn(&tools.chrome, flags, profile.path())?;
        Ok(Browser { chrome, next_id: 0, profile, guard })
    }

    fn died(&self) -> Error {
        let log = std::fs::read_to_string(self.profile.path().join("stderr.log")).unwrap_or_default();
        let mut tail: Vec<&str> = log.lines().rev().take(5).collect();
        tail.reverse();
        Error::Chrome(format!("the browser closed its pipe: {}", tail.join(" | ")))
    }

    fn send(&mut self, session: Option<&str>, method: &str, params: Value) -> Result<u64, Error> {
        self.next_id += 1;
        let mut message = json!({"id": self.next_id, "method": method, "params": params});
        if let Some(session) = session {
            message["sessionId"] = json!(session);
        }
        let mut bytes = message.to_string().into_bytes();
        bytes.push(0);
        self.chrome.writer.write_all(&bytes).map_err(|_| self.died())?;
        Ok(self.next_id)
    }

    /// Sends one command and waits for its answer, deciding paused requests meanwhile.
    pub(crate) fn call(&mut self, session: Option<&str>, method: &str, params: Value) -> Result<Value, Error> {
        let id = self.send(session, method, params)?;
        let deadline = Instant::now() + CALL_TIMEOUT;
        loop {
            let message = match self.chrome.inbox.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
                Ok(message) => message,
                Err(RecvTimeoutError::Timeout) => return Err(Error::Timeout(method.to_owned(), CALL_TIMEOUT)),
                Err(RecvTimeoutError::Disconnected) => return Err(self.died()),
            };
            if message["id"].as_u64() == Some(id) {
                return answer(method, message);
            }
            self.on_event(&message)?;
        }
    }

    fn on_event(&mut self, message: &Value) -> Result<(), Error> {
        match message["method"].as_str() {
            Some("Inspector.targetCrashed") => Err(Error::Chrome("the page crashed".to_owned())),
            Some("Fetch.requestPaused") => self.decide(message),
            _ => Ok(()),
        }
    }

    fn decide(&mut self, message: &Value) -> Result<(), Error> {
        let url = message["params"]["request"]["url"].as_str().unwrap_or_default();
        let id = &message["params"]["requestId"];
        let session = message["sessionId"].as_str().map(str::to_owned);
        let (method, verdict) = match self.guard.decide(url) {
            true => ("Fetch.continueRequest", json!({"requestId": id})),
            false => ("Fetch.failRequest", json!({"requestId": id, "errorReason": "BlockedByClient"})),
        };
        self.send(session.as_deref(), method, verdict).map(|_| ())
    }
}

impl Drop for Browser {
    fn drop(&mut self) {
        let _ = self.send(None, "Browser.close", json!({}));
        let deadline = Instant::now() + EXIT_GRACE;
        while Instant::now() < deadline && matches!(self.chrome.child.try_wait(), Ok(None)) {
            std::thread::sleep(Duration::from_millis(20));
        }
        let _ = self.chrome.child.kill();
        let _ = self.chrome.child.wait();
    }
}
