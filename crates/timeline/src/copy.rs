//! The edits that copy: a clip into a scene, a scene after itself.

use std::collections::HashSet;

use crate::Error;
use crate::check::{EPSILON, Place, check};
use crate::doc::{Unit, lines_of};
use crate::patch::{Patch, attribute, number, patched};
use crate::place::append;
use crate::scan::{Located, Sketch};

/// Where `duplicate_clip` puts the copy; `None` keeps the clip's own scene or track.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Target {
    pub scene: Option<usize>,
    pub start: f64,
    pub track: Option<i64>,
}

fn taken(sketch: &Sketch) -> HashSet<String> {
    sketch.stages.iter().flat_map(|stage| stage.items.iter().map(|item| item.clip.id.clone())).collect()
}

/// The name of a copy of `id`: `<id>-copy`, else `<id>-copy-<n>` from 2, the first one not in `taken`.
fn copy_name(taken: &HashSet<String>, id: &str) -> String {
    let numbered = (2..).map(|n| format!("{id}-copy-{n}"));
    std::iter::once(format!("{id}-copy")).chain(numbered).find(|name| !taken.contains(name)).unwrap_or_default()
}

fn lasting(item: &Located, scene: f64, start: f64) -> f64 {
    match item.element.attr("data-duration") {
        Some(_) => item.clip.duration,
        None => (scene - start).max(EPSILON),
    }
}

/// `duplicate_clip`. The copy is the element as written, named by `data-clip`, at its new start
/// and track, written as a new line after the last clip of the target scene.
pub(crate) fn duplicate(source: &str, sketch: &Sketch, id: &str, target: Target) -> Result<Vec<Patch>, Error> {
    let (own, item) = sketch.find(id)?;
    let stage = sketch.stage(target.scene.unwrap_or(own.scene.index))?;
    let name = copy_name(&taken(sketch), id);
    let track = target.track.or(item.tracked.then_some(item.clip.track));
    let place = Place { start: target.start, duration: lasting(item, stage.scene.duration, target.start), track };
    check(stage, &name, &place)?;
    let element = &item.element;
    let mut patches = vec![attribute(element, "data-clip", &name)];
    patches.extend(number(source, element, "data-start", target.start));
    let lane = track.filter(|track| !item.tracked || *track != item.clip.track);
    patches.extend(lane.map(|track| attribute(element, "data-track-index", &track.to_string())));
    let copy = patched(source, element.tag.span.start..element.end, patches);
    Ok(vec![append(source, sketch, stage, &copy)])
}

/// `duplicate_scene`. The copy keeps the scene's frontmatter, without the project keys, and its
/// body, where every clip gets a new `data-clip` name the way `duplicate_clip` names a copy.
pub(crate) fn duplicate_scene(sketch: &Sketch, index: usize) -> Result<String, Error> {
    let stage = sketch.stage(index)?;
    let mut doc = sketch.doc.clone();
    let unit = &doc.units[index];
    let body = unit.body.concat();
    let mut names = taken(sketch);
    let renames = stage.items.iter().map(|item| {
        let name = copy_name(&names, &item.clip.id);
        names.insert(name.clone());
        let mut patch = attribute(&item.element, "data-clip", &name);
        patch.at = patch.at.start - unit.at..patch.at.end - unit.at;
        patch
    });
    let renamed = patched(&body, 0..body.len(), renames.collect());
    let mut copy = Unit { front: unit.front.clone(), body: lines_of(&renamed, doc.eol), ..Unit::default() };
    copy.take_project();
    doc.units.insert(index + 1, copy);
    doc.settle();
    Ok(doc.emit())
}
