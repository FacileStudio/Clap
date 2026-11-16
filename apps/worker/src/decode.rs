//! The commands of one media job, each writing one file of the job directory whole or not at
//! all.

use std::ffi::OsString;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::time::Duration;

use clap_media::{Probe, Step, files, max};

use crate::child::{self, drain};
use crate::files as fs;
use crate::shared::Shared;

/// The binaries a media job runs.
#[derive(Debug, Clone)]
pub(crate) struct Media {
    pub ffmpeg: PathBuf,
    /// `None` when no `ffprobe` was found: every media job then fails, saying so.
    pub ffprobe: Option<PathBuf>,
}

/// One media job at work: its directory, its binaries, and what it shares with its status thread.
pub(crate) struct Decoder<'a> {
    pub dir: &'a Path,
    pub media: &'a Media,
    pub shared: &'a Shared,
}

impl Decoder<'_> {
    fn stopped(&self) -> bool {
        self.shared.stop.load(Ordering::Relaxed) != 0
    }

    /// Marks `step` finished in the status the API reads.
    pub(crate) fn finished(&self, step: Step) {
        self.shared.frame.store(step as u32, Ordering::Relaxed);
    }

    fn written(
        &self,
        file: &str,
        limit: Duration,
        args: impl FnOnce(&Path, &Path) -> Vec<OsString>,
    ) -> Result<(), String> {
        let (target, partial) = (self.dir.join(file), self.dir.join(format!("{file}.part")));
        fs::clear(&target);
        fs::clear(&partial);
        let ran = child::run(
            &self.media.ffmpeg,
            &args(&self.dir.join(files::INPUT), &partial),
            limit,
            &|| self.stopped(),
            drain,
        );
        let moved = ran.and_then(|()| std::fs::rename(&partial, &target).map_err(|error| error.to_string()));
        if moved.is_err() {
            fs::clear(&partial);
        }
        moved
    }

    /// Runs `ffprobe` on the input, keeps what it printed as `probe.json`, and reads it.
    pub(crate) fn probe(&self) -> Result<Probe, String> {
        let ffprobe = self.media.ffprobe.as_deref().ok_or("ffprobe is missing: set CLAP_FFPROBE")?;
        let args = clap_media::probe_args(&self.dir.join(files::INPUT));
        let read = |out: std::process::ChildStdout| {
            let mut bytes = Vec::new();
            let mut out = out;
            let _ = (&mut out).take(max::PROBE + 1).read_to_end(&mut bytes);
            drain(out);
            bytes
        };
        let report = child::run(ffprobe, &args, clap_media::PROBE_TIMEOUT, &|| self.stopped(), read)?;
        if report.len() as u64 > max::PROBE {
            return Err(format!("ffprobe answered more than {} KiB", max::PROBE >> 10));
        }
        let probe = clap_media::parse(&report)?;
        fs::write(&self.dir.join(files::PROBE), &String::from_utf8_lossy(&report))
            .map_err(|error| error.to_string())?;
        Ok(probe)
    }

    fn peaks(&self, limit: Duration) -> Result<(), String> {
        let args = clap_media::peaks_args(&self.dir.join(files::INPUT));
        let fold = |mut out: std::process::ChildStdout| {
            let (mut state, mut chunks, mut buffer) = ((Vec::new(), 0, 0), Vec::new(), vec![0; 64 * 1024]);
            while let Ok(read @ 1..) = out.read(&mut buffer) {
                clap_media::measure(&buffer[..read], &mut state, &mut chunks);
            }
            if state.1 > 0 {
                chunks.push(state.2);
            }
            chunks
        };
        let chunks = child::run(&self.media.ffmpeg, &args, limit, &|| self.stopped(), fold)?;
        if chunks.is_empty() {
            return Err("the sound has no sample".to_owned());
        }
        let json = String::from_utf8(clap_media::buckets(&chunks)).map_err(|error| error.to_string())?;
        fs::write(&self.dir.join(files::PEAKS), &json).map_err(|error| error.to_string())
    }

    /// Makes the file of `step` from the input `probe` describes.
    pub(crate) fn part(&self, step: Step, probe: &Probe) -> Result<(), String> {
        let (duration, long) = (probe.duration_ms, clap_media::proxy_limit(probe.duration_ms));
        let at = clap_media::poster_at(duration);
        match step {
            Step::Probe => Ok(()),
            Step::Poster => self.written(files::POSTER, clap_media::POSTER_TIMEOUT, |input, output| {
                clap_media::poster_args(input, at, output)
            }),
            Step::Strip => {
                let plan = clap_media::plan(duration).ok_or("the video has no length")?;
                self.written(files::STRIP, long, |input, output| clap_media::strip_args(input, plan, output))
            }
            Step::Peaks => self.peaks(long),
            Step::Proxy => self.written(files::PROXY, long, clap_media::proxy_args),
        }
    }
}
