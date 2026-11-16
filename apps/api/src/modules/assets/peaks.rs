//! The waveform of a sound: at most [`BUCKETS`] loudness values from 0 to 1, each the largest
//! absolute sample of its slice of the file, mono, written as `{"peaks": [...]}`.

use std::ffi::OsString;
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

use tokio::io::AsyncReadExt;

use super::derive;
use super::tools::Tools;

/// The most values a waveform holds.
pub const BUCKETS: usize = 1000;

/// The rate the sound is decoded at before it is measured.
pub const RATE: u32 = 8000;

/// The samples folded into one measure while the sound streams in: 10 ms at [`RATE`].
const CHUNK: usize = 80;

/// The arguments that decode the first sound stream of `input` to signed 16-bit mono at
/// [`RATE`] on standard output.
pub fn args(input: &Path) -> Vec<OsString> {
    let rate = RATE.to_string();
    let after = ["-map", "0:a:0", "-vn", "-ac", "1", "-ar", &rate, "-c:a", "pcm_s16le", "-f", "s16le"];
    derive::framed(&[], input, &after, Path::new("pipe:1"))
}

/// Folds the loudest of each [`CHUNK`] into at most [`BUCKETS`] values, each the loudest of the
/// chunks it covers, rounded to a thousandth.
pub fn buckets(chunks: &[u16]) -> Vec<f64> {
    let count = chunks.len().min(BUCKETS);
    (0..count)
        .map(|bucket| {
            let (from, to) = (bucket * chunks.len() / count, ((bucket + 1) * chunks.len() / count).max(bucket + 1));
            let loudest = chunks[from..to.min(chunks.len())].iter().copied().max().unwrap_or(0);
            (f64::from(loudest) / 32768.0 * 1000.0).round() / 1000.0
        })
        .collect()
}

/// The loudest absolute sample of each [`CHUNK`] of the signed 16-bit little-endian `bytes`,
/// carrying an odd byte and a partial chunk over to the next call through `state`.
pub fn measure(bytes: &[u8], state: &mut (Vec<u8>, usize, u16), chunks: &mut Vec<u16>) {
    let (carry, filled, loudest) = state;
    carry.extend_from_slice(bytes);
    let whole = carry.len() / 2 * 2;
    for pair in carry[..whole].as_chunks::<2>().0 {
        *loudest = (*loudest).max(i16::from_le_bytes(*pair).unsigned_abs());
        *filled += 1;
        if *filled == CHUNK {
            chunks.push(std::mem::take(loudest));
            *filled = 0;
        }
    }
    carry.drain(..whole);
}

async fn decoded(tools: &Tools, input: &Path) -> Result<Vec<u16>, String> {
    let mut child = tokio::process::Command::new(&tools.ffmpeg)
        .args(args(input))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .map_err(|error| format!("ffmpeg could not start: {error}"))?;
    let mut stdout = child.stdout.take().ok_or("ffmpeg has no output")?;
    let (mut state, mut chunks, mut buffer) = ((Vec::new(), 0, 0), Vec::new(), vec![0; 64 * 1024]);
    loop {
        let read = stdout.read(&mut buffer).await.map_err(|error| format!("ffmpeg failed: {error}"))?;
        if read == 0 {
            break;
        }
        measure(&buffer[..read], &mut state, &mut chunks);
    }
    if state.1 > 0 {
        chunks.push(state.2);
    }
    let status = child.wait().await.map_err(|error| format!("ffmpeg failed: {error}"))?;
    if !status.success() {
        return Err(format!("ffmpeg failed: {status}"));
    }
    Ok(chunks)
}

/// Writes the waveform of `input` to `output` within `limit`, unless it is already there. A
/// file with no sound sample is an error. The file appears whole or not at all.
pub async fn peaks(tools: &Tools, input: &Path, output: &Path, limit: Duration) -> Result<(), String> {
    if tokio::fs::try_exists(output).await.unwrap_or(false) {
        return Ok(());
    }
    let chunks = match tokio::time::timeout(limit, decoded(tools, input)).await {
        Ok(chunks) => chunks?,
        Err(_) => return Err(format!("ffmpeg was stopped after {} s", limit.as_secs())),
    };
    if chunks.is_empty() {
        return Err("the sound has no sample".to_owned());
    }
    let body =
        serde_json::to_vec(&serde_json::json!({ "peaks": buckets(&chunks) })).map_err(|error| error.to_string())?;
    let scratch = output.with_extension("part");
    if let Some(dir) = output.parent() {
        tokio::fs::create_dir_all(dir).await.map_err(|error| error.to_string())?;
    }
    let moved = match tokio::fs::write(&scratch, body).await {
        Ok(()) => tokio::fs::rename(&scratch, output).await.map_err(|error| error.to_string()),
        Err(error) => Err(error.to_string()),
    };
    if moved.is_err() {
        tokio::fs::remove_file(&scratch).await.ok();
    }
    moved
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn samples_fold_into_loudest_chunks_whatever_the_read_sizes() {
        let samples: Vec<i16> = (0..CHUNK as i16 * 3).map(|n| if n == 100 { -16384 } else { n % 7 }).collect();
        let bytes: Vec<u8> = samples.iter().flat_map(|sample| sample.to_le_bytes()).collect();
        let (mut state, mut chunks) = ((Vec::new(), 0, 0), Vec::new());
        for part in bytes.chunks(3) {
            measure(part, &mut state, &mut chunks);
        }
        assert_eq!(chunks, [6, 16384, 6]);
        assert_eq!(state.1, 0);
        assert_eq!(buckets(&chunks), [0.0, 0.5, 0.0]);
        let long: Vec<u16> = (0..2500).map(|n| if n == 2499 { 32767 } else { 0 }).collect();
        let folded = buckets(&long);
        assert_eq!((folded.len(), folded[999], folded[0]), (BUCKETS, 1.0, 0.0));
        let words: Vec<String> =
            args(Path::new("/d/a b")).iter().map(|arg| arg.to_string_lossy().into_owned()).collect();
        assert!(words.windows(2).any(|pair| pair == ["-max_pixels", "8847360"]), "{words:?}");
        assert_eq!((words[11].as_str(), words.last().unwrap().as_str()), ("/d/a b", "pipe:1"));
    }
}
