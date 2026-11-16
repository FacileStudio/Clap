use std::collections::HashSet;
use std::path::Path;
use std::time::Duration;

use tokio_util::sync::CancellationToken;

use crate::state::AppState;

/// How often unreferenced files are removed.
pub const EVERY: Duration = Duration::from_secs(24 * 60 * 60);

/// A file younger than this is kept: its row may not be committed yet.
pub const GRACE: Duration = Duration::from_secs(60 * 60);

type SweepError = Box<dyn std::error::Error + Send + Sync>;

fn digest(file_name: &str) -> String {
    file_name.split('.').next().unwrap_or_default().to_owned()
}

async fn on_disk(root: &Path, found: &mut Vec<(String, std::path::PathBuf)>) -> std::io::Result<()> {
    let Ok(mut shards) = tokio::fs::read_dir(root).await else { return Ok(()) };
    while let Some(shard) = shards.next_entry().await? {
        let mut files = tokio::fs::read_dir(shard.path()).await?;
        while let Some(file) = files.next_entry().await? {
            let age = file.metadata().await?.modified()?.elapsed().unwrap_or_default();
            if age >= GRACE {
                found.push((digest(&file.file_name().to_string_lossy()), file.path()));
            }
        }
    }
    Ok(())
}

/// Removes every upload, poster and proxy older than [`GRACE`] whose digest no `assets` row
/// names, and answers how many files went.
pub async fn sweep(state: &AppState) -> Result<usize, SweepError> {
    let mut files = Vec::new();
    on_disk(&state.data_dir.join("assets"), &mut files).await?;
    on_disk(&state.data_dir.join("derived"), &mut files).await?;
    let kept: HashSet<String> =
        sqlx::query_scalar("SELECT DISTINCT sha256 FROM assets").fetch_all(&state.pool).await?.into_iter().collect();
    let mut removed = 0;
    for (name, path) in files.into_iter().filter(|(name, _)| !kept.contains(name)) {
        let refreshed = tokio::fs::metadata(&path).await?.modified()?.elapsed().unwrap_or_default() < GRACE;
        if refreshed {
            continue;
        }
        tokio::fs::remove_file(&path).await?;
        tracing::info!(file = %name, "assets: removed an unreferenced file");
        removed += 1;
    }
    Ok(removed)
}

/// Sweeps once a day until `token` is cancelled. Run it in one replica.
pub async fn run(state: AppState, token: CancellationToken) {
    let mut tick = tokio::time::interval(EVERY);
    loop {
        tokio::select! {
            () = token.cancelled() => return,
            _ = tick.tick() => {
                if let Err(error) = sweep(&state).await {
                    tracing::error!(%error, "assets: sweep failed");
                }
            }
        }
    }
}
