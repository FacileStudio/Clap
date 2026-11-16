//! What links a folder to a project on a server: the `.clap.json` file, and the content hashes
//! `pull` and `push` compare.

pub mod plan;

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Component, Path};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::failure::{Failure, Outcome};

/// The file that links a folder to its project.
pub const LINK_FILE: &str = ".clap.json";

/// The folder, beside the entry file, whose files are the project's assets.
pub const ASSETS_DIR: &str = "assets";

/// Asset names to the SHA-256 of their content. A name is the path the source writes,
/// relative to the project folder, such as `assets/chart.png`.
pub type Hashes = BTreeMap<String, String>;

/// The content of `.clap.json`: the project, and what it held at the last pull or push.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Link {
    /// The project id.
    pub project: String,
    /// The site root of the server that holds the project.
    pub server: String,
    /// The revision of the source at the last pull or push.
    pub rev: i64,
    /// The SHA-256 of the source at the last pull or push.
    #[serde(default)]
    pub source_sha256: String,
    /// The assets at the last pull or push.
    #[serde(default)]
    pub assets: Hashes,
}

impl Link {
    /// Reads the link of `dir`, or `None` when the folder is not linked.
    pub fn load(dir: &Path) -> Outcome<Option<Link>> {
        let path = dir.join(LINK_FILE);
        if !path.is_file() {
            return Ok(None);
        }
        let reason = |detail: String| Failure::new("bad_link", format!("cannot read {}: {detail}", path.display()));
        let text = fs::read_to_string(&path).map_err(|error| reason(error.to_string()))?;
        serde_json::from_str(&text).map(Some).map_err(|error| reason(error.to_string()))
    }

    /// Writes the link into `dir`.
    pub fn save(&self, dir: &Path) -> Outcome {
        let text = serde_json::to_string_pretty(self).map_err(io::Error::other)?;
        Ok(fs::write(dir.join(LINK_FILE), text + "\n")?)
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// The SHA-256 of `bytes`, in lowercase hex.
pub fn sha256(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}

/// The SHA-256 of a file, read in pieces: a video is never held in memory.
pub fn sha256_file(path: &Path) -> io::Result<String> {
    let (mut file, mut hasher, mut buffer) = (File::open(path)?, Sha256::new(), vec![0u8; 1 << 16]);
    loop {
        match file.read(&mut buffer)? {
            0 => return Ok(hex(&hasher.finalize())),
            read => hasher.update(&buffer[..read]),
        }
    }
}

/// Whether an asset name may be written into a project folder: a relative path with no empty,
/// `.` or `..` part, no hidden part, and not the project's own source file. The server's names
/// are checked with this before a pull writes anything.
pub fn safe_name(name: &str) -> bool {
    let plain =
        |part: Component<'_>| matches!(part, Component::Normal(text) if !text.to_string_lossy().starts_with('.'));
    !name.is_empty() && name != crate::project::ENTRY && Path::new(name).components().all(plain)
}

fn walk(root: &Path, at: &Path, found: &mut Hashes) -> io::Result<()> {
    for entry in fs::read_dir(at)? {
        let path = entry?.path();
        let hidden = path.file_name().is_some_and(|name| name.to_string_lossy().starts_with('.'));
        if hidden {
            continue;
        }
        if path.is_dir() {
            walk(root, &path, found)?;
        } else if let Ok(relative) = path.strip_prefix(root) {
            let name: Vec<String> = relative.iter().map(|part| part.to_string_lossy().into_owned()).collect();
            found.insert(name.join("/"), sha256_file(&path)?);
        }
    }
    Ok(())
}

/// Hashes the folder's assets: every file under `dir/assets`, hidden files left out, and
/// each file of `known`, the last sync, that still exists elsewhere in the folder.
pub fn local_assets(dir: &Path, known: &Hashes) -> io::Result<Hashes> {
    let mut found = Hashes::new();
    if dir.join(ASSETS_DIR).is_dir() {
        walk(dir, &dir.join(ASSETS_DIR), &mut found)?;
    }
    for name in known.keys().filter(|name| safe_name(name)) {
        if let Ok(hash) = sha256_file(&dir.join(name)) {
            found.insert(name.clone(), hash);
        }
    }
    Ok(found)
}
