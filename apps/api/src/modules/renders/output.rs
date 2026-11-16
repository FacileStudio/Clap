//! Serves a kept render, `outputs/<id>.mp4`, from a descriptor opened with `O_NOFOLLOW` and
//! checked to be a regular file, with one range of `Range: bytes=` answered `206`.

use std::path::Path;

use axum::body::Body;
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::Response;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio_util::io::ReaderStream;
use tronc::{Error, Result};

/// The bytes `header` asks for in a file of `size`: `None` for the whole file (no header, one
/// that is not a single `bytes=` range), `Some(Err)` when the range lies past the end.
pub fn range(header: Option<&str>, size: u64) -> Option<std::result::Result<(u64, u64), ()>> {
    let spec = header?.trim().strip_prefix("bytes=")?;
    if spec.contains(',') {
        return None;
    }
    let (first, last) = spec.split_once('-')?;
    let (first, last) = (first.trim(), last.trim());
    let bounds = match (first.parse::<u64>(), last.parse::<u64>()) {
        (Ok(start), Ok(end)) if start <= end => (start, end.min(size.saturating_sub(1))),
        (Ok(start), Err(_)) if last.is_empty() => (start, size.saturating_sub(1)),
        (Err(_), Ok(tail)) if first.is_empty() && tail > 0 => (size.saturating_sub(tail), size.saturating_sub(1)),
        _ => return None,
    };
    Some(match bounds.0 < size {
        true => Ok(bounds),
        false => Err(()),
    })
}

async fn open(path: &Path) -> Result<(tokio::fs::File, u64)> {
    let missing = |error: std::io::Error| Error::internal("a render file is missing", error);
    let flags = (rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::NONBLOCK).bits() as i32;
    let file = tokio::fs::OpenOptions::new().read(true).custom_flags(flags).open(path).await.map_err(missing)?;
    let metadata = file.metadata().await.map_err(missing)?;
    match metadata.is_file() {
        true => Ok((file, metadata.len())),
        false => Err(missing(std::io::Error::other("not a regular file"))),
    }
}

/// The file at `path` for a request with `headers`, with `Accept-Ranges`, the length, and
/// `206` with `Content-Range` for one satisfiable range or `416` for one past the end.
pub async fn send(path: &Path, headers: &HeaderMap) -> Result<Response> {
    let (mut file, size) = open(path).await?;
    let asked = headers.get(header::RANGE).and_then(|value| value.to_str().ok());
    let (status, start, end) = match range(asked, size) {
        None => (StatusCode::OK, 0, size.saturating_sub(1)),
        Some(Ok((start, end))) => (StatusCode::PARTIAL_CONTENT, start, end),
        Some(Err(())) => {
            let mut refused = Response::new(Body::empty());
            *refused.status_mut() = StatusCode::RANGE_NOT_SATISFIABLE;
            let unsatisfied =
                HeaderValue::from_str(&format!("bytes */{size}")).map_err(|e| Error::internal("range", e))?;
            refused.headers_mut().insert(header::CONTENT_RANGE, unsatisfied);
            return Ok(refused);
        }
    };
    let length = if size == 0 { 0 } else { end - start + 1 };
    file.seek(std::io::SeekFrom::Start(start)).await.map_err(|error| Error::internal("a render file", error))?;
    let mut response = Response::new(Body::from_stream(ReaderStream::new(file.take(length))));
    *response.status_mut() = status;
    let answer = response.headers_mut();
    answer.insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    answer.insert(header::CONTENT_LENGTH, HeaderValue::from(length));
    if status == StatusCode::PARTIAL_CONTENT {
        let span = format!("bytes {start}-{end}/{size}");
        answer.insert(header::CONTENT_RANGE, HeaderValue::from_str(&span).map_err(|e| Error::internal("range", e))?);
    }
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::range;

    #[test]
    fn one_range_is_read_the_rest_is_the_whole_file_and_past_the_end_is_refused() {
        assert_eq!(range(Some("bytes=4-11"), 100), Some(Ok((4, 11))));
        assert_eq!(range(Some("bytes=90-"), 100), Some(Ok((90, 99))));
        assert_eq!(range(Some("bytes=-10"), 100), Some(Ok((90, 99))));
        assert_eq!(range(Some("bytes=50-500"), 100), Some(Ok((50, 99))));
        assert_eq!(range(Some("bytes=100-"), 100), Some(Err(())));
        for whole in [None, Some("bytes=1-2,4-5"), Some("items=1-2"), Some("bytes=9-3"), Some("bytes=x")] {
            assert_eq!(range(whole, 100), None, "{whole:?}");
        }
    }
}
