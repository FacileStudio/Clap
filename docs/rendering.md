# Clap: rendering

How `crates/render` (package `clap-render`) turns a scene into an MP4, why it is built this
way, and what it does not do yet.

## The contract

A scene is a directory holding an `index.html`. The page exposes one object:

```js
window.__scene = { duration, seek(t) }
```

`duration` is the length in seconds. `seek(t)` puts every element where it belongs at second
`t`; it may return a promise. The renderer never plays the page. For frame `n` at `fps` it
calls `seek(n / fps)` and captures. The frame index is an integer, so the time handed to
`seek` is the same number on every run and on every path that reaches that frame. The preview
in the editor will drive the same `seek`.

The technique is HeyGen's, described at https://www.heygen.com/research/html-to-video.

## The public API

```rust
pub fn render(job: &Job, tools: &Tools) -> Result<Report, Error>;
pub fn render_with(job: &Job, tools: &Tools, progress: OnProgress) -> Result<Report, Error>;
pub type OnProgress<'a> = &'a mut dyn FnMut(Progress) -> ControlFlow<()>;

pub struct Job { pub dir: PathBuf, pub width: u32, pub height: u32, pub fps: u32,
                 pub range: Option<Range<f64>>, pub output: PathBuf, pub capture: Capture }
impl Job { pub fn new(dir: impl Into<PathBuf>, output: impl Into<PathBuf>) -> Job }
pub enum Capture { BeginFrame, Screenshot }
pub struct Tools { pub chrome: PathBuf, pub ffmpeg: PathBuf, pub sandbox: bool }
impl Tools { pub fn locate() -> Result<Tools, Error> }
pub struct Progress { pub frame: u32, pub frames: u32 }
pub struct Report { pub frames: u32, pub duration: f64, pub frame_hashes: Vec<String>,
                    pub blocked: Vec<String>, pub capture: Capture, pub elapsed: Duration }
pub mod tools { pub fn chrome(), ffmpeg(), ffprobe() -> Option<PathBuf> }
```

`render` blocks until the MP4 is complete. The crate has no async runtime: a server calls it
from `tokio::task::spawn_blocking`. Answering `ControlFlow::Break` from the progress callback
cancels the render. A cancelled or failed render removes its output file, and dropping the
browser and the encoder ends both processes.

`Job::new` defaults to 1920x1080 at 30 fps, the whole scene. Width and height must be even
(H.264 with 4:2:0 chroma). The crate enforces no product limit: the caller owns the caps on
size, rate and length.

## Finding the binaries

`Tools::locate` reads `CLAP_CHROME` and `CLAP_FFMPEG`, then searches `PATH` for
`chrome-headless-shell` and `ffmpeg`, then, for Chrome, takes the newest
`chromium_headless_shell-*` build in Playwright's cache. `tools::ffprobe` does the same with
`CLAP_FFPROBE`. An environment variable that names a missing file is an error, not a reason to
fall through.

## Talking to Chrome

`clap-render` speaks the DevTools Protocol itself, over `--remote-debugging-pipe`: JSON
messages ended by a NUL byte, commands on the child's file descriptor 3, answers and events on
4. The whole client is `src/pipe.rs` and `src/cdp.rs`, under 170 lines.

Why not a CDP crate such as `chromiumoxide`:

- It needs a dozen protocol methods. A generated binding for the whole protocol is thousands of
  types to compile and to keep in step with the pinned Chrome.
- `chromiumoxide` attaches over a WebSocket on a local port. Any process on the host can
  attach to that port and drive a browser that is rendering somebody's project. A pipe is
  reachable only by the process that spawned Chrome.
- It brings an async runtime and its own event loop. Rendering is one command at a time, so a
  blocking call with a deadline is the whole concurrency model: `Browser::call` writes a
  command, then reads messages until its answer arrives, handling events on the way.
- `HeadlessExperimental.beginFrame` is experimental. Sending it as plain JSON keeps the crate
  independent of whether a binding exposes it.

The one dependency this costs is `command-fds`, which maps the two pipe ends onto descriptors
3 and 4 of the child without `unsafe` in this crate (`#![deny(unsafe_code)]`).

## Capture

Two paths, chosen by `Job::capture`. The default is `BeginFrame` on Linux and `Screenshot`
elsewhere.

