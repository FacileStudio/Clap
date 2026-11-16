use axum::extract::Multipart;
use axum::extract::multipart::{Field, MultipartError};
use tronc::{Error, Result};

use super::name::MAX_NAME;
use super::upload::{Sink, Stored};
use crate::state::AppState;

/// What a multipart upload carried.
#[derive(Debug, Default)]
pub struct Intake {
    /// The `name` part, when the upload names its asset.
    pub name: Option<String>,
    /// The `file` part: the file name the browser sent, and the bytes on disk.
    pub file: Option<(String, Stored)>,
}

fn invalid(error: MultipartError) -> Error {
    Error::invalid(error.body_text())
}

async fn short(mut field: Field<'_>) -> Result<String> {
    let mut text = Vec::new();
    while let Some(chunk) = field.chunk().await.map_err(invalid)? {
        text.extend_from_slice(&chunk);
        if text.len() > MAX_NAME * 4 {
            return Err(Error::invalid("the `name` part is too long"));
        }
    }
    String::from_utf8(text).map_err(|_| Error::invalid("the `name` part is not UTF-8"))
}

async fn stored(state: &AppState, mut field: Field<'_>) -> Result<Stored> {
    let mut sink = Sink::open(&state.data_dir, state.limits.asset).await?;
    while let Some(chunk) = field.chunk().await.map_err(invalid)? {
        sink.write(&chunk).await?;
    }
    sink.finish().await
}

/// Reads the parts of an upload. The `file` part streams to disk, never through memory; a second
/// `file` part replaces the first, and any other part is skipped.
pub async fn read(state: &AppState, mut multipart: Multipart) -> Result<Intake> {
    let mut intake = Intake::default();
    while let Some(field) = multipart.next_field().await.map_err(invalid)? {
        match field.name() {
            Some("name") => intake.name = Some(short(field).await?),
            Some("file") => {
                let file_name = field.file_name().unwrap_or_default().to_owned();
                intake.file = Some((file_name, stored(state, field).await?));
            }
            _ => {}
        }
    }
    Ok(intake)
}
