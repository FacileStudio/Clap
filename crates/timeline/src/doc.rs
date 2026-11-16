//! A source file as the lines of its scenes, so that a scene can move without a byte of it
//! changing.

use crate::lines::{key, scene_key, text};

/// The frontmatter block of one scene.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Front {
    /// The lines between the two `---`, line endings kept.
    pub lines: Vec<String>,
    /// The `---` line that closes the block.
    pub close: String,
}

/// One scene: the `---` that opens it, its frontmatter and its body.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct Unit {
    pub sep: Option<String>,
    pub front: Option<Front>,
    /// The body lines, line endings kept.
    pub body: Vec<String>,
    /// The 1-based line the body starts on, as parsed.
    pub line: usize,
    /// The byte the body starts at, as parsed.
    pub at: usize,
}

/// A whole file.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Doc {
    pub bom: String,
    pub units: Vec<Unit>,
    pub tail: Vec<String>,
    /// The line ending new lines get.
    pub eol: &'static str,
    pub open_end: bool,
}

impl Unit {
    pub(crate) fn all(self) -> Vec<String> {
        let front = self.front.map(|front| [front.lines, vec![front.close]].concat()).unwrap_or_default();
        [self.sep.into_iter().collect(), front, self.body].concat()
    }

    /// Removes and answers the frontmatter lines that belong to the project, not to the scene.
    pub(crate) fn take_project(&mut self) -> Vec<String> {
        let Some(front) = self.front.as_mut() else { return Vec::new() };
        let (kept, project) = std::mem::take(&mut front.lines).into_iter().partition(|line| scene_key(line));
        front.lines = kept;
        if front.lines.is_empty() {
            self.front = None;
        }
        project
    }

    /// Puts project lines at the top of the frontmatter, opening one when the scene has none.
    pub(crate) fn give_project(&mut self, project: Vec<String>, eol: &str) {
        if project.is_empty() {
            return;
        }
        let front = self.front.get_or_insert_with(|| Front { lines: Vec::new(), close: format!("---{eol}") });
        front.lines.splice(0..0, project);
    }
}

impl Doc {
    /// Makes every scene readable where it now sits: the first one needs no `---` unless it has
    /// frontmatter, the others need one, and a body that opens like frontmatter needs an empty
    /// frontmatter block before it.
    pub(crate) fn settle(&mut self) {
        let sep = format!("---{}", self.eol);
        for (index, unit) in self.units.iter_mut().enumerate() {
            let keyed = unit.body.first().is_some_and(|line| key(line).is_some());
            match (index, unit.front.is_some()) {
                (0, false) => unit.sep = None,
                (_, false) if keyed => unit.front = Some(Front { lines: Vec::new(), close: sep.clone() }),
                _ => {}
            }
            if index > 0 || unit.front.is_some() {
                unit.sep.get_or_insert_with(|| sep.clone());
            }
        }
    }

    /// The file as text.
    pub(crate) fn emit(&self) -> String {
        let units = self.units.iter().cloned().flat_map(Unit::all);
        let mut out: String = std::iter::once(self.bom.clone()).chain(units).chain(self.tail.iter().cloned()).collect();
        if self.open_end && out.ends_with(self.eol) {
            out.truncate(out.len() - self.eol.len());
        }
        out
    }
}

/// The lines of `body` as text with their line ending, the last one closed with `eol`.
pub(crate) fn lines_of(body: &str, eol: &str) -> Vec<String> {
    let closed = |line: &str| if line.ends_with('\n') { line.to_owned() } else { format!("{}{eol}", text(line)) };
    body.split_inclusive('\n').map(closed).collect()
}
