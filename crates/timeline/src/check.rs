//! The two rules every clip placement passes: stay inside the scene, and keep a track to one
//! clip at a time.

use crate::Error;
use crate::scan::Stage;

/// Two times closer than this are the same time.
pub(crate) const EPSILON: f64 = 1e-6;

/// Where an op wants a clip.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Place {
    pub start: f64,
    pub duration: f64,
    /// The track, when the clip will carry `data-track-index`.
    pub track: Option<i64>,
}

fn bounds(stage: &Stage, id: &str, place: &Place) -> Result<(), Error> {
    let scene = &stage.scene;
    if !place.start.is_finite() || !place.duration.is_finite() {
        return Err(Error::Invalid(format!("clip `{id}`: a time must be a finite number")));
    }
    if place.start < 0.0 {
        return Err(Error::OutOfBounds(format!("clip `{id}` would start at {} s, before its scene", place.start)));
    }
    if place.duration <= 0.0 {
        return Err(Error::OutOfBounds(format!("clip `{id}` would last {} s", place.duration)));
    }
    let end = place.start + place.duration;
    if scene.fixed_duration && end > scene.duration + EPSILON {
        let (index, length) = (scene.index, scene.duration);
        return Err(Error::OutOfBounds(format!(
            "clip `{id}` would end at {end} s, past the {length} s of scene {index}"
        )));
    }
    Ok(())
}

/// Checks `place` for the clip `id` of `stage` against the scene bounds and the overlap policy:
/// two clips that both carry `data-track-index` may not share a track and a moment.
pub(crate) fn check(stage: &Stage, id: &str, place: &Place) -> Result<(), Error> {
    bounds(stage, id, place)?;
    let Some(track) = place.track else { return Ok(()) };
    let end = place.start + place.duration;
    let rivals = stage.items.iter().filter(|item| item.tracked && item.clip.track == track && item.clip.id != id);
    let hit = rivals
        .map(|item| &item.clip)
        .find(|clip| place.start < clip.start + clip.duration - EPSILON && clip.start < end - EPSILON);
    match hit {
        Some(clip) => Err(Error::Overlap { clip: id.to_owned(), with: clip.id.clone(), track }),
        None => Ok(()),
    }
}
