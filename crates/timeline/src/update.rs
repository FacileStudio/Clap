//! `update_clip`: the stage attributes of one element, each set, kept or removed.

use crate::Error;
use crate::geometry::{CLIP_TRANSITIONS, FITS};
use crate::op::Update;
use crate::patch::{Patch, attribute, number};
use crate::scan::Sketch;
use crate::tree::Element;

/// One attribute value an update writes.
enum Value {
    Number(f64),
    Text(String),
}

/// What happens to one attribute: `None` removes it.
type Change = (&'static str, Option<Value>);

fn measure(name: &'static str, value: Option<f64>, valid: fn(f64) -> bool, rule: &str) -> Result<Change, Error> {
    match value {
        Some(number) if !number.is_finite() || !valid(number) => {
            Err(Error::Invalid(format!("`{name}` must be {rule}, not {number}")))
        }
        _ => Ok((name, value.map(Value::Number))),
    }
}

fn word(name: &'static str, value: Option<&String>, allowed: &[&str]) -> Result<Change, Error> {
    match value.map(|text| text.trim()) {
        Some(text) if !allowed.contains(&text) => {
            Err(Error::Invalid(format!("`{name}` must be one of {}, not `{text}`", allowed.join(", "))))
        }
        text => Ok((name, text.map(|text| Value::Text(text.to_owned())))),
    }
}

/// A numeric attribute: its name, the requested change, its rule and how the rule reads.
type Number = (&'static str, Option<Option<f64>>, fn(f64) -> bool, &'static str);

fn numbers(update: &Update) -> [Number; 9] {
    let any = |_| true;
    let positive = |value| value > 0.0;
    let unit = |value| (0.0..=1.0).contains(&value);
    [
        ("data-x", update.x, any, "a number"),
        ("data-y", update.y, any, "a number"),
        ("data-w", update.w, positive, "above 0"),
        ("data-h", update.h, positive, "above 0"),
        ("data-rot", update.rot, any, "a number"),
        ("data-opacity", update.opacity, unit, "from 0 to 1"),
        ("data-volume", update.volume, unit, "from 0 to 1"),
        ("data-enter-duration", update.enter_duration, positive, "above 0"),
        ("data-exit-duration", update.exit_duration, positive, "above 0"),
    ]
}

fn changes(update: &Update) -> Result<Vec<Change>, Error> {
    let mut found = Vec::new();
    for (name, value, valid, rule) in numbers(update) {
        if let Some(value) = value {
            found.push(measure(name, value, valid, rule)?);
        }
    }
    let words = [("data-fit", &update.fit, &FITS[..]), ("data-enter", &update.enter, &CLIP_TRANSITIONS[..])];
    for (name, value, allowed) in words.into_iter().chain([("data-exit", &update.exit, &CLIP_TRANSITIONS[..])]) {
        if let Some(value) = value {
            found.push(word(name, value.as_ref(), allowed)?);
        }
    }
    Ok(found)
}

/// Removes attribute `name` and the spaces before it, or nothing when the element has none.
fn removal(source: &str, element: &Element, name: &str) -> Option<Patch> {
    let attr = element.attr(name)?;
    let before = &source[element.tag.span.start..attr.span.start];
    let start = attr.span.start - (before.len() - before.trim_end().len());
    Some(Patch { at: start..attr.span.end, text: String::new() })
}

fn change(source: &str, element: &Element, (name, value): Change) -> Option<Patch> {
    match value {
        None => removal(source, element, name),
        Some(Value::Number(value)) => number(source, element, name, value),
        Some(Value::Text(text)) => {
            (element.value(source, name) != Some(text.as_str())).then(|| attribute(element, name, &text))
        }
    }
}

/// `update_clip`.
pub(crate) fn update(source: &str, sketch: &Sketch, id: &str, update: &Update) -> Result<Vec<Patch>, Error> {
    let (_, item) = sketch.find(id)?;
    let changes = changes(update)?;
    Ok(changes.into_iter().filter_map(|change| self::change(source, &item.element, change)).collect())
}
