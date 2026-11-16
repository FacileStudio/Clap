//! Bodies too large to hold in memory: a file sent as it is read, an answer written as it comes.

use std::io::{self, Read, Write};

use ureq::SendBody;

use super::{ApiError, Client, open, patient};

/// Told how far a transfer is: bytes done, bytes in all (0 when the server did not say).
pub type OnBytes<'a> = &'a mut dyn FnMut(u64, u64);

/// A body that is read while it is sent.
pub(crate) struct Stream<'a> {
    /// The `Content-Type` of the body.
    pub kind: &'a str,
    /// Where the bytes come from.
    pub reader: &'a mut dyn Read,
    /// How many bytes `reader` gives.
    pub length: u64,
}

struct Counted<'a, 'b> {
    reader: &'a mut dyn Read,
    done: u64,
    total: u64,
    progress: OnBytes<'b>,
}

impl Read for Counted<'_, '_> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let read = self.reader.read(buffer)?;
        self.done += read as u64;
        (self.progress)(self.done, self.total);
        Ok(read)
    }
}

fn unreachable(error: impl ToString) -> ApiError {
    ApiError::Unreachable(error.to_string())
}

impl Client {
    /// Sends `stream` to `path` without a time limit, telling `progress` as bytes leave.
    pub(crate) fn send(&self, path: &str, stream: Stream<'_>, progress: OnBytes<'_>) -> Result<(), ApiError> {
        let mut counted = Counted { reader: stream.reader, done: 0, total: stream.length, progress };
        let request = self
            .request("POST", path)
            .header("Content-Type", stream.kind)
            .header("Content-Length", stream.length.to_string())
            .body(SendBody::from_reader(&mut counted))
            .map_err(unreachable)?;
        open(&patient(), request).map(drop)
    }

    /// Writes the answer of `GET path` into `sink` as it arrives. Answers the bytes written.
    pub(crate) fn fetch(&self, path: &str, sink: &mut dyn Write, progress: OnBytes<'_>) -> Result<u64, ApiError> {
        let request = self.request("GET", path).header("Accept", "*/*").body(()).map_err(unreachable)?;
        let mut response = open(&patient(), request)?;
        let total = response.body().content_length().unwrap_or(0);
        let mut counted = Counted { reader: &mut response.body_mut().as_reader(), done: 0, total, progress };
        io::copy(&mut counted, sink).map_err(unreachable)
    }
}
