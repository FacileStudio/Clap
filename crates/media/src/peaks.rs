//! The waveform of a sound: at most [`BUCKETS`] loudness values from 0 to 1, each the largest
//! absolute sample of its slice of the file, mono, written as `{"peaks": [...]}`.

use std::ffi::OsString;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// The most values a waveform holds.
pub const BUCKETS: usize = 1000;

/// The rate the sound is decoded at before it is measured.
pub const RATE: u32 = 8000;

/// The samples folded into one measure while the sound streams in: 10 ms at [`RATE`].
const CHUNK: usize = 80;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Waveform {
    peaks: Vec<f64>,
}

/// The arguments that decode the first sound stream of `input` to signed 16-bit mono at
/// [`RATE`] on standard output.
pub fn peaks_args(input: &Path) -> Vec<OsString> {
    let rate = RATE.to_string();
    let after = ["-map", "0:a:0", "-vn", "-ac", "1", "-ar", &rate, "-c:a", "pcm_s16le", "-f", "s16le"];
    crate::framed(&[], input, &after, Path::new("pipe:1"))
}

/// Folds the loudest of each chunk into at most [`BUCKETS`] values, each the loudest of the
/// chunks it covers, rounded to a thousandth, as the JSON file the timeline reads.
pub fn buckets(chunks: &[u16]) -> Vec<u8> {
    let count = chunks.len().min(BUCKETS);
    let peaks = (0..count)
        .map(|bucket| {
            let (from, to) = (bucket * chunks.len() / count, ((bucket + 1) * chunks.len() / count).max(bucket + 1));
            let loudest = chunks[from..to.min(chunks.len())].iter().copied().max().unwrap_or(0);
            (f64::from(loudest) / 32768.0 * 1000.0).round() / 1000.0
        })
        .collect();
    serde_json::to_vec(&Waveform { peaks }).unwrap_or_default()
}

/// The loudest absolute sample of each chunk of 10 ms of the signed 16-bit little-endian
/// `bytes`, carrying an odd byte and a partial chunk over to the next call through `state`.
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

/// A waveform file someone else wrote, written again by us: `{"peaks": [...]}` with 1 to
/// [`BUCKETS`] values, each from 0 to 1. Anything else is refused.
pub fn checked(json: &[u8]) -> Result<Vec<u8>, String> {
    let waveform: Waveform = serde_json::from_slice(json).map_err(|error| format!("not a waveform: {error}"))?;
    let fits = (1..=BUCKETS).contains(&waveform.peaks.len());
    if !fits || !waveform.peaks.iter().all(|value| (0.0..=1.0).contains(value)) {
        return Err(format!("a waveform holds 1 to {BUCKETS} values from 0 to 1"));
    }
    serde_json::to_vec(&waveform).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn values(json: &[u8]) -> Vec<f64> {
        serde_json::from_slice::<Waveform>(json).unwrap().peaks
    }

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
        assert_eq!(values(&buckets(&chunks)), [0.0, 0.5, 0.0]);
        let long: Vec<u16> = (0..2500).map(|n| if n == 2499 { 32767 } else { 0 }).collect();
        let folded = values(&buckets(&long));
        assert_eq!((folded.len(), folded[999], folded[0]), (BUCKETS, 1.0, 0.0));
        let words: Vec<String> =
            peaks_args(Path::new("/d/a b")).iter().map(|arg| arg.to_string_lossy().into_owned()).collect();
        assert!(words.windows(2).any(|pair| pair == ["-max_pixels", "8847360"]), "{words:?}");
        assert_eq!((words[11].as_str(), words.last().unwrap().as_str()), ("/d/a b", "pipe:1"));
    }

    #[test]
    fn only_a_plain_waveform_is_taken_from_the_worker() {
        assert_eq!(checked(br#"{"peaks": [0, 0.5, 1]}"#).unwrap(), br#"{"peaks":[0.0,0.5,1.0]}"#);
        let too_many = format!("{{\"peaks\": [{}]}}", vec!["0.1"; BUCKETS + 1].join(","));
        for bad in [r#"{"peaks": []}"#, r#"{"peaks": [1.5]}"#, r#"{"peaks": [0], "x": "<script>"}"#, "[]", &too_many] {
            assert!(checked(bad.as_bytes()).is_err(), "{bad}");
        }
    }
}
