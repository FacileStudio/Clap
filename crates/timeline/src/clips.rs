//! The clip edits that change attributes: move, trim, split.

use crate::Error;
use crate::check::{EPSILON, Place, check};
use crate::model::Kind;
use crate::patch::{Patch, attribute, number, patched};
use crate::scan::{Located, Sketch, Stage};

fn rest(stage: &Stage, item: &Located, start: f64) -> f64 {
    match item.element.attr("data-duration") {
        Some(_) => item.clip.duration,
        None => (stage.scene.duration - start).max(EPSILON),
    }
}

/// `move_clip`.
pub(crate) fn shift(
    source: &str,
    sketch: &Sketch,
    id: &str,
    start: f64,
    track: Option<i64>,
) -> Result<Vec<Patch>, Error> {
    let (stage, item) = sketch.find(id)?;
    let track = track.or(item.tracked.then_some(item.clip.track));
    check(stage, id, &Place { start, duration: rest(stage, item, start), track })?;
    let moved = number(source, &item.element, "data-start", start);
    let lane = track.filter(|track| !item.tracked || *track != item.clip.track);
    let lane = lane.map(|track| attribute(&item.element, "data-track-index", &track.to_string()));
    Ok(moved.into_iter().chain(lane).collect())
}

/// What `trim_clip` may change.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Trim {
    pub start: Option<f64>,
    pub duration: Option<f64>,
    pub media_in: Option<f64>,
}

/// `trim_clip`.
pub(crate) fn trim(source: &str, sketch: &Sketch, id: &str, trim: Trim) -> Result<Vec<Patch>, Error> {
    let (stage, item) = sketch.find(id)?;
    let start = trim.start.unwrap_or(item.clip.start);
    let duration = trim.duration.unwrap_or_else(|| rest(stage, item, start));
    check(stage, id, &Place { start, duration, track: item.tracked.then_some(item.clip.track) })?;
    if trim.media_in.is_some_and(|offset| !offset.is_finite() || offset < 0.0) {
        return Err(Error::OutOfBounds(format!("clip `{id}`: the media offset must be 0 or more")));
    }
    let set = |name: &str, value: Option<f64>| value.and_then(|value| number(source, &item.element, name, value));
    let patches = [set("data-start", trim.start), set("data-duration", trim.duration), set("data-in", trim.media_in)];
    Ok(patches.into_iter().flatten().collect())
}

fn fresh(sketch: &Sketch, id: &str) -> String {
    let taken = |name: &str| sketch.find(name).is_ok();
    let numbered = (2..).map(|n| format!("{id}-b{n}"));
    std::iter::once(format!("{id}-b")).chain(numbered).find(|name| !taken(name)).unwrap_or_default()
}

fn second_half(source: &str, sketch: &Sketch, item: &Located, at: f64) -> String {
    let (element, clip) = (&item.element, &item.clip);
    let named = if element.attr("data-clip").is_none() && element.attr("id").is_some() { "id" } else { "data-clip" };
    let mut patches = vec![attribute(element, named, &fresh(sketch, &clip.id))];
    patches.extend(number(source, element, "data-start", at));
    patches.extend(number(source, element, "data-duration", clip.start + clip.duration - at));
    if matches!(clip.kind, Kind::Video | Kind::Audio) {
        patches.extend(number(source, element, "data-in", clip.media_in + at - clip.start));
    }
    patched(source, element.tag.span.start..element.end, patches)
}

/// The spaces and tabs `at`'s line starts with.
pub(crate) fn indent(source: &str, at: usize) -> &str {
    let line = &source[source[..at].rfind('\n').map_or(0, |end| end + 1)..];
    &line[..line.len() - line.trim_start_matches([' ', '\t']).len()]
}

/// `split_clip`.
pub(crate) fn split(source: &str, sketch: &Sketch, id: &str, at: f64) -> Result<Vec<Patch>, Error> {
    let (_, item) = sketch.find(id)?;
    let (start, end) = (item.clip.start, item.clip.start + item.clip.duration);
    if !at.is_finite() || at <= start + EPSILON || at >= end - EPSILON {
        return Err(Error::OutOfBounds(format!("clip `{id}` runs from {start} s to {end} s: {at} s does not cut it")));
    }
    let element = &item.element;
    let copy = second_half(source, sketch, item, at);
    let text = format!("{}{}{copy}", sketch.doc.eol, indent(source, element.tag.span.start));
    let first = number(source, element, "data-duration", at - start);
    Ok(first.into_iter().chain([Patch { at: element.end..element.end, text }]).collect())
}
