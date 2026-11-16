//! The scene edits: they move whole lines and touch one frontmatter value.

use crate::Error;
use crate::check::EPSILON;
use crate::doc::{Doc, Front, Unit, lines_of};
use crate::lines::{key, value};
use crate::patch::seconds;
use crate::scan::Sketch;

fn length(duration: f64) -> Result<String, Error> {
    match duration.is_finite() && duration >= 0.001 {
        true => Ok(seconds(duration)),
        false => Err(Error::OutOfBounds(format!("a scene cannot last {duration} s"))),
    }
}

/// `set_scene_duration`. A scene may not end before one of its clips does.
pub(crate) fn set_duration(sketch: &Sketch, index: usize, duration: f64) -> Result<String, Error> {
    let stage = sketch.stage(index)?;
    let written = length(duration)?;
    let ends = stage.items.iter().filter(|item| item.element.attr("data-duration").is_some());
    if let Some(item) = ends.map(|item| &item.clip).find(|clip| clip.start + clip.duration > duration + EPSILON) {
        let end = item.start + item.duration;
        return Err(Error::OutOfBounds(format!(
            "scene {index} cannot last {duration} s: clip `{}` ends at {end} s",
            item.id
        )));
    }
    let mut doc = sketch.doc.clone();
    let eol = doc.eol;
    let unit = &mut doc.units[index];
    let front = unit.front.get_or_insert_with(|| Front { lines: Vec::new(), close: format!("---{eol}") });
    match front.lines.iter_mut().find(|line| key(line) == Some("duration")) {
        Some(line) => line.replace_range(value(line), &written),
        None => front.lines.push(format!("duration: {written}{eol}")),
    }
    doc.settle();
    Ok(doc.emit())
}

fn rehead(doc: &mut Doc, project: Vec<String>) {
    let eol = doc.eol;
    if let Some(first) = doc.units.first_mut() {
        first.give_project(project, eol);
    }
    doc.settle();
}

/// `reorder_scene`. The project keys of the headmatter stay at the top of the file.
pub(crate) fn reorder(sketch: &Sketch, from: usize, to: usize) -> Result<String, Error> {
    sketch.stage(from)?;
    sketch.stage(to)?;
    let mut doc = sketch.doc.clone();
    if from == to {
        return Ok(doc.emit());
    }
    let project = if from == 0 || to == 0 { doc.units[0].take_project() } else { Vec::new() };
    let unit = doc.units.remove(from);
    doc.units.insert(to, unit);
    rehead(&mut doc, project);
    Ok(doc.emit())
}

/// `delete_scene`.
pub(crate) fn delete(sketch: &Sketch, index: usize) -> Result<String, Error> {
    sketch.stage(index)?;
    if sketch.stages.len() == 1 {
        return Err(Error::Invalid("a project keeps at least one scene".to_owned()));
    }
    let mut doc = sketch.doc.clone();
    let project = if index == 0 { doc.units[0].take_project() } else { Vec::new() };
    doc.units.remove(index);
    rehead(&mut doc, project);
    Ok(doc.emit())
}

/// `add_scene`.
pub(crate) fn add(sketch: &Sketch, at: usize, duration: f64, body: &str) -> Result<String, Error> {
    if at > sketch.stages.len() {
        return Err(Error::UnknownScene(at));
    }
    let mut doc = sketch.doc.clone();
    let eol = doc.eol;
    let raw = body.trim_start().starts_with('<');
    let layout = raw.then(|| format!("layout: raw{eol}"));
    let lines = layout.into_iter().chain([format!("duration: {}{eol}", length(duration)?)]).collect();
    let front = Some(Front { lines, close: format!("---{eol}") });
    let unit = Unit { front, body: lines_of(body, eol), ..Unit::default() };
    let project = if at == 0 { doc.units[0].take_project() } else { Vec::new() };
    doc.units.insert(at, unit);
    rehead(&mut doc, project);
    Ok(doc.emit())
}
