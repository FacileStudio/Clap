//! One page under the `window.__scene` contract: load it, seek it, take its picture.

use std::time::{Duration, Instant};

use serde_json::{Value, json};
use url::Url;

use crate::Error;
use crate::cdp::Browser;
use crate::job::{Capture, Job, entry};
use crate::media::{self, Media};

/// What "loaded" means: the document, its fonts and a seekable scene.
const READY: &str = "document.readyState === 'complete' && document.fonts.status === 'loaded' \
    && !!window.__scene && typeof window.__scene.seek === 'function' && window.__scene.duration > 0";
/// Resolves once two animation frames have run, so the seek has been painted.
const PAINTED: &str = "new Promise(done => requestAnimationFrame(() => requestAnimationFrame(() => done(true))))";
/// Runs before any script of the page: swaps each `<video>` for an `<img>` the capture loop
/// feeds, silences each `<audio>`, and exposes `window.__clap.scan` and `window.__clap.show`.
const PRELUDE: &str = include_str!("prelude.js");
/// How long a page gets to become ready.
const READY_TIMEOUT: Duration = Duration::from_secs(20);
/// The pause between two readiness checks, and the frame time one warm-up frame advances.
const WARMUP_STEP: Duration = Duration::from_millis(33);
/// The frame time of the first warm-up frame, in milliseconds.
const WARMUP_START: f64 = 1_000.0;
/// The frame time of frame zero, past any warm-up so time never runs backwards.
const CAPTURE_START: f64 = 100_000.0;

/// A page attached over CDP.
pub(crate) struct Page<'a> {
    browser: &'a mut Browser,
    session: String,
    capture: Capture,
    interval: f64,
    warmed: f64,
}

fn text(value: &Value, what: &str) -> Result<String, Error> {
    value.as_str().map(str::to_owned).ok_or_else(|| Error::Chrome(format!("no {what} in the answer")))
}

impl<'a> Page<'a> {
    /// Opens a blank page of the job's size with every request paused for the guard.
    pub(crate) fn open(browser: &'a mut Browser, job: &Job) -> Result<Page<'a>, Error> {
        let begin = job.capture == Capture::BeginFrame;
        let target =
            json!({"url": "about:blank", "width": job.width, "height": job.height, "enableBeginFrameControl": begin});
        let target = text(&browser.call(None, "Target.createTarget", target)?["targetId"], "targetId")?;
        let attach = json!({"targetId": target, "flatten": true});
        let session = text(&browser.call(None, "Target.attachToTarget", attach)?["sessionId"], "sessionId")?;
        let interval = 1000.0 / f64::from(job.fps);
        let mut page = Page { browser, session, capture: job.capture, interval, warmed: WARMUP_START };
        page.call("Page.enable", json!({}))?;
        page.call("Page.addScriptToEvaluateOnNewDocument", json!({"source": PRELUDE}))?;
        page.call("Fetch.enable", json!({"patterns": [{"urlPattern": "*"}]}))?;
        let metrics = json!({"width": job.width, "height": job.height, "deviceScaleFactor": 1, "mobile": false});
        page.call("Emulation.setDeviceMetricsOverride", metrics)?;
        Ok(page)
    }

    fn call(&mut self, method: &str, params: Value) -> Result<Value, Error> {
        self.browser.call(Some(&self.session), method, params)
    }

    fn eval(&mut self, expression: &str) -> Result<Value, Error> {
        let params = json!({"expression": expression, "returnByValue": true, "awaitPromise": true});
        let mut answer = self.call("Runtime.evaluate", params)?;
        match answer.get("exceptionDetails") {
            Some(thrown) => {
                let reason = thrown["exception"]["description"].as_str().or(thrown["text"].as_str());
                Err(Error::Scene(reason.unwrap_or("the page threw").to_owned()))
            }
            None => Ok(answer["result"]["value"].take()),
        }
    }

    fn warm(&mut self) -> Result<(), Error> {
        if self.capture == Capture::BeginFrame {
            let frame = json!({"frameTimeTicks": self.warmed, "interval": self.interval, "noDisplayUpdates": true});
            self.call("HeadlessExperimental.beginFrame", frame)?;
            self.warmed += WARMUP_STEP.as_secs_f64() * 1000.0;
        }
        Ok(())
    }

    /// Loads the bundle's `index.html`, waits for the scene and answers its duration in seconds.
    /// Under begin-frame control the page's event loop only advances when a frame is asked, so
    /// each readiness check is preceded by a frame that draws nothing.
    pub(crate) fn load(&mut self) -> Result<f64, Error> {
        let url = Url::from_file_path(entry(self.browser.guard.root()))
            .map_err(|()| Error::Job("the bundle path is not absolute".to_owned()))?;
        let answer = self.call("Page.navigate", json!({"url": url.as_str()}))?;
        if let Some(reason) = answer["errorText"].as_str() {
            return Err(Error::Scene(format!("index.html did not load: {reason}")));
        }
        let deadline = Instant::now() + READY_TIMEOUT;
        loop {
            self.warm()?;
            if self.eval(READY)? == Value::Bool(true) {
                break;
            }
            if Instant::now() > deadline {
                return Err(Error::Scene(format!("window.__scene was not ready within {READY_TIMEOUT:?}")));
            }
            std::thread::sleep(WARMUP_STEP);
        }
        let duration = self.eval("window.__scene.duration")?;
        duration.as_f64().ok_or_else(|| Error::Scene("window.__scene.duration is not a number".to_owned()))
    }

    /// The `<video>` and `<audio>` elements of the loaded page, as `prelude.js` found them.
    pub(crate) fn media(&mut self) -> Result<Vec<Media>, Error> {
        let scan = self.eval("window.__clap.scan()")?;
        media::read(&scan, &self.browser.guard)
    }

    /// Hands each swapped video the still it shows now, and waits until every one has loaded.
    pub(crate) fn show(&mut self, stills: &[(usize, &str)]) -> Result<(), Error> {
        if stills.is_empty() {
            return Ok(());
        }
        self.eval(&format!("window.__clap.show({})", json!(stills))).map(|_| ())
    }

    /// Puts the scene at `seconds`. The page never plays: every frame is a seek.
    pub(crate) fn seek(&mut self, seconds: f64) -> Result<(), Error> {
        self.eval(&format!("window.__scene.seek({seconds:?})")).map(|_| ())
    }

    /// The PNG of frame `index`, base64. `None` when the compositor saw nothing change since
    /// the frame before.
    pub(crate) fn shot(&mut self, index: u64) -> Result<Option<String>, Error> {
        let picture = json!({"format": "png", "optimizeForSpeed": true});
        if self.capture == Capture::Screenshot {
            self.eval(PAINTED)?;
            let answer = self.call("Page.captureScreenshot", picture)?;
            return text(&answer["data"], "screenshot").map(Some);
        }
        let ticks = CAPTURE_START + index as f64 * self.interval;
        let frame = json!({"frameTimeTicks": ticks, "interval": self.interval, "screenshot": picture});
        let answer = self.call("HeadlessExperimental.beginFrame", frame)?;
        Ok(answer["screenshotData"].as_str().map(str::to_owned))
    }
}
