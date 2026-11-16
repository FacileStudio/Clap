//! How the scene format reads one line. These mirror `facile-scene`'s splitter, which keeps its
//! own private: a file is cut here exactly where the parser cuts it.

/// The keys that belong to a scene; every other frontmatter key belongs to the project.
const SCENE_KEYS: [&str; 4] = ["layout", "steps", "duration", "class"];

/// `line` without its line ending.
pub(crate) fn text(line: &str) -> &str {
    let line = line.strip_suffix('\n').unwrap_or(line);
    line.strip_suffix('\r').unwrap_or(line)
}

/// Whether `line` splits two scenes.
pub(crate) fn separator(line: &str) -> bool {
    line.trim_end() == "---"
}

/// The key of a `key: value` line.
pub(crate) fn key(line: &str) -> Option<&str> {
    let line = text(line);
    let name = line.split(':').next().unwrap_or_default();
    let named = !name.is_empty() && name.bytes().all(|b| b.is_ascii_lowercase() || b == b'_' || b.is_ascii_digit());
    let ended = line[name.len()..].starts_with(':') && matches!(line.as_bytes().get(name.len() + 1), None | Some(b' '));
    (named && ended).then_some(name)
}

/// Whether `line` is a frontmatter line that travels with its scene.
pub(crate) fn scene_key(line: &str) -> bool {
    key(line).is_some_and(|name| SCENE_KEYS.contains(&name))
}

/// The character of the code fence `line` opens or closes.
pub(crate) fn fence(line: &str) -> Option<char> {
    let line = text(line);
    let inner = line.trim_start();
    let mark = inner.chars().next().filter(|c| *c == '`' || *c == '~')?;
    (line.len() - inner.len() < 4 && inner.chars().take_while(|c| *c == mark).count() >= 3).then_some(mark)
}

/// The fence state after `line`, given the state before it.
pub(crate) fn fenced(open: Option<char>, line: &str) -> Option<char> {
    match (open, fence(line)) {
        (None, mark) => mark,
        (Some(open), Some(mark)) if open == mark && text(line).trim().chars().all(|c| c == mark) => None,
        (open, _) => open,
    }
}

/// The byte range of the value of a `key: value` line, quotes included, comment excluded.
pub(crate) fn value(line: &str) -> std::ops::Range<usize> {
    let body = text(line);
    let after = body.find(':').map_or(body.len(), |colon| colon + 1);
    let start = after + (body[after..].len() - body[after..].trim_start().len());
    let rest = &body[start..];
    let quoted = rest.starts_with('"') || rest.starts_with('\'');
    let end = if quoted { rest.len() } else { rest.find(" #").unwrap_or(rest.len()) };
    start..start + rest[..end].trim_end().len()
}
