//! A media job, start to final status: `ffprobe` on the upload, then its poster, filmstrip,
//! waveform and proxy, each only when the job asks for it and the probe allows it. The API reads
//! the files back as untrusted; the worker decodes so the API never has to.

use std::path::Path;
use std::sync::atomic::Ordering;

use clap_media::{Make, Step, files};

use crate::config::Limits;
use crate::decode::{Decoder, Media};
use crate::shared::Shared;
use crate::status::{State, Status};

const STEPS: u32 = Step::Proxy as u32;

fn steps(decoder: &Decoder, make: Make) -> Result<Vec<String>, String> {
    let probe = decoder.probe()?;
    decoder.finished(Step::Probe);
    let picture = probe.has_video;
    let parts = [
        (Step::Poster, files::POSTER, make.poster && picture),
        (Step::Strip, files::STRIP, make.strip && picture),
        (Step::Peaks, files::PEAKS, make.peaks && probe.has_audio),
        (Step::Proxy, files::PROXY, make.proxy && picture),
    ];
    let mut notes = Vec::new();
    for (step, file, wanted) in parts {
        let made = if wanted { decoder.part(step, &probe) } else { Ok(()) };
        if let Err(error) = made {
            notes.push(format!("{file}: {error}"));
        }
        decoder.finished(step);
    }
    Ok(notes)
}

fn work(dir: &Path, media: &Media, make: Make, limits: &Limits) -> Status {
    let shared = Shared::default();
    shared.frames.store(STEPS, Ordering::Relaxed);
    let made = std::thread::scope(|scope| {
        scope.spawn(|| shared.beat(dir, limits.wall));
        let made = steps(&Decoder { dir, media, shared: &shared }, make);
        shared.over.store(true, Ordering::Relaxed);
        made
    });
    match made {
        Ok(notes) => {
            Status { error: (!notes.is_empty()).then(|| notes.join("; ")), ..Status::new(State::Done, STEPS, STEPS) }
        }
        Err(reason) => Status { error: Some(reason), ..shared.status(State::Failed) },
    }
}

/// Runs the media job in `dir` and writes its final status. Nothing outside `dir` is written.
pub(crate) fn run(dir: &Path, media: &Media, limits: &Limits) -> Status {
    let text = crate::files::read(&dir.join("job.json"));
    let status = match text.and_then(|text| serde_json::from_str::<Make>(&text).map_err(|error| error.to_string())) {
        Err(reason) => Status::failed(format!("job.json: {reason}")),
        Ok(_) if crate::files::exists(&dir.join("cancel")) => Status::new(State::Cancelled, 0, 0),
        Ok(_) if !regular(&dir.join(files::INPUT)) => Status::failed("the job has no input file"),
        Ok(make) => {
            let _ = Status::new(State::Running, 0, STEPS).save(dir);
            work(dir, media, make, limits)
        }
    };
    let _ = status.save(dir);
    status
}

fn regular(path: &Path) -> bool {
    path.symlink_metadata().is_ok_and(|meta| meta.file_type().is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SKIP: &str = "\n================ SKIPPED: the worker's media test needs ffmpeg and ffprobe \
        (CLAP_FFMPEG, CLAP_FFPROBE or PATH) ================\n";

    fn job(renders: &Path, id: &str, input: &[u8]) -> std::path::PathBuf {
        let dir = renders.join(id);
        std::fs::create_dir_all(dir.join("bundle")).unwrap();
        std::fs::write(dir.join(files::INPUT), input).unwrap();
        let json = format!(
            r#"{{"id":"{id}","kind":"media","created_at":"2026-11-16T10:00:00Z","poster":true,"strip":true,"peaks":true,"proxy":false}}"#
        );
        std::fs::write(dir.join("job.json"), json).unwrap();
        dir
    }

    fn clip(dir: &Path, ffmpeg: &Path) -> Vec<u8> {
        let path = dir.join("clip.mp4");
        let made = std::process::Command::new(ffmpeg)
            .args(["-nostdin", "-v", "error", "-y", "-f", "lavfi", "-i", "testsrc=duration=1:size=160x120:rate=10"])
            .args(["-f", "lavfi", "-i", "sine=duration=1", "-c:v", "mpeg4", "-c:a", "aac", "-shortest"])
            .arg(&path)
            .status()
            .unwrap();
        assert!(made.success());
        std::fs::read(path).unwrap()
    }

    #[test]
    fn a_media_job_writes_what_it_was_asked_and_a_broken_file_fails_with_the_reason() {
        let (Some(ffmpeg), Some(ffprobe)) = (clap_render::tools::ffmpeg(), clap_render::tools::ffprobe()) else {
            return eprintln!("{SKIP}");
        };
        let renders = tempfile::tempdir().unwrap();
        let media = Media { ffmpeg: ffmpeg.clone(), ffprobe: Some(ffprobe) };
        let good = job(renders.path(), "good", &clip(renders.path(), &ffmpeg));
        let done = run(&good, &media, &Limits::default());
        assert_eq!((done.state, done.frame, done.frames, done.error.clone()), (State::Done, 5, 5, None), "{done:?}");
        for file in [files::PROBE, files::POSTER, files::STRIP, files::PEAKS] {
            assert!(good.join(file).metadata().is_ok_and(|meta| meta.len() > 10), "{file}");
        }
        assert!(!good.join(files::PROXY).exists(), "the proxy was not asked for");
        let broken = run(&job(renders.path(), "broken", b"\0\0\0\x18ftypisom"), &media, &Limits::default());
        assert_eq!((broken.state, broken.frame), (State::Failed, 0));
        assert!(broken.error.as_deref().is_some_and(|error| error.starts_with("ffprobe failed")), "{broken:?}");
        let missing = Media { ffprobe: None, ..media };
        let lost = run(&job(renders.path(), "lost", b"x"), &missing, &Limits::default());
        assert!(lost.error.as_deref().is_some_and(|error| error.contains("CLAP_FFPROBE")), "{lost:?}");
    }
}
