use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::Serialize;
use time::format_description::well_known::Rfc3339;
use tronc::{Error, Result};
use uuid::Uuid;

use super::spec::Spec;
use super::types::RenderRow;
use crate::modules::assets::service::Asset;
use crate::modules::assets::{files, name};

/// The folder of the bundle the media sit in, so no asset name can shadow `index.html`.
pub const MEDIA: &str = "media";

/// `DATA_DIR/renders`: every job directory.
pub fn root(data_dir: &Path) -> PathBuf {
    data_dir.join("renders")
}

/// `DATA_DIR/renders/<id>`: the directory of one job.
pub fn dir(data_dir: &Path, id: Uuid) -> PathBuf {
    root(data_dir).join(id.to_string())
}

/// A scene compiled for the worker: the document, and the media it names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    /// `index.html`, every project file referenced by a relative path under [`MEDIA`].
    pub html: String,
    /// Each asset name the document loads, with the digest of its bytes.
    pub media: BTreeMap<String, String>,
}

/// Compiles `project` for the worker. `412` when the source names a file the project does not
/// hold: a render with a hole in it is refused instead of produced.
pub fn plan(project: &scene::Project, assets: &[Asset]) -> Result<Plan> {
    let held: BTreeMap<&str, &str> = assets.iter().map(|asset| (asset.name.as_str(), asset.sha256.as_str())).collect();
    let (media, missing) = (RefCell::new(BTreeMap::new()), RefCell::new(BTreeSet::new()));
    let resolve = |path: &str| {
        let (name, fragment) = name::referenced(path);
        let Some(sha256) = held.get(name) else {
            missing.borrow_mut().insert(name.to_owned());
            return None;
        };
        media.borrow_mut().insert(name.to_owned(), (*sha256).to_owned());
        Some(format!("{MEDIA}/{name}{fragment}"))
    };
    let html = scene::compile(project, &scene::Options::new().assets(&resolve));
    let missing: Vec<String> = missing.into_inner().into_iter().collect();
    match missing.is_empty() {
        true => Ok(Plan { html, media: media.into_inner() }),
        false => Err(Error::failed(format!(
            "the source names files the project does not hold: {}. Upload them or remove the references",
            missing.join(", ")
        ))),
    }
}

/// `job.json`, exactly as the worker reads it.
#[derive(Debug, Serialize)]
pub struct JobFile {
    pub id: Uuid,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub range: Option<[f64; 2]>,
    pub created_at: String,
}

impl JobFile {
    /// The job of the render `row` with `spec`.
    pub fn of(row: &RenderRow, spec: &Spec) -> Self {
        JobFile {
            id: row.id,
            width: spec.width,
            height: spec.height,
            fps: spec.fps,
            range: spec.range.map(|(start, end)| [start, end]),
            created_at: row.created_at.format(&Rfc3339).unwrap_or_default(),
        }
    }
}

async fn linked(original: &Path, target: &Path) -> std::io::Result<()> {
    if let Some(parent) = target.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    if tokio::fs::hard_link(original, target).await.is_ok() {
        return Ok(());
    }
    tokio::fs::copy(original, target).await.map(|_| ())
}

/// Writes `value` to `path` so a reader sees the whole file or none: a temporary name, then a
/// rename.
pub async fn atomic(path: &Path, value: &[u8]) -> std::io::Result<()> {
    let temporary = path.with_extension("tmp");
    tokio::fs::write(&temporary, value).await?;
    tokio::fs::rename(&temporary, path).await
}

/// Writes the job directory of `job`: the bundle first, each media file a hard link to the
/// original (a copy when the volume refuses links), then `job.json` last and by rename, which
/// is what tells the worker the job is whole.
pub async fn write(data_dir: &Path, plan: &Plan, job: &JobFile) -> std::io::Result<()> {
    let bundle = dir(data_dir, job.id).join("bundle");
    tokio::fs::create_dir_all(&bundle).await?;
    for (name, sha256) in &plan.media {
        linked(&files::path(data_dir, sha256), &bundle.join(MEDIA).join(name)).await?;
    }
    tokio::fs::write(bundle.join("index.html"), &plan.html).await?;
    let encoded = serde_json::to_vec(job).map_err(std::io::Error::other)?;
    atomic(&dir(data_dir, job.id).join("job.json"), &encoded).await
}
