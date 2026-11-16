//! `job.json`: what the API asks for, checked against the worker's own limits.

use std::path::Path;

use serde::Deserialize;

use crate::config::Limits;
use crate::{clock, files};

/// The content of the `job.json` of a render; [`Head`] holds its id and its date.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub(crate) struct Spec {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    /// The seconds to render, `[start, end]`; `null` is the whole scene.
    pub range: Option<(f64, f64)>,
}

/// What a job asks for.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Kind {
    /// A short job on an upload: probe it, then make its poster, filmstrip, waveform and proxy.
    /// It sorts first, so an upload never waits behind a queue of renders.
    Media,
    /// A render of `bundle/index.html`, what a `job.json` with no `kind` asks for.
    #[default]
    Render,
}

/// What every `job.json` holds, whatever its kind.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub(crate) struct Head {
    pub id: String,
    #[serde(default)]
    pub kind: Kind,
    pub created_at: String,
}

impl Head {
    /// Reads the `job.json` of `dir` far enough to queue it: its id must be the directory's
    /// name and its `created_at` a timestamp.
    pub(crate) fn load(dir: &Path) -> Result<(Head, String), String> {
        let text = files::read(&dir.join("job.json"))?;
        let head: Head = serde_json::from_str(&text).map_err(|error| format!("job.json: {error}"))?;
        let name = dir.file_name().unwrap_or_default().to_string_lossy();
        if head.id != name {
            return Err(format!("job.json names job `{}` in directory `{name}`", head.id));
        }
        match clock::parse(&head.created_at) {
            Some(_) => Ok((head, text)),
            None => Err("created_at must be an RFC 3339 timestamp".to_owned()),
        }
    }
}

/// Whether `name` can be a job id: 1 to 64 letters, digits, `-` and `_`.
pub(crate) fn named(name: &str) -> bool {
    (1..=64).contains(&name.len())
        && name.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}

impl Spec {
    /// Reads and checks the `job.json` of the job directory `dir`.
    pub(crate) fn load(dir: &Path, limits: &Limits) -> Result<Spec, String> {
        let (_, text) = Head::load(dir)?;
        let spec: Spec = serde_json::from_str(&text).map_err(|error| format!("job.json: {error}"))?;
        spec.check(limits).map(|()| spec)
    }

    fn check(&self, limits: &Limits) -> Result<(), String> {
        let (long, short) = (self.width.max(self.height), self.width.min(self.height));
        let even = self.width.is_multiple_of(2) && self.height.is_multiple_of(2) && short >= 2;
        if !even || long > limits.long_side || short > limits.short_side {
            let (w, h, long, short) = (self.width, self.height, limits.long_side, limits.short_side);
            return Err(format!("{w}x{h} must be even and at most {long}x{short} in either orientation"));
        }
        if !(1..=limits.fps).contains(&self.fps) {
            return Err(format!("{} fps must be between 1 and {}", self.fps, limits.fps));
        }
        let Some((start, end)) = self.range else { return Ok(()) };
        if !(start.is_finite() && end.is_finite() && start >= 0.0 && end > start) {
            return Err(format!("range [{start}, {end}] must be two seconds, the second after the first"));
        }
        match end - start > limits.seconds {
            true => Err(format!("{} s is longer than the {} s a render may last", end - start, limits.seconds)),
            false => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn job(dir: &Path, json: &str) -> Result<Spec, String> {
        let job = dir.join("j1");
        let _ = std::fs::create_dir(&job);
        std::fs::write(job.join("job.json"), json).unwrap();
        Spec::load(&job, &Limits::default())
    }

    #[test]
    fn a_job_is_held_to_the_limits() {
        let dir = tempfile::tempdir().unwrap();
        let with = |fields: &str| format!(r#"{{"id":"j1","created_at":"2026-11-16T10:00:00Z",{fields}}}"#);
        let good = job(dir.path(), &with(r#""width":1920,"height":1080,"fps":30,"range":[0,2.5]"#)).unwrap();
        assert_eq!((good.width, good.range), (1920, Some((0.0, 2.5))));
        assert!(job(dir.path(), &with(r#""width":1080,"height":1920,"fps":30,"range":null"#)).is_ok());
        for (fields, reason) in [
            (r#""width":3840,"height":2160,"fps":30,"range":null"#, "at most 1920x1080"),
            (r#""width":1281,"height":720,"fps":30,"range":null"#, "must be even"),
            (r#""width":1280,"height":720,"fps":60,"range":null"#, "between 1 and 30"),
            (r#""width":1280,"height":720,"fps":30,"range":[0,601]"#, "longer than the 600 s"),
            (r#""width":1280,"height":720,"fps":30,"range":[3,1]"#, "the second after the first"),
        ] {
            let error = job(dir.path(), &with(fields)).unwrap_err();
            assert!(error.contains(reason), "{error}");
        }
        let other = r#"{"id":"j2","width":2,"height":2,"fps":1,"range":null,"created_at":"2026-11-16T10:00:00Z"}"#;
        assert!(job(dir.path(), other).unwrap_err().contains("names job `j2`"));
        let undated = r#"{"id":"j1","width":2,"height":2,"fps":1,"range":null,"created_at":"yesterday"}"#;
        assert!(job(dir.path(), undated).unwrap_err().contains("RFC 3339"));
        let media = r#"{"id":"j1","kind":"media","created_at":"2026-11-16T10:00:00Z","proxy":true}"#;
        std::fs::write(dir.path().join("j1/job.json"), media).unwrap();
        assert_eq!(Head::load(&dir.path().join("j1")).unwrap().0.kind, Kind::Media);
        assert!(named("a-b_9") && !named("") && !named("../x") && !named(".hidden"));
    }
}
