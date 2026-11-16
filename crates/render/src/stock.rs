//! Every `<video>` of a scene as numbered JPEG frames, extracted by ffmpeg before capture.
//! Chrome never decodes video: at each captured frame the page is handed the still to show.

use std::ops::Range;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use tempfile::TempDir;
use url::Url;

use crate::Error;
use crate::job::Job;
use crate::media::Media;
use crate::proc::run;
use crate::tools::Tools;

/// One extracted clip: the output frames it covers and the stills that fill them. From the end
/// of `frames` to `until` it holds its last still.
struct Reel {
    index: usize,
    frames: Range<u64>,
    until: u64,
    stills: Vec<String>,
}

/// The extracted frames of one render. Dropping it removes them from the disk.
pub(crate) struct Stock {
    reels: Vec<Reel>,
    _dir: Option<TempDir>,
}

fn size(dir: &Path) -> u64 {
    let entries = std::fs::read_dir(dir).into_iter().flatten().flatten();
    entries
        .map(
            |entry| {
                if entry.path().is_dir() { size(&entry.path()) } else { entry.metadata().map_or(0, |file| file.len()) }
            },
        )
        .sum()
}

fn stills(dir: &Path) -> Result<Vec<String>, Error> {
    let mut files: Vec<_> = std::fs::read_dir(dir)?.filter_map(|entry| Some(entry.ok()?.path())).collect();
    files.sort();
    let url = |file: &std::path::PathBuf| Url::from_file_path(file).map(String::from);
    files.iter().map(|file| url(file).map_err(|()| Error::Job("the frame path is not absolute".to_owned()))).collect()
}

/// What one extraction needs to know.
struct Cut<'a> {
    tools: &'a Tools,
    job: &'a Job,
    root: &'a Path,
}

impl Cut<'_> {
    fn reel(
        &self,
        media: &Media,
        frames: Range<u64>,
        tick: &mut dyn FnMut() -> Result<(), Error>,
    ) -> Result<Reel, Error> {
        let fps = f64::from(self.job.fps);
        let seek = (media.offset + frames.start as f64 / fps - media.at - 0.0005).max(0.0);
        let count = frames.end - frames.start;
        let dir = self.root.join(media.index.to_string());
        std::fs::create_dir(&dir)?;
        let mut command = Command::new(&self.tools.ffmpeg);
        command.args(["-hide_banner", "-loglevel", "error", "-nostdin", "-y", "-ss", &format!("{seek:.6}"), "-i"]);
        command.arg(&media.path).args(["-an", "-sn", "-vf", &format!("fps={}", self.job.fps)]);
        command.args(["-frames:v", &count.to_string(), "-q:v", "2", "-start_number", "0"]).arg(dir.join("%06d.jpg"));
        let limit = Duration::from_secs(60) + Duration::from_millis(250) * count as u32;
        let mut checks = 0u32;
        let budget = self.job.frame_budget;
        let done = run(&mut command, limit, &mut || {
            checks += 1;
            let over = checks.is_multiple_of(10) && size(self.root) > budget;
            if over { Err(Error::Job(format!("the extracted video frames pass {budget} bytes"))) } else { tick() }
        })?;
        let name = media.path.file_name().unwrap_or_default().to_string_lossy().into_owned();
        let stills = stills(&dir)?;
        match done.ok && !stills.is_empty() {
            true => Ok(Reel { index: media.index, until: frames.end, frames, stills }),
            false => Err(Error::Encode(format!("no frame could be read from {name}: {}", done.said))),
        }
    }
}

impl Stock {
    /// Extracts, for each video of `media`, the frames that the output frames `span` show, at the
    /// job's rate, into a directory under the bundle that the guard already allows.
    pub(crate) fn extract<'m>(
        tools: &Tools,
        job: &Job,
        plan: (&'m [Media], Range<u64>),
        tick: &mut dyn FnMut() -> Result<(), Error>,
    ) -> Result<Stock, Error> {
        let (media, span) = plan;
        let fps = f64::from(job.fps);
        let frame = |seconds: f64| (seconds * fps - 1e-6).ceil().max(0.0) as u64;
        let covered = |clip: &'m Media| -> (&'m Media, Range<u64>, u64) {
            let (first, end) = (frame(clip.at), frame(clip.at + clip.duration));
            let start = first.max(span.start.min(end.saturating_sub(1)));
            (clip, start..end.min(span.end), frame(clip.at + clip.duration + clip.hold).min(span.end))
        };
        let shown = |(_, frames, held): &(&Media, Range<u64>, u64)| frames.end > frames.start && *held > span.start;
        let videos: Vec<_> = media.iter().filter(|clip| clip.video).map(covered).filter(shown).collect();
        if videos.is_empty() {
            return Ok(Stock { reels: Vec::new(), _dir: None });
        }
        let dir = tempfile::Builder::new().prefix(".clap-frames-").tempdir_in(job.dir.canonicalize()?)?;
        let cut = Cut { tools, job, root: dir.path() };
        let reels: Result<Vec<Reel>, Error> = videos
            .into_iter()
            .map(|(clip, frames, held)| {
                cut.reel(clip, frames, tick).map(|reel| Reel { until: held.max(reel.until), ..reel })
            })
            .collect();
        Ok(Stock { reels: reels?, _dir: Some(dir) })
    }

    /// How many videos were extracted.
    pub(crate) fn len(&self) -> usize {
        self.reels.len()
    }

    /// What the page must show at output frame `index`: each active video and its still. A file
    /// shorter than its clip holds its last frame.
    pub(crate) fn at(&self, index: u64) -> Vec<(usize, &str)> {
        let active = self.reels.iter().filter(|reel| (reel.frames.start..reel.until).contains(&index));
        active
            .map(|reel| {
                (reel.index, reel.stills[((index - reel.frames.start) as usize).min(reel.stills.len() - 1)].as_str())
            })
            .collect()
    }
}
