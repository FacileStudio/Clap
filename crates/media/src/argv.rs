//! The argument vectors of every decode of an upload: never a shell, each word one element.

use std::ffi::OsString;
use std::path::Path;

/// The input options of every `ffprobe` and `ffmpeg` run on an upload: two decoding threads, and
/// no picture over DCI 4K (4096x2160 pixels), so a hostile file cannot make a decoder allocate
/// without bound.
pub const DECODE: [&str; 4] = ["-threads", "2", "-max_pixels", "8847360"];

const HEAD: [&str; 6] = ["-nostdin", "-v", "error", "-y", "-protocol_whitelist", "file"];

/// `words` as an argument vector, each word one whole element.
pub fn argv(words: &[&str]) -> Vec<OsString> {
    words.iter().map(OsString::from).collect()
}

/// The arguments that make `ffprobe` describe `path` as JSON, reading only local files.
pub fn probe_args(path: &Path) -> Vec<OsString> {
    let mut args = argv(&["-v", "error", "-protocol_whitelist", "file", "-print_format", "json"]);
    args.extend(argv(&["-show_format", "-show_streams"]));
    args.extend(argv(&DECODE));
    args.push("-i".into());
    args.push(path.into());
    args
}

/// The `ffmpeg` arguments that read `input` under the decoder limits, after the input options
/// `before`, then write `output` with the output options `after` on two threads.
pub fn framed(before: &[&str], input: &Path, after: &[&str], output: &Path) -> Vec<OsString> {
    let mut args = argv(&HEAD);
    args.extend(argv(before));
    args.extend(argv(&DECODE));
    args.push("-i".into());
    args.push(input.into());
    args.extend(argv(after));
    args.extend(argv(&["-threads", "2"]));
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

#[cfg(test)]
mod tests {
    use super::*;

    fn words(args: &[OsString]) -> Vec<String> {
        args.iter().map(|arg| arg.to_string_lossy().into_owned()).collect()
    }

    #[test]
    fn ffmpeg_reads_nothing_but_the_input_and_takes_paths_as_whole_arguments() {
        let poster = words(&poster_args(Path::new("/d/in put"), 1500, Path::new("/d/out.jpg")));
        let head = ["-nostdin", "-v", "error", "-y", "-protocol_whitelist", "file", "-ss", "1.500"];
        assert_eq!(&poster[..8], head);
        assert_eq!(&poster[8..13], ["-threads", "2", "-max_pixels", "8847360", "-i"], "decoder limits come first");
        assert_eq!((poster[13].as_str(), poster.last().unwrap().as_str()), ("/d/in put", "/d/out.jpg"));
        let proxy = words(&proxy_args(Path::new("/d/in"), Path::new("/d/out.mp4")));
        assert!(proxy.contains(&"libx264".to_owned()) && proxy.contains(&"+faststart".to_owned()));
        assert!(proxy.iter().any(|word| word.contains("min(720,ih)")));
        assert!(proxy.windows(2).any(|pair| pair == ["-max_pixels", "8847360"]), "{proxy:?}");
        assert_eq!(proxy.iter().filter(|word| *word == "-threads").count(), 2, "decoder and encoder: {proxy:?}");
    }

    #[test]
    fn the_probed_file_is_one_whole_argument_and_only_local_files_are_read() {
        let args = words(&probe_args(Path::new("/data/assets/ab/a b;rm -rf")));
        assert_eq!(args.last().unwrap(), "/data/assets/ab/a b;rm -rf");
        assert!(args.windows(2).any(|pair| pair[0] == "-protocol_whitelist" && pair[1] == "file"));
        let at = |word: &str| args.iter().position(|arg| arg == word).unwrap();
        assert!(at("-max_pixels") < at("-i") && at("-threads") < at("-i"), "{args:?}");
    }
}
