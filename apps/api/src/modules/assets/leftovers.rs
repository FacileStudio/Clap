//! What a previous process left half written: uploads, staged bundles and media jobs.

use std::io;
use std::path::Path;

/// Removes the uploads, staged bundles and media job directories a previous process left
/// unfinished, once they are a day old: during a deploy the other process may still be writing
/// or waiting on younger ones.
pub async fn clear(data_dir: &Path) {
    let renders = crate::modules::renders::bundle::root(data_dir);
    let dirs =
        [(super::files::incoming(data_dir), ""), (data_dir.join("staging"), ""), (renders, super::media::PREFIX)];
    for (dir, prefix) in dirs {
        if let Err(error) = clear_older(&dir, prefix, LEFTOVER).await {
            tracing::warn!(%error, dir = %dir.display(), "assets: could not clear unfinished uploads");
        }
    }
}

/// How old an upload or a staged bundle must be before the boot pass takes it: a day. Another
/// process may be receiving a younger one right now.
pub const LEFTOVER: std::time::Duration = std::time::Duration::from_secs(24 * 60 * 60);

/// Removes the entries of `dir` whose name starts with `prefix` and that were last changed more
/// than `age` ago, and answers how many went.
pub async fn clear_older(dir: &Path, prefix: &str, age: std::time::Duration) -> io::Result<usize> {
    let mut removed = 0;
    let Ok(mut entries) = tokio::fs::read_dir(dir).await else { return Ok(0) };
    while let Some(entry) = entries.next_entry().await? {
        if !entry.file_name().to_string_lossy().starts_with(prefix) {
            continue;
        }
        let metadata = tokio::fs::symlink_metadata(entry.path()).await?;
        if metadata.modified()?.elapsed().unwrap_or_default() < age {
            continue;
        }
        match metadata.is_dir() {
            true => tokio::fs::remove_dir_all(entry.path()).await?,
            false => tokio::fs::remove_file(entry.path()).await?,
        }
        removed += 1;
    }
    Ok(removed)
}
