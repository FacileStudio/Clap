//! A source read into scenes and located clips: the model, plus where each piece sits.

use std::collections::HashSet;
use std::ops::Range;

use crate::Error;
use crate::body::readable;
use crate::cut::cut;
use crate::doc::Doc;
use crate::geometry::look;
use crate::model::{Clip, Kind, Scene, Span};
use crate::tree::{Element, elements};

/// A clip and the element it was read from.
#[derive(Debug, Clone)]
pub(crate) struct Located {
    pub clip: Clip,
    pub element: Element,
    /// Whether the element carries `data-track-index`.
    pub tracked: bool,
}

/// One scene with its located clips.
#[derive(Debug, Clone)]
pub(crate) struct Stage {
    /// The scene, its `clips` left empty.
    pub scene: Scene,
    pub items: Vec<Located>,
    /// Whether the body ends with a presenter note.
    pub noted: bool,
}

/// A whole source, located.
#[derive(Debug, Clone)]
pub(crate) struct Sketch {
    pub project: scene::Project,
    pub doc: Doc,
    pub stages: Vec<Stage>,
}

impl Sketch {
    /// The clip with this id.
    pub(crate) fn find(&self, id: &str) -> Result<(&Stage, &Located), Error> {
        let all = self.stages.iter().flat_map(|stage| stage.items.iter().map(move |item| (stage, item)));
        all.into_iter().find(|(_, item)| item.clip.id == id).ok_or_else(|| Error::UnknownClip(id.to_owned()))
    }

    /// The scene at `index`.
    pub(crate) fn stage(&self, index: usize) -> Result<&Stage, Error> {
        self.stages.get(index).ok_or(Error::UnknownScene(index))
    }
}

fn kind(element: &Element, source: &str) -> Kind {
    let inner = &source[element.inner.clone()];
    match element.tag.name.as_str() {
        "video" => Kind::Video,
        "audio" => Kind::Audio,
        "img" => Kind::Image,
        _ if !inner.trim().is_empty() && !inner.contains('<') => Kind::Text,
        _ => Kind::Html,
    }
}

struct Reading<'a> {
    source: &'a str,
    size: (u32, u32),
    starts: Vec<usize>,
    seen: HashSet<String>,
}

impl Reading<'_> {
    fn span(&self, bytes: Range<usize>) -> Span {
        let line = |at: usize| self.starts.partition_point(|start| *start <= at);
        Span { first: line(bytes.start), last: line(bytes.end.saturating_sub(1).max(bytes.start)) }
    }

    fn id(&mut self, element: &Element, scene: usize, nth: usize) -> String {
        let named = ["data-clip", "id"].iter().find_map(|name| element.value(self.source, name));
        let mut id = named.map(str::trim).filter(|id| !id.is_empty() && !self.seen.contains(*id)).map(str::to_owned);
        let mut derived = format!("clip-{scene}-{nth}");
        while id.is_none() {
            id = Some(derived.clone()).filter(|id| !self.seen.contains(id));
            derived.push('x');
        }
        let id = id.unwrap_or_default();
        self.seen.insert(id.clone());
        id
    }

    fn clip(&mut self, element: Element, scene: &Scene, nth: usize) -> Located {
        let number = |name: &str| element.number(self.source, name);
        let start = number("data-start").unwrap_or(0.0);
        let clip = Clip {
            id: self.id(&element, scene.index, nth),
            kind: kind(&element, self.source),
            src: element.value(self.source, "src").map(str::to_owned),
            start,
            duration: number("data-duration").unwrap_or((scene.duration - start).max(0.0)),
            track: number("data-track-index").unwrap_or(0.0) as i64,
            media_in: number("data-in").unwrap_or(0.0).max(0.0),
            scene: scene.index,
            lines: self.span(element.tag.span.start..element.end),
            look: look(&element, self.source, self.size),
        };
        Located { clip, tracked: element.attr("data-track-index").is_some(), element }
    }
}

/// Reads `source` into located scenes and clips.
pub(crate) fn sketch(source: &str) -> Result<Sketch, Error> {
    let project = crate::parse(source).map_err(|errors| Error::Parse(errors.to_string()))?;
    let doc = cut(source);
    let agree = doc.units.len() == project.scenes().len();
    if !agree || doc.units.iter().zip(project.scenes()).any(|(unit, scene)| unit.line != scene.line) {
        return Err(Error::Parse("the timeline and the scene parser disagree on where the scenes are".to_owned()));
    }
    let starts = std::iter::once(0).chain(source.match_indices('\n').map(|(at, _)| at + 1)).collect();
    let size = (project.config().size.width, project.config().size.height);
    let mut reading = Reading { source, size, starts, seen: HashSet::new() };
    let mut stages = Vec::new();
    let mut start = 0.0;
    for (index, (unit, parsed)) in doc.units.iter().zip(project.scenes()).enumerate() {
        let scene = unit.scene(index, start, parsed);
        let text = readable(source, unit, parsed.layout == scene::Layout::Raw);
        let timed = elements(&text, unit.at).into_iter().filter(|element| element.attr("data-start").is_some());
        let items = timed.enumerate().map(|(nth, element)| reading.clip(element, &scene, nth)).collect();
        start += parsed.duration;
        stages.push(Stage { scene, items, noted: parsed.note.is_some() });
    }
    Ok(Sketch { project, doc, stages })
}
