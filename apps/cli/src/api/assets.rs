//! A project's assets: list, upload, download, delete.

use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::path::Path;

use serde::Deserialize;

use super::transfer::{OnBytes, Stream};
use super::{ApiError, Client, Payload};
use crate::url::{encode, encode_path};

/// An asset as the server lists it.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RemoteAsset {
    /// The name: the relative path the source writes, such as `assets/shot.mp4`.
    pub name: String,
    /// The size in bytes.
    #[serde(default)]
    pub size: u64,
    /// The SHA-256 of the content, in hex.
    #[serde(default)]
    pub sha256: String,
}

/// A file to upload as an asset.
pub struct Upload<'a> {
    /// The asset name, such as `assets/shot.mp4`.
    pub name: &'a str,
    /// The media type sent with the file. The server types it by its bytes anyway.
    pub kind: &'a str,
    /// The file on disk.
    pub path: &'a Path,
    /// A token no file holds, which separates the parts of the form.
    pub boundary: &'a str,
}

fn head(upload: &Upload<'_>) -> Vec<u8> {
    let (boundary, name, kind) = (upload.boundary, upload.name, upload.kind);
    let file_name = name.rsplit('/').next().unwrap_or(name).replace(['"', '\r', '\n'], "_");
    format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"name\"\r\n\r\n{name}\r\n\
         --{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{file_name}\"\r\n\
         Content-Type: {kind}\r\n\r\n"
    )
    .into_bytes()
}

fn route(id: &str, name: Option<&str>) -> String {
    match name {
        Some(name) => format!("/projects/{}/assets/{}", encode(id), encode_path(name)),
        None => format!("/projects/{}/assets", encode(id)),
    }
}

impl Client {
    /// `GET /api/projects/{id}/assets`.
    pub fn assets(&self, id: &str) -> Result<Vec<RemoteAsset>, ApiError> {
        self.list(&route(id, None))
    }

    /// `POST /api/projects/{id}/assets`: a multipart form with the asset's `name` and its
    /// `file`, read from disk as it is sent. The server replaces an asset of the same name.
    pub fn upload_asset(&self, id: &str, upload: &Upload<'_>, progress: OnBytes<'_>) -> Result<(), ApiError> {
        let unreadable = |error: std::io::Error| ApiError::Unreadable(format!("{}: {error}", upload.name));
        let file = File::open(upload.path).map_err(unreadable)?;
        let size = file.metadata().map_err(unreadable)?.len();
        let (head, tail) = (head(upload), format!("\r\n--{}--\r\n", upload.boundary).into_bytes());
        let length = head.len() as u64 + size + tail.len() as u64;
        let mut reader = Cursor::new(head).chain(file).chain(Cursor::new(tail));
        let kind = format!("multipart/form-data; boundary={}", upload.boundary);
        self.send(&route(id, None), Stream { kind: &kind, reader: &mut reader, length }, progress)
    }

    /// `GET /api/projects/{id}/assets/{name}`: the bytes of one asset, written into `sink`.
    pub fn download_asset(
        &self,
        id: &str,
        name: &str,
        sink: &mut dyn Write,
        progress: OnBytes<'_>,
    ) -> Result<u64, ApiError> {
        self.fetch(&route(id, Some(name)), sink, progress)
    }

    /// `DELETE /api/projects/{id}/assets/{name}`.
    pub fn delete_asset(&self, id: &str, name: &str) -> Result<(), ApiError> {
        self.call("DELETE", &route(id, Some(name)), Payload::Empty).map(drop)
    }
}
