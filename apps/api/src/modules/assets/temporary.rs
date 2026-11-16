use std::path::{Path, PathBuf};

/// A file, or a directory, on its way in: removed when dropped, unless it was kept.
#[derive(Debug)]
pub struct Temporary(Option<PathBuf>);

impl Temporary {
    /// The file at `path`, removed on drop.
    pub fn at(path: PathBuf) -> Self {
        Temporary(Some(path))
    }

    /// Where the bytes are.
    pub fn path(&self) -> &Path {
        self.0.as_deref().unwrap_or(Path::new(""))
    }

    /// Gives the path up: the caller moved the file and nothing is removed on drop.
    pub fn keep(mut self) {
        self.0.take();
    }
}

impl Drop for Temporary {
    fn drop(&mut self) {
        if let Some(path) = self.0.take() {
            match path.is_dir() {
                true => std::fs::remove_dir_all(path).ok(),
                false => std::fs::remove_file(path).ok(),
            };
        }
    }
}
