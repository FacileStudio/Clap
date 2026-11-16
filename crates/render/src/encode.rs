//! ffmpeg, fed PNG frames on stdin, writing H.264 in an MP4.

use std::ffi::OsString;
use std::io::{Read, Write};
use std::process::{Child, ChildStdin, Command, Stdio};

use crate::Error;
use crate::job::Job;
use crate::tools::Tools;

/// A running ffmpeg. Dropping it before [`Encoder::finish`] kills the process.
pub(crate) struct Encoder {
    child: Child,
    stdin: Option<ChildStdin>,
}

/// The argument vector for `job`. Every value is its own element: nothing is parsed by a shell,
/// and the output is an absolute path, so ffmpeg reads it as neither an option nor a protocol.
/// Audio inputs and their filter graph join here.
pub(crate) fn args(job: &Job) -> Result<Vec<OsString>, Error> {
    let rate = job.fps.to_string();
    let mut args: Vec<OsString> = ["-hide_banner", "-loglevel", "error", "-y"].map(OsString::from).to_vec();
    args.extend(["-f", "image2pipe", "-framerate", &rate, "-c:v", "png", "-i", "pipe:0"].map(OsString::from));
    args.extend(["-an", "-c:v", "libx264", "-preset", "medium", "-crf", "18"].map(OsString::from));
    args.extend(["-pix_fmt", "yuv420p", "-r", &rate, "-movflags", "+faststart"].map(OsString::from));
    args.extend(["-fflags", "+bitexact", "-flags:v", "+bitexact", "-map_metadata", "-1"].map(OsString::from));
    args.extend(["-f", "mp4"].map(OsString::from));
    args.push(std::path::absolute(&job.output)?.into_os_string());
    Ok(args)
}

impl Encoder {
    /// Starts ffmpeg for `job`.
    pub(crate) fn start(tools: &Tools, job: &Job) -> Result<Encoder, Error> {
        let mut child = Command::new(&tools.ffmpeg)
            .args(args(job)?)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()?;
        let stdin = child.stdin.take();
        Ok(Encoder { child, stdin })
    }

    /// Feeds one PNG frame.
    pub(crate) fn write(&mut self, frame: &[u8]) -> Result<(), Error> {
        let written = self.stdin.as_mut().map(|stdin| stdin.write_all(frame));
        match written {
            Some(Ok(())) => Ok(()),
            _ => self.finish(),
        }
    }

    /// Closes the input and waits for the file to be complete.
    pub(crate) fn finish(&mut self) -> Result<(), Error> {
        drop(self.stdin.take());
        let mut complaint = String::new();
        if let Some(mut stderr) = self.child.stderr.take() {
            let _ = stderr.read_to_string(&mut complaint);
        }
        match self.child.wait()? {
            status if status.success() => Ok(()),
            status => Err(Error::Encode(format!("{status}: {}", complaint.trim()))),
        }
    }
}

impl Drop for Encoder {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[cfg(test)]
mod tests {
    use super::args;
    use crate::job::Job;

    #[test]
    fn the_output_is_absolute_and_the_rate_is_the_jobs() {
        let job = Job { fps: 24, ..Job::new("/bundle", "-out.mp4") };
        let args = args(&job).unwrap();
        let text: Vec<String> = args.iter().map(|arg| arg.to_string_lossy().into_owned()).collect();
        assert!(text.last().unwrap().starts_with('/'), "{text:?}");
        assert!(text.windows(2).any(|pair| pair == ["-framerate", "24"]), "{text:?}");
        assert!(text.windows(2).any(|pair| pair == ["-c:v", "libx264"]), "{text:?}");
    }
}
