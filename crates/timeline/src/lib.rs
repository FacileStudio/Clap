//! The timeline of a `clap.md`: its scenes and clips as a model, and edits written back as
//! minimal text patches.
//!
//! [`read`] answers the model the editor's timeline draws. [`apply`] takes the source and a
//! list of [`Op`]s and answers the new source: only the attribute values and the lines an op
//! concerns change, and everything else survives byte for byte. A drag in the browser and an
//! agent edit go through the same ops, so both leave the same kind of diff.
//!
//! ```
//! let source = "---\nlayout: raw\nduration: 4\n---\n<p id=\"a\" data-start=\"0\" data-duration=\"2\">Hi</p>\n";
//! let timeline = clap_timeline::read(source).unwrap();
//! assert_eq!(timeline.scenes[0].clips[0].id, "a");
//! let op = clap_timeline::Op::MoveClip { clip: "a".into(), start: 1.5, track: None };
//! let moved = clap_timeline::apply(source, &[op]).unwrap();
//! assert_eq!(moved, source.replace("data-start=\"0\"", "data-start=\"1.5\""));
//! ```
#![deny(unused, dead_code, unsafe_code, missing_docs)]

mod body;
mod check;
mod clips;
mod copy;
mod cut;
mod doc;
mod error;
mod geometry;
mod guide;
mod html;
mod lines;
mod media;
mod model;
mod op;
mod paste;
mod patch;
mod place;
mod references;
mod runtime;
mod scan;
mod scenes;
mod starter;
mod text;
mod transition;
mod tree;
mod update;

pub use error::Error;
pub use geometry::{CLIP_TRANSITIONS, FITS, SCENE_TRANSITIONS};
pub use guide::{CLAP_GUIDE, format_guide};
pub use media::{MAX_ASSET_NAME, asset_names, asset_reference, valid_asset_name};
pub use model::{Clip, Geometry, Kind, Look, Scene, Span, Timeline};
pub use op::{Op, Update};
pub use runtime::{RUNTIME_JS, STAGE_CSS, with_runtime};
pub use starter::{starter, starter_language};
pub use transition::parse;

use clips::Trim;
use copy::Target;
use paste::Paste;
use scan::{Sketch, sketch};

/// Reads a source into its scenes and clips.
pub fn read(source: &str) -> Result<Timeline, Error> {
    let sketch = sketch(source)?;
    let config = sketch.project.config();
    let scenes = sketch.stages.iter().map(|stage| {
        let clips = stage.items.iter().map(|item| item.clip.clone()).collect();
        Scene { clips, ..stage.scene.clone() }
    });
    Ok(Timeline {
        title: config.title.clone(),
        width: config.size.width,
        height: config.size.height,
        duration: sketch.project.duration(),
        scenes: scenes.collect(),
    })
}

fn edit(source: &str, sketch: &Sketch, op: &Op) -> Result<String, Error> {
    let patches = match op {
        Op::MoveClip { clip, start, track } => clips::shift(source, sketch, clip, *start, *track)?,
        Op::TrimClip { clip, start, duration, media_in } => {
            clips::trim(source, sketch, clip, Trim { start: *start, duration: *duration, media_in: *media_in })?
        }
        Op::SplitClip { clip, at } => clips::split(source, sketch, clip, *at)?,
        Op::DeleteClip { clip } => place::delete(source, sketch, clip)?,
        Op::AddClip { .. } => place::add(source, sketch, op)?,
        Op::DuplicateClip { clip, scene, start, track } => {
            copy::duplicate(source, sketch, clip, Target { scene: *scene, start: *start, track: *track })?
        }
        Op::SetSceneDuration { scene, duration } => return scenes::set_duration(sketch, *scene, *duration),
        Op::ReorderScene { scene, to } => return scenes::reorder(sketch, *scene, *to),
        Op::DeleteScene { scene } => return scenes::delete(sketch, *scene),
        Op::AddScene { at, duration, body } => return scenes::add(sketch, *at, *duration, body),
        Op::DuplicateScene { scene } => return copy::duplicate_scene(sketch, *scene),
        Op::UpdateClip { clip, update } => update::update(source, sketch, clip, update)?,
        Op::RenameMedia { from, to } => media::rename(source, sketch, from, to)?,
        Op::PasteClip { scene, start, track, element } => {
            paste::paste(source, sketch, Paste { scene: *scene, start: *start, track: *track, element })?
        }
        Op::SetClipText { clip, text } => text::set(source, sketch, clip, text)?,
        Op::SetSceneTransition { scene, transition, duration } => {
            return transition::set(sketch, *scene, transition.as_deref(), *duration);
        }
    };
    Ok(patch::patched(source, 0..source.len(), patches))
}

/// Applies `ops` in order and answers the new source. Each op sees the result of the one
/// before. Nothing is answered unless every op applies and every result is a valid project.
pub fn apply(source: &str, ops: &[Op]) -> Result<String, Error> {
    let mut text = source.to_owned();
    for op in ops {
        let before = sketch(&text)?;
        let after = edit(&text, &before, op)?;
        let grown = usize::from(matches!(op, Op::AddScene { .. } | Op::DuplicateScene { .. }));
        let shrunk = usize::from(matches!(op, Op::DeleteScene { .. }));
        let scenes = parse(&after).map_err(|errors| Error::InvalidResult(errors.to_string()))?.scenes().len();
        if scenes + shrunk != before.stages.len() + grown {
            return Err(Error::InvalidResult(format!(
                "the edit left {scenes} scenes: a body may not hold a `---` line"
            )));
        }
        text = after;
    }
    Ok(text)
}
