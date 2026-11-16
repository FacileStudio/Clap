//! The flags `chrome-headless-shell` runs with.

use crate::job::{Capture, Job};
use crate::tools::Tools;

/// Quiet, offline and reproducible, whatever the capture path.
const BASE: &[&str] = &[
    "--remote-debugging-pipe",
    "--no-first-run",
    "--no-default-browser-check",
    "--disable-background-networking",
    "--disable-component-update",
    "--disable-default-apps",
    "--disable-extensions",
    "--disable-sync",
    "--disable-breakpad",
    "--disable-dev-shm-usage",
    "--metrics-recording-only",
    "--mute-audio",
    "--hide-scrollbars",
    "--disable-gpu",
    "--force-device-scale-factor=1",
    "--force-color-profile=srgb",
    "--font-render-hinting=none",
    "--proxy-server=http://127.0.0.1:1",
    "--proxy-bypass-list=<-loopback>",
    "--host-resolver-rules=MAP * ~NOTFOUND",
    "--force-webrtc-ip-handling-policy=disable_non_proxied_udp",
];

/// The compositor runs on the main thread and only when a `beginFrame` asks.
const DETERMINISTIC: &[&str] = &[
    "--deterministic-mode",
    "--enable-begin-frame-control",
    "--run-all-compositor-stages-before-draw",
    "--disable-threaded-animation",
    "--disable-threaded-scrolling",
    "--disable-checker-imaging",
    "--disable-image-animation-resync",
    "--enable-surface-synchronization",
];

/// Every flag for `job`, without the profile directory.
pub(crate) fn flags(job: &Job, tools: &Tools) -> Vec<String> {
    let mut flags: Vec<String> = BASE.iter().map(|flag| (*flag).to_owned()).collect();
    flags.push(format!("--window-size={},{}", job.width, job.height));
    if job.capture == Capture::BeginFrame {
        flags.extend(DETERMINISTIC.iter().map(|flag| (*flag).to_owned()));
    }
    if !tools.sandbox {
        flags.push("--no-sandbox".to_owned());
    }
    flags
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_network_is_cut_and_the_sandbox_stays_unless_asked() {
        let job = Job { capture: Capture::Screenshot, ..Job::new("/bundle", "out.mp4") };
        let tools = Tools { chrome: "chrome".into(), ffmpeg: "ffmpeg".into(), sandbox: true };
        let plain = flags(&job, &tools);
        assert!(plain.contains(&"--proxy-server=http://127.0.0.1:1".to_owned()));
        assert!(plain.contains(&"--window-size=1920,1080".to_owned()));
        assert!(!plain.iter().any(|flag| flag == "--no-sandbox" || flag == "--deterministic-mode"));
        let job = Job { capture: Capture::BeginFrame, ..job };
        let begin = flags(&job, &Tools { sandbox: false, ..tools });
        assert!(begin.contains(&"--enable-begin-frame-control".to_owned()));
        assert!(begin.contains(&"--no-sandbox".to_owned()));
    }
}
