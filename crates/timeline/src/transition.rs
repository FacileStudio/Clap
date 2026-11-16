//! The two frontmatter keys Clap adds to a scene, `transition` and `transition_duration`.
//!
//! `facile-scene` refuses keys it does not define, so [`parse`] hands it the source with these
//! two lines folded into the scene's `class` as `clap-transition-<name>` and
//! `clap-transition-ms-<milliseconds>`. Every line keeps its number, and the compiled section
//! carries the transition for the runtime to read.

use scene::{Error as Problem, ErrorKind, Errors};

use crate::Error;
use crate::doc::Front;
use crate::geometry::SCENE_TRANSITIONS;
use crate::lines::{key, text, unquoted, value};
use crate::patch::seconds;
use crate::scan::Sketch;

fn duration(text: &str) -> Option<f64> {
    text.parse::<f64>().ok().filter(|n| n.is_finite() && *n > 0.0 && *n <= 86_400.0)
}

fn token(key: &str, text: &str) -> Result<String, ErrorKind> {
    let bad = |expected| ErrorKind::BadValue { key: key.to_owned(), expected };
    match key {
        "transition" => match SCENE_TRANSITIONS.contains(&text) {
            true => Ok(format!("clap-transition-{text}")),
            false => Err(bad("one of fade, dip, slide-left, slide-right, wipe-left, wipe-right, zoom")),
        },
        _ => duration(text)
            .map(|seconds| format!("clap-transition-ms-{}", (seconds * 1000.0).round()))
            .ok_or(bad("a number of seconds above 0")),
    }
}

fn ending(line: &str) -> &str {
    &line[text(line).len()..]
}

/// Folds the transition lines of one frontmatter block into its `class` line. `first` is the
/// source line of the block's first line.
fn fold(front: &mut Front, first: usize, problems: &mut Vec<Problem>) {
    let mut tokens = Vec::new();
    let mut touched = Vec::new();
    for (at, line) in front.lines.iter().enumerate() {
        let Some(name) = key(line).filter(|name| ["transition", "transition_duration"].contains(name)) else {
            continue;
        };
        let again = touched.iter().any(|seen: &usize| key(&front.lines[*seen]) == Some(name));
        let found =
            if again { Err(ErrorKind::DuplicateKey { key: name.to_owned() }) } else { token(name, unquoted(line)) };
        match found {
            Ok(token) => tokens.push(token),
            Err(kind) => problems.push(Problem { line: first + at, kind }),
        }
        touched.push(at);
    }
    let Some(&place) = touched.first() else { return };
    let class = front.lines.iter().position(|line| key(line) == Some("class"));
    let mut words: Vec<String> = class.map(|at| unquoted(&front.lines[at]).to_owned()).into_iter().collect();
    words.extend(tokens);
    let place = class.map_or(place, |at| at.min(place));
    for at in touched.into_iter().chain(class) {
        front.lines[at] = ending(&front.lines[at]).to_owned();
    }
    let merged = format!("class: {}{}", words.join(" ").trim(), ending(&front.lines[place]));
    front.lines[place] = merged;
}

/// Parses a Clap source with `facile-scene`: the scene keys `transition` and
/// `transition_duration` are checked here and reach the compiled sections as classes. Use it
/// wherever a Clap source is parsed, so the preview and every render see the transitions.
pub fn parse(source: &str) -> Result<scene::Project, Errors> {
    let mut doc = crate::cut::cut(source);
    let mut problems = Vec::new();
    let mut line = 1;
    for unit in &mut doc.units {
        let opened = usize::from(unit.sep.is_some());
        if let Some(front) = unit.front.as_mut().filter(|front| !front.close.is_empty()) {
            fold(front, line + opened, &mut problems);
        }
        line = unit.line + unit.body.len();
    }
    let parsed = scene::parse(&doc.emit());
    if problems.is_empty() {
        return parsed;
    }
    problems.extend(parsed.err().map(|errors| errors.0).unwrap_or_default());
    problems.sort_by_key(|problem| problem.line);
    Err(Errors(problems))
}

fn put(front: &mut Front, name: &str, written: &str, eol: &str) {
    match front.lines.iter_mut().find(|line| key(line) == Some(name)) {
        Some(line) => line.replace_range(value(line), written),
        None => front.lines.push(format!("{name}: {written}{eol}")),
    }
}

fn checked(name: Option<&str>, length: Option<f64>) -> Result<(), Error> {
    if let Some(name) = name.filter(|name| !SCENE_TRANSITIONS.contains(name)) {
        return Err(Error::Invalid(format!("`{name}` is not one of {}", SCENE_TRANSITIONS.join(", "))));
    }
    if let Some(bad) = length.filter(|seconds| duration(&seconds.to_string()).is_none()) {
        return Err(Error::Invalid(format!("a transition cannot last {bad} s")));
    }
    match (name, length) {
        (None, Some(_)) => Err(Error::Invalid("a transition duration needs a transition".to_owned())),
        _ => Ok(()),
    }
}

/// `set_scene_transition`.
pub(crate) fn set(
    sketch: &Sketch,
    index: usize,
    name: Option<&str>,
    length: Option<Option<f64>>,
) -> Result<String, Error> {
    sketch.stage(index)?;
    checked(name, length.flatten())?;
    let mut doc = sketch.doc.clone();
    let eol = doc.eol;
    let unit = &mut doc.units[index];
    let existed = unit.front.is_some();
    let front = unit.front.get_or_insert_with(|| Front { lines: Vec::new(), close: format!("---{eol}") });
    let had = front.lines.len();
    let gone: &[&str] = match (name, length) {
        (None, _) => &["transition", "transition_duration"],
        (Some(_), Some(None)) => &["transition_duration"],
        _ => &[],
    };
    front.lines.retain(|line| !key(line).is_some_and(|key| gone.contains(&key)));
    if let Some(name) = name {
        put(front, "transition", name, eol);
    }
    if let (Some(_), Some(Some(length))) = (name, length) {
        put(front, "transition_duration", &seconds(length), eol);
    }
    if front.lines.is_empty() && (had > 0 || !existed) {
        unit.front = None;
    }
    doc.settle();
    Ok(doc.emit())
}
