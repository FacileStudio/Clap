//! Which URLs the page may load: files of its own bundle, and nothing else.

use std::path::{Path, PathBuf};

use url::Url;

/// How many refused URLs a guard remembers.
const REFUSED_KEPT: usize = 100;

/// Allows `file://` URLs that resolve inside one directory, symlinks followed, and remembers
/// what it refused.
#[derive(Debug)]
pub(crate) struct Guard {
    root: PathBuf,
    refused: Vec<String>,
}

impl Guard {
    /// A guard over `dir`, which must exist.
    pub(crate) fn new(dir: &Path) -> std::io::Result<Guard> {
        Ok(Guard { root: dir.canonicalize()?, refused: Vec::new() })
    }

    /// The bundle directory, canonical.
    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    /// The URLs refused so far, at most the first hundred.
    pub(crate) fn refused(&self) -> &[String] {
        &self.refused
    }

    /// Decides one request, remembering a refusal.
    pub(crate) fn decide(&mut self, url: &str) -> bool {
        let allowed = self.allows(url);
        if !allowed && self.refused.len() < REFUSED_KEPT {
            self.refused.push(url.to_owned());
        }
        allowed
    }

    /// The file a `file://` URL names, canonical, when it sits inside the bundle.
    pub(crate) fn file(&self, url: &str) -> Option<PathBuf> {
        let url = Url::parse(url).ok().filter(|url| url.scheme() == "file")?;
        url.to_file_path().ok()?.canonicalize().ok().filter(|path| path.starts_with(&self.root))
    }

    /// Whether the page may load `url`.
    fn allows(&self, url: &str) -> bool {
        let scheme = Url::parse(url).map(|url| url.scheme().to_owned()).unwrap_or_default();
        matches!(scheme.as_str(), "data" | "blob" | "about") || self.file(url).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::Guard;

    #[test]
    fn only_files_inside_the_bundle_load() {
        let bundle = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::fs::write(bundle.path().join("index.html"), "").unwrap();
        std::fs::write(outside.path().join("secret.txt"), "").unwrap();
        std::os::unix::fs::symlink(outside.path().join("secret.txt"), bundle.path().join("link.txt")).unwrap();
        let guard = Guard::new(bundle.path()).unwrap();
        let inside = |name: &str| format!("file://{}/{name}", guard.root().display());
        assert!(guard.allows(&inside("index.html")));
        assert!(guard.allows(&format!("{}?v=1#top", inside("index.html"))));
        assert!(guard.allows("data:text/plain,hi"));
        assert!(!guard.allows(&inside("missing.png")));
        assert!(!guard.allows(&inside("link.txt")));
        assert!(!guard.allows(&inside("../../etc/hostname")));
        assert!(!guard.allows(&format!("file://{}/secret.txt", outside.path().display())));
        assert!(!guard.allows("file:///etc/hostname"));
        assert!(!guard.allows("https://example.com/a.png"));
        assert!(!guard.allows("http://127.0.0.1:4000/api/health"));
        assert!(!guard.allows("ws://127.0.0.1:4000/"));
        assert!(!guard.allows("not a url"));
        let mut guard = guard;
        assert!(guard.decide("data:,") && !guard.decide("https://example.com/") && !guard.decide("ftp://x/"));
        assert_eq!(guard.refused(), ["https://example.com/", "ftp://x/"]);
    }
}
