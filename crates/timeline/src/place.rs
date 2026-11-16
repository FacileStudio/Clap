//! The clip edits that add or remove an element.

use crate::Error;
use crate::check::{Place, check};
use crate::clips::indent;
use crate::model::Kind;
use crate::op::Op;
use crate::patch::{Patch, quoted, seconds};
use crate::scan::{Sketch, Stage};

/// `delete_clip`. An element alone on its lines takes the lines with it.
pub(crate) fn delete(source: &str, sketch: &Sketch, id: &str) -> Result<Vec<Patch>, Error> {
    let (_, item) = sketch.find(id)?;
    let (start, end) = (item.element.tag.span.start, item.element.end);
    let line_start = source[..start].rfind('\n').map_or(0, |at| at + 1);
    let line_end = source[end..].find('\n').map_or(source.len(), |at| end + at + 1);
    let alone = source[line_start..start].trim().is_empty() && source[end..line_end].trim().is_empty();
    let before = &source[..line_start];
    let unbroken = before.strip_suffix('\n').map_or(before, |line| line.strip_suffix('\r').unwrap_or(line));
    let at = match (alone, source[..line_end].ends_with('\n')) {
        (false, _) => start..end,
        (true, true) => line_start..line_end,
        (true, false) => unbroken.len()..line_end,
    };
    Ok(vec![Patch { at, text: String::new() }])
}

fn name(sketch: &Sketch, kind: Kind) -> String {
    let word = match kind {
        Kind::Video => "video",
        Kind::Audio => "audio",
        Kind::Image => "image",
        Kind::Text => "text",
        Kind::Html => "html",
    };
    (1..).map(|n| format!("{word}-{n}")).find(|id| sketch.find(id).is_err()).unwrap_or_default()
}

fn markup(id: &str, op: &Op, (width, height): (u32, u32)) -> Result<String, Error> {
    let Op::AddClip { kind, src, start, duration, track, text, .. } = op else { return Ok(String::new()) };
    let frame =
        format!("data-x=\"0\" data-y=\"0\" data-w=\"{width}\" data-h=\"{height}\" data-rot=\"0\" data-fit=\"cover\"");
    let timing = format!(
        "data-start=\"{}\" data-duration=\"{}\" data-track-index=\"{track}\"",
        seconds(*start),
        seconds(*duration)
    );
    let file = || match src.as_deref().map(str::trim).filter(|src| !src.is_empty()) {
        Some(src) => Ok(quoted(src)),
        None => Err(Error::Invalid(format!("a {id} clip needs `src`"))),
    };
    let text = text.as_deref().unwrap_or_default();
    let escaped = text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
    Ok(match kind {
        Kind::Video => {
            format!("<video class=\"clap-fill\" data-clip=\"{id}\" src=\"{}\" {timing} {frame}></video>", file()?)
        }
        Kind::Audio => format!("<audio data-clip=\"{id}\" src=\"{}\" {timing}></audio>", file()?),
        Kind::Image => {
            format!("<img class=\"clap-fill\" data-clip=\"{id}\" src=\"{}\" alt=\"\" {timing} {frame}>", file()?)
        }
        Kind::Text => format!("<p class=\"clap-title\" data-clip=\"{id}\" {timing}>{escaped}</p>"),
        Kind::Html => format!("<div class=\"clap-layer\" data-clip=\"{id}\" {timing}>{text}</div>"),
    })
}

fn content_end(sketch: &Sketch, stage: &Stage) -> Option<usize> {
    let unit = &sketch.doc.units[stage.scene.index];
    let mut offsets = Vec::new();
    let mut at = unit.at;
    for line in &unit.body {
        offsets.push((at, line));
        at += line.len();
    }
    let note = offsets.iter().rposition(|(_, line)| line.trim_start().starts_with("<!--")).filter(|_| stage.noted);
    let content = &offsets[..note.unwrap_or(offsets.len())];
    let last = content.iter().rev().find(|(_, line)| !line.trim().is_empty())?;
    Some(last.0 + crate::lines::text(last.1).len())
}

/// `add_clip`. The element goes after the last clip of the scene, else after the last line of
/// the body that is not the presenter note, else at the top of the body.
pub(crate) fn add(source: &str, sketch: &Sketch, op: &Op) -> Result<Vec<Patch>, Error> {
    let Op::AddClip { scene, kind, start, duration, track, .. } = op else { return Ok(Vec::new()) };
    let stage = sketch.stage(*scene)?;
    let id = name(sketch, *kind);
    check(stage, &id, &Place { start: *start, duration: *duration, track: Some(*track) })?;
    let size = sketch.project.config().size;
    let element = markup(&id, op, (size.width, size.height))?;
    let eol = sketch.doc.eol;
    let after = stage.items.last().map(|item| (item.element.end, indent(source, item.element.tag.span.start)));
    let patch = match after.or_else(|| content_end(sketch, stage).map(|at| (at, ""))) {
        Some((at, indent)) => {
            Patch { at: at.min(source.len())..at.min(source.len()), text: format!("{eol}{indent}{element}") }
        }
        None => {
            let top = sketch.doc.units[stage.scene.index].at.min(source.len());
            Patch { at: top..top, text: format!("{element}{eol}") }
        }
    };
    Ok(vec![patch])
}
