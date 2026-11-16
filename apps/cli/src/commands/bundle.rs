//! What the renderer loads: a folder holding the compiled `index.html` and the files it names.

use std::cell::RefCell;
use std::fs;
use std::path::Path;

use scene::{Options, Project};
use tempfile::TempDir;

use crate::failure::Outcome;
use crate::project::Folder;
use crate::url::decode;

fn place(from: &Path, to: &Path) -> std::io::Result<()> {
    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent)?;
    }
    if to.exists() {
        return Ok(());
    }
    fs::hard_link(from, to).or_else(|_| fs::copy(from, to).map(drop))
}

/// Builds the bundle of `project` in a hidden folder inside the project folder, removed when
/// the answer is dropped. Each file the source names is a hard link to the original, or a
/// copy where the disk refuses links, so the renderer reads nothing outside the bundle.
pub fn build(folder: &Folder, project: &Project) -> Outcome<TempDir> {
    let bundle = tempfile::Builder::new().prefix(".clap-render-").tempdir_in(&folder.dir)?;
    let named = RefCell::new(Vec::new());
    let note = |path: &str| -> Option<String> {
        named.borrow_mut().push(path.to_owned());
        None
    };
    let html = scene::compile(project, &Options::new().assets(&note));
    for path in named.into_inner() {
        let bare = decode(path.split(['?', '#']).next().unwrap_or(&path));
        let file = folder.asset(&path);
        if file.is_file() {
            place(&file, &bundle.path().join(bare))?;
        }
    }
    fs::write(bundle.path().join("index.html"), html)?;
    Ok(bundle)
}
