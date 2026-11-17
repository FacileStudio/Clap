//! `paste_clip`: an element copied as text, written into a scene as a new clip.

use crate::Error;
use crate::check::{EPSILON, Place, check};
use crate::copy::{copy_name, taken};
use crate::media::{asset_names, valid_asset_name};
use crate::patch::{Patch, attribute, number, patched};
use crate::place::append;
use crate::scan::Sketch;
use crate::tree::{Element, elements};

/// Where `paste_clip` puts the element; `track` `None` keeps the element's own.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Paste<'a> {
    pub scene: usize,
    pub start: f64,
    pub track: Option<i64>,
    pub element: &'a str,
}

fn single(text: &str) -> Result<Element, Error> {
    match elements(text, 0).into_iter().next() {
        Some(element) if element.tag.span.start == 0 && element.end == text.len() => Ok(element),
        _ => Err(Error::Invalid("`element` must be one HTML element, from its start tag to its end tag".to_owned())),
    }
}

fn files(text: &str) -> Result<(), Error> {
    match asset_names(text).into_iter().find(|name| !valid_asset_name(name)) {
        Some(bad) => {
            Err(Error::Invalid(format!("`{bad}` is not a project file: a pasted clip loads project files only")))
        }
        None => Ok(()),
    }
}

/// `paste_clip`. The element, trimmed, gets a `data-clip` named as `duplicate_clip` names a copy
/// of its own `data-clip` or `id` (`clip` without either), its new start and, when given, its
/// new track, and goes after the last clip of the scene. It passes the placement checks of
/// `add_clip`, and each file it loads must be named like an asset.
pub(crate) fn paste(source: &str, sketch: &Sketch, paste: Paste) -> Result<Vec<Patch>, Error> {
    let text = paste.element.trim();
    let element = single(text)?;
    files(text)?;
    let stage = sketch.stage(paste.scene)?;
    let named = ["data-clip", "id"].iter().find_map(|name| element.value(text, name)).map(str::trim);
    let name = copy_name(&taken(sketch), named.filter(|id| !id.is_empty()).unwrap_or("clip"));
    let own = element.number(text, "data-track-index").map(|track| track as i64);
    let duration = element.number(text, "data-duration").unwrap_or((stage.scene.duration - paste.start).max(EPSILON));
    check(stage, &name, &Place { start: paste.start, duration, track: paste.track.or(own) })?;
    let mut patches = vec![attribute(&element, "data-clip", &name)];
    patches.extend(number(text, &element, "data-start", paste.start));
    let lane = paste.track.filter(|track| own != Some(*track));
    patches.extend(lane.map(|track| attribute(&element, "data-track-index", &track.to_string())));
    let copy = patched(text, 0..text.len(), patches);
    Ok(vec![append(source, sketch, stage, &copy)])
}
