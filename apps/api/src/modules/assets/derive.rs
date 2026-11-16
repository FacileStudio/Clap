use std::ffi::OsString;
use std::path::Path;
use std::time::Duration;

use super::tools::{self, Tools};

/// How long `ffmpeg` gets to grab a poster frame.
pub const POSTER_TIMEOUT: Duration = Duration::from_secs(60);

/// How long `ffmpeg` gets to encode a proxy.
pub const PROXY_TIMEOUT: Duration = Duration::from_secs(60 * 60);

const HEAD: [&str; 6] = ["-nostdin", "-v", "error", "-y", "-protocol_whitelist", "file"];

fn framed(before: &[&str], input: &Path, after: &[&str], output: &Path) -> Vec<OsString> {
    let mut args = tools::argv(&HEAD);
    args.extend(tools::argv(before));
    args.push("-i".into());
    args.push(input.into());
    args.extend(tools::argv(after));
    args.push(output.into());
    args
}

/// The arguments that write one JPEG frame of `input`, taken at `at_ms`, at most 640 pixels wide.
pub fn poster_args(input: &Path, at_ms: i64, output: &Path) -> Vec<OsString> {
    let at = format!("{:.3}", at_ms.max(0) as f64 / 1000.0);
    let after = ["-frames:v", "1", "-vf", "scale='min(640,iw)':-2", "-q:v", "5", "-f", "mjpeg"];
    framed(&["-ss", &at], input, &after, output)
}

/// The arguments that write the proxy of `input`: H.264 and AAC in MP4, at most 720 pixels high,
/// playable before it is fully loaded.
pub fn proxy_args(input: &Path, output: &Path) -> Vec<OsString> {
    let video = ["-map", "0:v:0", "-map", "0:a:0?", "-vf", "scale=-2:'2*trunc(min(720,ih)/2)'"];
    let codecs = ["-c:v", "libx264", "-preset", "veryfast", "-crf", "23", "-pix_fmt", "yuv420p"];
    let sound = ["-c:a", "aac", "-b:a", "128k", "-movflags", "+faststart", "-f", "mp4"];
    framed(&[], input, &[&video[..], &codecs[..], &sound[..]].concat(), output)
}

async fn written(tools: &Tools, args: Vec<OsString>, limit: Duration, output: &Path) -> Result<(), String> {
    let scratch = output.with_extension("part");
    let mut args = args;
    args.pop();
    args.push(scratch.clone().into());
    if let Some(dir) = output.parent() {
        tokio::fs::create_dir_all(dir).await.map_err(|error| error.to_string())?;
    }
    let done = tools::run(&tools.ffmpeg, &args, limit).await;
    let moved = match done {
        Ok(_) => tokio::fs::rename(&scratch, output).await.map_err(|error| error.to_string()),
        Err(error) => Err(error),
    };
    if moved.is_err() {
        tokio::fs::remove_file(&scratch).await.ok();
    }
    moved
}

/// Writes the poster of `input` to `output`, unless it is already there. The file appears whole
/// or not at all.
pub async fn poster(tools: &Tools, input: &Path, at_ms: i64, output: &Path) -> Result<(), String> {
    if tokio::fs::try_exists(output).await.unwrap_or(false) {
        return Ok(());
    }
    written(tools, poster_args(input, at_ms, output), POSTER_TIMEOUT, output).await
}

/// Writes the proxy of `input` to `output`, unless it is already there.
pub async fn proxy(tools: &Tools, input: &Path, output: &Path) -> Result<(), String> {
    if tokio::fs::try_exists(output).await.unwrap_or(false) {
        return Ok(());
    }
    written(tools, proxy_args(input, output), PROXY_TIMEOUT, output).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(args: &[OsString]) -> Vec<String> {
        args.iter().map(|arg| arg.to_string_lossy().into_owned()).collect()
    }

    #[test]
    fn ffmpeg_reads_nothing_but_the_input_and_takes_paths_as_whole_arguments() {
        let poster = words(&poster_args(Path::new("/d/in put"), 1500, Path::new("/d/out.jpg")));
        assert_eq!(
            &poster[..9],
            ["-nostdin", "-v", "error", "-y", "-protocol_whitelist", "file", "-ss", "1.500", "-i"]
        );
        assert_eq!((poster[9].as_str(), poster.last().unwrap().as_str()), ("/d/in put", "/d/out.jpg"));
        let proxy = words(&proxy_args(Path::new("/d/in"), Path::new("/d/out.mp4")));
        assert!(proxy.contains(&"libx264".to_owned()) && proxy.contains(&"+faststart".to_owned()));
        assert!(proxy.iter().any(|word| word.contains("min(720,ih)")));
    }
}
