//! Builds a bundle where the worker cannot see it, then publishes it as a job.

use std::path::{Path, PathBuf};

use super::bundle::{JobFile, MEDIA, Plan};
use crate::modules::assets::files;

/// `DATA_DIR/staging`: where the API builds a bundle, out of the worker's sight.
pub fn staging(data_dir: &Path) -> PathBuf {
    data_dir.join("staging")
}

async fn opened(bundle: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mode = std::fs::Permissions::from_mode(super::jobfs::BUNDLE_MODE);
    tokio::fs::set_permissions(bundle, mode).await
}

/// Publishes the bundle the API built in `staged` as the job `name` described by `job`, with
/// [`jobfs::publish`](super::jobfs::publish): the worker sees it once `job.json` is there.
pub async fn publish(data_dir: &Path, staged: &Path, name: &str, job: &impl serde::Serialize) -> std::io::Result<()> {
    opened(staged).await?;
    let encoded = serde_json::to_vec(job).map_err(std::io::Error::other)?;
    let (data_dir, staged, name) = (data_dir.to_owned(), staged.to_owned(), name.to_owned());
    let written = move || super::jobfs::publish(&data_dir, &name, &staged, &encoded);
    tokio::task::spawn_blocking(written).await.map_err(std::io::Error::other)?
}

async fn built(data_dir: &Path, plan: &Plan, bundle: &Path) -> std::io::Result<()> {
    tokio::fs::create_dir_all(bundle).await?;
    for (name, sha256) in &plan.media {
        let target = bundle.join(MEDIA).join(name);
        tokio::fs::create_dir_all(target.parent().unwrap_or(bundle)).await?;
        tokio::fs::copy(files::path(data_dir, sha256), &target).await?;
    }
    tokio::fs::write(bundle.join("index.html"), &plan.html).await
}

/// Writes the job of `job`: builds the bundle in `DATA_DIR/staging/<id>`, each media file a copy
/// of the original (never a link to the blob other projects share), then publishes it with
/// [`jobfs::publish`](super::jobfs::publish): the bundle moves into `renders/<id>` and
/// `job.json` comes last. The staging directory goes either way.
pub async fn write(data_dir: &Path, plan: &Plan, job: &JobFile) -> std::io::Result<()> {
    let staged = staging(data_dir).join(job.id.to_string());
    let bundle = staged.join("bundle");
    let done = match built(data_dir, plan, &bundle).await {
        Ok(()) => publish(data_dir, &bundle, &job.id.to_string(), job).await,
        Err(error) => Err(error),
    };
    tokio::fs::remove_dir_all(&staged).await.ok();
    done
}

/// Makes a received bundle, which sits in the API's own uploads directory, the job `job`, as
/// [`write`] publishes a built one.
pub async fn adopt(data_dir: &Path, received: &Path, job: &JobFile) -> std::io::Result<()> {
    publish(data_dir, received, &job.id.to_string(), job).await
}
