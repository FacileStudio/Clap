//! The little URL handling the CLI needs: escaping, query values and project references.

/// Percent-decodes `text`. A malformed escape is kept as written.
pub fn decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        let escaped = text.get(index + 1..index + 3).and_then(|hex| u8::from_str_radix(hex, 16).ok());
        match (bytes[index], escaped) {
            (b'%', Some(byte)) => {
                out.push(byte);
                index += 3;
            }
            (byte, _) => {
                out.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Percent-encodes everything in `text` but letters, digits and `-._~`.
pub fn encode(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for byte in text.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => out.push(byte as char),
            other => out.push_str(&format!("%{other:02X}")),
        }
    }
    out
}

/// Percent-encodes each segment of a `/`-separated path.
pub fn encode_path(path: &str) -> String {
    path.split('/').map(encode).collect::<Vec<_>>().join("/")
}

/// The decoded value of `key` in the query of `target`, when it is there and not empty.
pub fn query(target: &str, key: &str) -> Option<String> {
    let (_, query) = target.split_once('?')?;
    query.split('&').find_map(|pair| {
        let (name, value) = pair.split_once('=')?;
        (name == key && !value.is_empty()).then(|| decode(&value.replace('+', " ")))
    })
}

/// Splits what `clap pull` was given: a bare project id, or the web URL of a project, which also
/// names its server.
pub fn project_reference(text: &str) -> (Option<String>, String) {
    let Some((scheme, rest)) = text.split_once("://") else { return (None, text.to_owned()) };
    let path = rest.split(['?', '#']).next().unwrap_or(rest);
    let (host, tail) = path.split_once('/').unwrap_or((path, ""));
    let id = tail.trim_end_matches('/').rsplit('/').next().unwrap_or("");
    (Some(format!("{scheme}://{host}")), decode(id))
}

/// A folder name for a project: its title in lowercase with dashes, else its id, else `project`.
pub fn slug(title: &str, id: &str) -> String {
    let dashed = |text: &str| -> String {
        let mapped: String =
            text.chars().map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '-' }).collect();
        mapped.split('-').filter(|part| !part.is_empty()).collect::<Vec<_>>().join("-")
    };
    [dashed(title), dashed(id)].into_iter().find(|name| !name.is_empty()).unwrap_or("project".to_owned())
}
