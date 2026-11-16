//! How long each command of a media job may run.

use std::time::Duration;

/// How long `ffprobe` gets for one file.
pub const PROBE_TIMEOUT: Duration = Duration::from_secs(30);

/// How long `ffmpeg` gets to grab a poster frame.
pub const POSTER_TIMEOUT: Duration = Duration::from_secs(60);

/// The longest `ffmpeg` gets to encode a proxy, whatever the length of the video.
pub const PROXY_TIMEOUT: Duration = Duration::from_secs(15 * 60);

/// How long `ffmpeg` gets to encode the proxy, the filmstrip or the waveform of a file of
/// `duration_ms`: a minute, plus four times its length, at most [`PROXY_TIMEOUT`].
pub fn proxy_limit(duration_ms: Option<i64>) -> Duration {
    let length = Duration::from_millis(duration_ms.unwrap_or(0).max(0) as u64);
    (Duration::from_secs(60) + length * 4).min(PROXY_TIMEOUT)
}

/// Where the poster of a video `duration_ms` long is taken: its middle, at most one second in.
pub fn poster_at(duration_ms: Option<i64>) -> i64 {
    duration_ms.map_or(0, |duration| (duration / 2).clamp(0, 1000))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_files_get_more_time_up_to_a_cap_and_the_poster_sits_early() {
        assert_eq!(proxy_limit(Some(30_000)), Duration::from_secs(180));
        assert_eq!((proxy_limit(None), proxy_limit(Some(3_600_000))), (Duration::from_secs(60), PROXY_TIMEOUT));
        assert_eq!((poster_at(None), poster_at(Some(600)), poster_at(Some(60_000))), (0, 300, 1000));
    }
}
