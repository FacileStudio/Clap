//! Cuts a source file into scenes exactly where `facile-scene` cuts it.

use crate::doc::{Doc, Front, Unit};
use crate::lines::{fenced, key, separator};

struct Cursor<'a> {
    lines: Vec<&'a str>,
    at: usize,
    byte: usize,
}

impl<'a> Cursor<'a> {
    fn peek(&self) -> Option<&'a str> {
        self.lines.get(self.at).copied()
    }

    fn take(&mut self) -> String {
        let line = self.lines[self.at];
        self.at += 1;
        self.byte += line.len();
        line.to_owned()
    }

    fn front(&mut self) -> Option<Front> {
        self.peek().filter(|line| key(line).is_some() || separator(line))?;
        let mut lines = Vec::new();
        while let Some(line) = self.peek() {
            if separator(line) {
                return Some(Front { lines, close: self.take() });
            }
            lines.push(self.take());
        }
        Some(Front { lines, close: String::new() })
    }

    fn unit(&mut self, opened: bool) -> Unit {
        let sep = opened.then(|| self.take());
        let front = if opened { self.front() } else { None };
        let mut unit = Unit { sep, front, body: Vec::new(), line: self.at + 1, at: self.byte };
        let mut fence = None;
        while let Some(line) = self.peek().filter(|line| fence.is_some() || !separator(line)) {
            fence = fenced(fence, line);
            unit.body.push(self.take());
        }
        unit
    }
}

fn blank(unit: &Unit) -> bool {
    let keyed = unit.front.as_ref().is_some_and(|front| front.lines.iter().any(|line| key(line).is_some()));
    !keyed && unit.body.iter().all(|line| line.trim().is_empty())
}

/// Cuts `source` the way the scene parser does.
pub(crate) fn cut(source: &str) -> Doc {
    let rest = source.trim_start_matches('\u{feff}');
    let bom = source[..source.len() - rest.len()].to_owned();
    let eol = if rest.split_inclusive('\n').next().is_some_and(|line| line.ends_with("\r\n")) { "\r\n" } else { "\n" };
    let open_end = !rest.is_empty() && !rest.ends_with('\n');
    let closed = if open_end { format!("{rest}{eol}") } else { rest.to_owned() };
    let mut cursor = Cursor { lines: closed.split_inclusive('\n').collect(), at: 0, byte: bom.len() };
    let mut units = vec![cursor.unit(cursor.peek().is_some_and(separator))];
    while cursor.peek().is_some() {
        units.push(cursor.unit(true));
    }
    let trailing = units.len() > 1 && units.last().is_some_and(blank);
    let tail = if trailing { units.pop().map(Unit::all).unwrap_or_default() } else { Vec::new() };
    Doc { bom, units, tail, eol, open_end }
}