**BeginFrame (Linux).** Chrome runs with `--deterministic-mode --enable-begin-frame-control`
and the compositor flags of `src/chrome.rs`. The compositor then draws only when
`HeadlessExperimental.beginFrame` asks, at the frame time the call names, and returns the
screenshot of exactly that frame. One seek, one `beginFrame`, one frame.

Under begin-frame control the page's event loop does not advance on its own, so a page that
waits for fonts would never finish loading. `Page::load` therefore runs the warm-up loop the
article describes: before each readiness check it sends a `beginFrame` with
`noDisplayUpdates: true`, 33 ms apart, starting at frame time 1000 ms. Capture starts at
frame time 100000 ms, past any warm-up (the load deadline is 20 s), so time never runs
backwards and the frame times of the capture do not depend on how long loading took.

`beginFrame` may return no picture when nothing changed since the previous frame. The loop
then repeats the previous frame.

**Screenshot (macOS, Windows).** No deterministic flags. After each seek the renderer waits
for two `requestAnimationFrame` callbacks, then calls `Page.captureScreenshot`. This path is
not guaranteed to be identical across runs.

On both paths a page is ready when `document.readyState` is `complete`, `document.fonts` has
finished loading and `window.__scene` has a `seek` function and a positive `duration`.

Frames are PNG, so capture loses nothing. Each frame's SHA-256 is in `Report::frame_hashes`.

## Network isolation

The page may load `file://` URLs that resolve inside the bundle directory, and `data:`,
`blob:` and `about:` URLs. Everything else is refused. Three layers:

1. `Fetch.enable` pauses every request. `Guard` (`src/guard.rs`) canonicalises each `file://`
   path, so `..` and a symlink that leaves the bundle are refused, and fails anything else
   with `BlockedByClient`. Refused URLs are in `Report::blocked`.
2. `--proxy-server=http://127.0.0.1:1 --proxy-bypass-list=<-loopback>` sends every HTTP,
   HTTPS and WebSocket connection, loopback included, to a port nothing listens on.
3. `--host-resolver-rules=MAP * ~NOTFOUND` fails every name lookup, and WebRTC is held to
   proxied transports.

Layer 1 is the allowlist. Layers 2 and 3 are there for requests the page session does not
pause, such as a worker's. The production container should add a fourth: no route out.

## Encoding

`ffmpeg` reads the PNG frames on stdin (`image2pipe`) and writes H.264 (`libx264`, `medium`,
CRF 18, `yuv420p`, `+faststart`). It is started with an argument vector, never through a
shell, and the output path is made absolute so it cannot be read as an option or a protocol.
The bit-exact flags and the dropped metadata make the file itself reproducible: two renders
of the fixture give byte-identical MP4s.

## Tests

`cargo test -p clap-render` renders `fixtures/title` for real:

- one second at 30 fps, 1280x720: `ffprobe` counts 30 frames of `h264` `yuv420p` at 30/1;
  two runs give the same 30 frame hashes, all different from each other, and the same MP4
  bytes
- the `Screenshot` path renders 6 frames
- a page that asks for an HTTP image, a `fetch`, a WebSocket and `file:///etc/hostname`
  reaches none of them: a local listener sees no connection and the URLs are in `blocked`
- a `seek` that throws is `Error::Scene`, a `Break` is `Error::Cancelled`, and neither leaves
  a file

The tests print a banner and pass when Chrome, ffmpeg or ffprobe is missing.

## Not in this crate yet

Each has a marked place to go.

- **Video inside the scene.** Chrome's video decoders are not frame-exact under begin-frame
  control. The plan is the article's: before capture, `ffmpeg` extracts each source video to
  frames at the target rate into the bundle directory, and each `<video>` is swapped for the
  frame of the current time. The swap goes in the capture loop of `src/lib.rs`, between
  `Page::seek` and `Page::shot`. Extracted frames sit under the bundle, so the guard already
  allows them.
- **Audio.** The output has no audio track (`-an`). Audio inputs and their filter graph join
  `encode::args`; mixing needs each clip's start and duration, which `crates/timeline` will
  own.
- **A job runner.** One render at a time, a queue, a hard timeout and `nice` belong to the
  API's `modules/renders`, on top of `render_with`.
- **Speed.** PNG capture of the fixture took about 85 ms per 1080p frame on the development
  host, and `libx264` more than that under load. JPEG capture and several Chrome processes
  over frame ranges are the known levers; neither is needed to be correct.

Back to the [documentation index](README.md).
