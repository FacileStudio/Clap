//! The stage attributes of a clip: its box, its fit, its opacity and volume, and how it comes in
//! and goes out.

use crate::model::{Geometry, Look};
use crate::tree::Element;

/// How a clip may come in (`data-enter`) or go out (`data-exit`).
pub const CLIP_TRANSITIONS: [&str; 11] = [
    "fade",
    "slide-left",
    "slide-right",
    "slide-up",
    "slide-down",
    "zoom-in",
    "zoom-out",
    "wipe-left",
    "wipe-right",
    "blur",
    "pop",
];

/// How a scene may come in (frontmatter `transition`).
pub const SCENE_TRANSITIONS: [&str; 7] =
    ["fade", "dip", "slide-left", "slide-right", "wipe-left", "wipe-right", "zoom"];

/// How a video or an image fills its box (`data-fit`).
pub const FITS: [&str; 3] = ["cover", "contain", "fill"];

/// The five box attributes, in the order they are written.
pub(crate) const BOX: [&str; 5] = ["data-x", "data-y", "data-w", "data-h", "data-rot"];

fn known(element: &Element, source: &str, name: &str, allowed: &[&str]) -> Option<String> {
    let value = element.value(source, name)?.trim();
    allowed.contains(&value).then(|| value.to_owned())
}

fn frame(element: &Element, source: &str, (width, height): (u32, u32)) -> Option<Geometry> {
    BOX.iter().find(|name| element.attr(name).is_some())?;
    let side = |name: &str, fallback: f64| element.number(source, name).unwrap_or(fallback);
    let w = element.number(source, "data-w").filter(|w| *w > 0.0).unwrap_or(f64::from(width));
    let h = element.number(source, "data-h").filter(|h| *h > 0.0).unwrap_or(f64::from(height));
    Some(Geometry { x: side("data-x", 0.0), y: side("data-y", 0.0), w, h, rot: side("data-rot", 0.0) })
}

/// The stage attributes `element` carries, on a stage of `size` pixels. A value the runtime
/// would ignore reads as missing.
pub(crate) fn look(element: &Element, source: &str, size: (u32, u32)) -> Look {
    let unit = |name: &str| element.number(source, name).filter(|value| (0.0..=1.0).contains(value));
    let seconds = |name: &str| element.number(source, name).filter(|value| *value > 0.0);
    Look {
        frame: frame(element, source, size),
        fit: known(element, source, "data-fit", &FITS),
        opacity: unit("data-opacity"),
        volume: unit("data-volume"),
        enter: known(element, source, "data-enter", &CLIP_TRANSITIONS),
        exit: known(element, source, "data-exit", &CLIP_TRANSITIONS),
        enter_duration: seconds("data-enter-duration"),
        exit_duration: seconds("data-exit-duration"),
    }
}
