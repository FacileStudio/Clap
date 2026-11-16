//! The project folder's files over HTTP, with byte ranges: a browser seeks a video by asking
//! for a part of its file.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Component, Path, PathBuf};

use tiny_http::{Header, Request, Response, StatusCode};

use crate::mime;
use crate::url::decode;

fn headers(pairs: &[(&str, &str)]) -> Vec<Header> {
    pairs.iter().filter_map(|(name, value)| Header::from_bytes(*name, *value).ok()).collect()
}

/// Answers `request` with `bytes`, never cached.
pub fn respond(request: Request, status: u16, kind: &str, bytes: Vec<u8>) {
    let mut response = Response::from_data(bytes).with_status_code(status);
    for header in headers(&[("Content-Type", kind), ("Cache-Control", "no-store")]) {
        response.add_header(header);
    }
    let _ = request.respond(response);
}

/// The file a request path names inside `dir`, or `None` for a path that leaves the folder
/// or names a hidden file.
pub fn resolve(dir: &Path, path: &str) -> Option<PathBuf> {
    let decoded = decode(path.trim_start_matches('/'));
    let relative = Path::new(&decoded);
    let visible =
        |part: Component<'_>| matches!(part, Component::Normal(name) if !name.to_string_lossy().starts_with('.'));
    (!decoded.is_empty() && relative.components().all(visible)).then(|| dir.join(relative))
}

/// The bytes `bytes=a-b` asks for in a file of `size` bytes: first and last, both included.
pub fn range(header: &str, size: u64) -> Option<(u64, u64)> {
    let (first, last) = header.trim().strip_prefix("bytes=")?.split_once('-')?;
    let last_byte = size.checked_sub(1)?;
    match (first.trim().parse::<u64>().ok(), last.trim().parse::<u64>().ok()) {
        (Some(first), last) if first <= last_byte => Some((first, last.unwrap_or(last_byte).min(last_byte))),
        (None, Some(suffix)) if suffix > 0 => Some((size.saturating_sub(suffix), last_byte)),
        _ => None,
    }
}

fn send(request: Request, mut file: File, kind: &str, size: u64) -> std::io::Result<()> {
    let asked =
        request.headers().iter().find(|header| header.field.equiv("Range")).map(|header| header.value.to_string());
    let part = asked.as_deref().and_then(|header| range(header, size));
    let (first, last) = part.unwrap_or((0, size.saturating_sub(1)));
    let length = if size == 0 { 0 } else { last - first + 1 };
    file.seek(SeekFrom::Start(first))?;
    let span = format!("bytes {first}-{last}/{size}");
    let mut sent = headers(&[("Content-Type", kind), ("Cache-Control", "no-store"), ("Accept-Ranges", "bytes")]);
    sent.extend(headers(&[("Content-Range", span.as_str())]).into_iter().filter(|_| part.is_some()));
    let status = StatusCode(if part.is_some() { 206 } else { 200 });
    request.respond(Response::new(status, sent, file.take(length), Some(length as usize), None))
}

/// Answers `request` with the file at `path`, whole or the range it asks for, or 404.
pub fn serve(dir: &Path, path: &str, request: Request) {
    let found = resolve(dir, path).filter(|file| file.is_file());
    let opened = found.and_then(|path| Some((File::open(&path).ok()?, path)));
    match opened.and_then(|(file, path)| Some((file.metadata().ok()?.len(), file, path))) {
        Some((size, file, path)) => drop(send(request, file, mime::of(&path), size)),
        None => respond(request, 404, "text/plain; charset=utf-8", b"not found".to_vec()),
    }
}
