use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// The lowercase hex of `bytes`.
pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// `DATA_DIR/assets/<first two hex>/<sha256>`: the bytes as uploaded.
pub fn path(data_dir: &Path, sha256: &str) -> PathBuf {
    data_dir.join("assets").join(&sha256[..2]).join(sha256)
}

/// Where the poster of the file `sha256` lives, relative to `DATA_DIR`.
pub fn poster(sha256: &str) -> String {
    format!("derived/{}/{sha256}.poster.jpg", &sha256[..2])
}

/// Where the 720p proxy of the file `sha256` lives, relative to `DATA_DIR`.
pub fn proxy(sha256: &str) -> String {
    format!("derived/{}/{sha256}.proxy.mp4", &sha256[..2])
}

/// `DATA_DIR/uploads`: where an upload is written before its digest is known. Emptied at boot.
pub fn incoming(data_dir: &Path) -> PathBuf {
    data_dir.join("uploads")
}

/// Moves the finished upload at `temporary` under its digest. An identical file is already
/// there: the upload is dropped and only the modification time is refreshed, so the sweep cannot
/// take the file while its new row commits. A rename never shows a reader half a file.
pub async fn adopt(data_dir: &Path, temporary: &Path, sha256: &str) -> io::Result<()> {
    let target = path(data_dir, sha256);
    if tokio::fs::try_exists(&target).await? {
        tokio::fs::remove_file(temporary).await?;
        let touch = move || std::fs::File::options().append(true).open(target)?.set_modified(SystemTime::now());
        return tokio::task::spawn_blocking(touch).await.map_err(io::Error::other)?;
    }
    let dir = target.parent().ok_or_else(|| io::Error::other("asset path has no parent"))?;
    tokio::fs::create_dir_all(dir).await?;
    tokio::fs::rename(temporary, &target).await
}

/// Removes every upload a previous process left unfinished.
pub async fn clear_incoming(data_dir: &Path) {
    let dir = incoming(data_dir);
    if let Err(error) = tokio::fs::remove_dir_all(&dir).await
        && error.kind() != io::ErrorKind::NotFound
    {
        tracing::warn!(%error, "assets: could not clear unfinished uploads");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn the_same_bytes_are_kept_once_under_their_digest() {
        let dir = std::env::temp_dir().join(format!("clap-files-{}", uuid::Uuid::new_v4()));
        let sha = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";
        tokio::fs::create_dir_all(incoming(&dir)).await.unwrap();
        for upload in ["one", "two"] {
            let temporary = incoming(&dir).join(upload);
            tokio::fs::write(&temporary, b"hello").await.unwrap();
            adopt(&dir, &temporary, sha).await.unwrap();
            assert!(!temporary.exists());
        }
        assert_eq!(tokio::fs::read(path(&dir, sha)).await.unwrap(), b"hello");
        assert_eq!(std::fs::read_dir(path(&dir, sha).parent().unwrap()).unwrap().count(), 1);
        assert_eq!(poster(sha), format!("derived/2c/{sha}.poster.jpg"));
        std::fs::remove_dir_all(dir).unwrap();
    }
}
