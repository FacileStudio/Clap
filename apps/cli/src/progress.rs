//! One line on stderr that says how far a long task is. Drawn only on a terminal.

use std::io::{self, IsTerminal, Write};
use std::time::{Duration, Instant};

use crate::ui;

const EVERY: Duration = Duration::from_millis(100);
const MIB: f64 = 1_048_576.0;

/// A progress line. It erases itself when finished, so the result line takes its place.
pub struct Bar {
    label: String,
    drawn: Option<Instant>,
    shown: bool,
}

impl Bar {
    /// A line that starts with `label`. Nothing is drawn with `--json` or when stderr is piped.
    pub fn new(label: impl Into<String>) -> Bar {
        Bar { label: label.into(), drawn: None, shown: io::stderr().is_terminal() && !ui::json() }
    }

    fn draw(&mut self, text: &str, last: bool) {
        let due = self.drawn.is_none_or(|drawn| drawn.elapsed() >= EVERY);
        if self.shown && (due || last) {
            eprint!("\r\x1b[2K\u{25b8} {} {text}", self.label);
            let _ = io::stderr().flush();
            self.drawn = Some(Instant::now());
        }
    }

    /// Shows `done` of `total` things, such as frames.
    pub fn count(&mut self, done: u64, total: u64, unit: &str) {
        let percent = (done * 100).checked_div(total).unwrap_or(0);
        self.draw(&format!("{done}/{total} {unit} ({percent}%)"), done == total);
    }

    /// Shows `done` of `total` bytes, in MiB. A `total` of 0 is unknown.
    pub fn bytes(&mut self, done: u64, total: u64) {
        let size = |bytes: u64| format!("{:.1}", bytes as f64 / MIB);
        match total {
            0 => self.draw(&format!("{} MiB", size(done)), false),
            _ => self.draw(&format!("{} / {} MiB", size(done), size(total)), done == total),
        }
    }

    /// Erases the line.
    pub fn finish(&mut self) {
        if self.shown && self.drawn.take().is_some() {
            eprint!("\r\x1b[2K");
            let _ = io::stderr().flush();
        }
    }
}
