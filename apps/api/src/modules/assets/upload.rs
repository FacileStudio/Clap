use std::path::Path;

use sha2::{Digest, Sha256};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tronc::{Error, Result};

use super::files;
use super::sniff::{HEAD_BYTES, sniff};
use super::temporary::Temporary;

/// One upload, fully received on disk.
#[derive(Debug)]
pub struct Stored {
    /// The bytes, under a temporary name.
    pub file: Temporary,
    /// Their lowercase hex SHA-256.
    pub sha256: String,
    /// How many there are.
    pub size: u64,
    /// The media type their content declares.
    pub mime: &'static str,
}

/// Writes an upload to disk chunk by chunk: the bytes are hashed and counted as they pass, held
/// to `max`, and typed from their first [`HEAD_BYTES`]. Nothing but that head is kept in memory.
pub struct Sink {
    file: File,
    temporary: Temporary,
    hasher: Sha256,
    head: Vec<u8>,
    mime: Option<&'static str>,
    size: u64,
    max: u64,
}

fn unsupported() -> Error {
    Error::invalid("unsupported file: an asset is a video, a sound, an image or a font")
}

fn io(error: std::io::Error) -> Error {
    Error::internal("failed to store an upload", error)
}

impl Sink {
    /// Opens a temporary file under `data_dir` for an upload of at most `max` bytes.
    pub async fn open(data_dir: &Path, max: u64) -> Result<Self> {
        let dir = files::incoming(data_dir);
        tokio::fs::create_dir_all(&dir).await.map_err(io)?;
        let path = dir.join(uuid::Uuid::new_v4().to_string());
        let file = File::create(&path).await.map_err(io)?;
        let temporary = Temporary::at(path);
        Ok(Sink { file, temporary, hasher: Sha256::new(), head: Vec::new(), mime: None, size: 0, max })
    }

    fn typed(&mut self, last: bool) -> Result<()> {
        if self.mime.is_some() || (!last && self.head.len() < HEAD_BYTES) {
            return Ok(());
        }
        self.mime = Some(sniff(&self.head).ok_or_else(unsupported)?);
        Ok(())
    }

    /// Takes the next chunk: `413` past the cap, `400` as soon as the head is no media file.
    pub async fn write(&mut self, chunk: &[u8]) -> Result<()> {
        self.size += chunk.len() as u64;
        if self.size > self.max {
            return Err(Error::too_large(format!("an asset is at most {} MiB", self.max >> 20)));
        }
        let room = HEAD_BYTES.saturating_sub(self.head.len()).min(chunk.len());
        self.head.extend_from_slice(&chunk[..room]);
        self.typed(false)?;
        self.hasher.update(chunk);
        self.file.write_all(chunk).await.map_err(io)
    }

    /// Ends the upload: the file is flushed and described.
    pub async fn finish(mut self) -> Result<Stored> {
        self.typed(true)?;
        self.file.flush().await.map_err(io)?;
        self.file.sync_all().await.map_err(io)?;
        let sha256 = files::hex(&self.hasher.finalize());
        Ok(Stored { file: self.temporary, sha256, size: self.size, mime: self.mime.ok_or_else(unsupported)? })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir() -> std::path::PathBuf {
        std::env::temp_dir().join(format!("clap-upload-{}", uuid::Uuid::new_v4()))
    }

    #[tokio::test]
    async fn an_upload_is_hashed_counted_typed_capped_and_removed_unless_kept() {
        let dir = dir();
        let mut sink = Sink::open(&dir, 64).await.unwrap();
        sink.write(b"GIF89a").await.unwrap();
        sink.write(b"-rest").await.unwrap();
        let stored = sink.finish().await.unwrap();
        assert_eq!((stored.size, stored.mime), (11, "image/gif"));
        assert_eq!(stored.sha256.len(), 64);
        let path = stored.file.path().to_owned();
        assert_eq!(std::fs::read(&path).unwrap(), b"GIF89a-rest");
        drop(stored);
        assert!(!path.exists());
        let mut big = Sink::open(&dir, 8).await.unwrap();
        assert_eq!(big.write(b"GIF89a-too-long").await.unwrap_err().code, tronc::Code::ResourceExhausted);
        drop(big);
        let mut html = Sink::open(&dir, 1 << 20).await.unwrap();
        let page = [b"<!doctype html>".as_slice(), &[b' '; HEAD_BYTES]].concat();
        assert_eq!(html.write(&page).await.unwrap_err().code, tronc::Code::InvalidArgument);
        drop(html);
        let empty = Sink::open(&dir, 8).await.unwrap().finish().await.unwrap_err();
        assert_eq!(empty.code, tronc::Code::InvalidArgument);
        assert_eq!(std::fs::read_dir(files::incoming(&dir)).unwrap().count(), 0);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
