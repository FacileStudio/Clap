//! The boot pass that makes stored blobs read-only.

use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use super::files::SEALED;

/// Makes every stored blob read-only, once at boot: blobs stored before [`SEALED`] existed were
/// writable. Answers how many changed.
pub fn seal(data_dir: &Path) -> io::Result<usize> {
    let mut changed = 0;
    let Ok(shards) = std::fs::read_dir(data_dir.join("assets")) else { return Ok(0) };
    for blob in shards.flatten().flat_map(|shard| std::fs::read_dir(shard.path()).into_iter().flatten().flatten()) {
        let metadata = std::fs::symlink_metadata(blob.path())?;
        if metadata.is_file() && metadata.permissions().mode() & 0o222 != 0 {
            std::fs::set_permissions(blob.path(), std::fs::Permissions::from_mode(SEALED))?;
            changed += 1;
        }
    }
    Ok(changed)
}
