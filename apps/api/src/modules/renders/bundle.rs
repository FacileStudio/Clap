use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::Serialize;
use time::format_description::well_known::Rfc3339;
use tronc::{Error, Result};
use uuid::Uuid;

use super::spec::Spec;
use super::types::RenderRow;
use crate::modules::assets::name;
use crate::modules::assets::service::Asset;

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
    /// `index.html`, every project file referenced by a relative path under [`MEDIA`], with
    /// Clap's media runtime, so a render draws what the preview shows.
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
    let html = clap_timeline::with_runtime(&scene::compile(project, &scene::Options::new().assets(&resolve)));
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
