//! The heartbeat: a file in the renders directory the worker rewrites while it is alive.

use std::path::Path;

use crate::{clock, files};

/// The name of the heartbeat file. It starts with a dot, so it is never taken for a job.
const NAME: &str = ".worker";
/// The oldest heartbeat a healthy worker has.
const FRESH_SECONDS: i64 = 30;

/// Writes the heartbeat into `renders`.
pub(crate) fn beat(renders: &Path) {
    let _ = files::write(&renders.join(NAME), &clock::now());
}

/// Whether the worker over `renders` wrote its heartbeat in the last half minute.
pub(crate) fn alive(renders: &Path) -> bool {
    let written = files::read(&renders.join(NAME)).ok();
    written.and_then(|text| clock::age(&text)).is_some_and(|age| (-FRESH_SECONDS..=FRESH_SECONDS).contains(&age))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_heartbeat_is_alive_and_an_old_one_is_not() {
        let renders = tempfile::tempdir().unwrap();
        assert!(!alive(renders.path()));
        beat(renders.path());
        assert!(alive(renders.path()));
        std::fs::write(renders.path().join(NAME), "2020-01-01T00:00:00Z").unwrap();
        assert!(!alive(renders.path()));
    }
}
