//! A scene body as the readers see it: raw HTML as written, any other layout with its fenced
//! code blanked, so a tag inside a code block is never read as a clip or a file reference.

use crate::doc::Unit;
use crate::lines::fenced;

fn masked(lines: &[String]) -> String {
    let mut fence = None;
    let mut out = String::new();
    for line in lines {
        let inside = fence.is_some();
        fence = fenced(fence, line);
        match inside || fence.is_some() {
            true => out.extend(line.bytes().map(|byte| if byte == b'\n' { '\n' } else { ' ' })),
            false => out.push_str(line),
        }
    }
    out
}

/// The body of `unit` as the clip reader sees it: as written in a raw scene, with fenced code
/// blanked in any other, cut where `source` ends.
pub(crate) fn readable(source: &str, unit: &Unit, raw: bool) -> String {
    let mut text = if raw { unit.body.concat() } else { masked(&unit.body) };
    text.truncate(text.len().min(source.len() - unit.at));
    text
}
