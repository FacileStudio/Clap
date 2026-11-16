# Clap: rendering

How `crates/render` (package `clap-render`) turns a scene into an MP4, how the render worker
(`apps/worker`) runs it in isolation, and what neither does yet.

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
                 pub range: Option<Range<f64>>, pub output: PathBuf, pub capture: Capture,
                 pub frame_budget: u64 }
impl Job { pub fn new(dir: impl Into<PathBuf>, output: impl Into<PathBuf>) -> Job }
pub enum Capture { BeginFrame, Screenshot }
pub struct Tools { pub chrome: PathBuf, pub ffmpeg: PathBuf, pub sandbox: bool }
impl Tools { pub fn locate() -> Result<Tools, Error> }
pub struct Progress { pub frame: u32, pub frames: u32 }
pub struct Report { pub frames: u32, pub duration: f64, pub frame_hashes: Vec<String>,
                    pub blocked: Vec<String>, pub capture: Capture, pub videos: u32,
                    pub audio_tracks: u32, pub elapsed: Duration }
pub mod tools { pub fn chrome(), ffmpeg(), ffprobe() -> Option<PathBuf> }
```

`render` blocks until the MP4 is complete. The crate has no async runtime: a server calls it
from `tokio::task::spawn_blocking`. Answering `ControlFlow::Break` from the progress callback
cancels the render. The callback is also called, with `frame: 0`, about ten times a second
while videos are being extracted. A cancelled or failed render removes its output file and its
extracted frames, and every Chrome and ffmpeg process it started is killed before the call
returns.

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
then repeats the previous frame. Before frame zero the renderer draws one primed frame, after a
style change that invalidates the paint and changes no pixel, so frame zero always has a frame to
repeat: under load the compositor sometimes reported no change on the very first frame, and the
render failed with "the first frame came back empty".

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
pause, such as a worker's. In production a fourth sits under them: the `clap-render` service
has no network at all.

## Video inside the scene

Chrome never decodes video. Under begin-frame control its decoders skip frames or never become
ready, so the renderer takes decoding away from it, as the article describes.

1. Before any script of the page runs, `src/prelude.js` (injected with
   `Page.addScriptToEvaluateOnNewDocument`) registers a `DOMContentLoaded` listener. It fires
   before the scene runtime's own, and replaces each `<video>` with an `<img>` that carries
   the same attributes: `id`, `class`, `style`, `data-start`, `data-duration`,
   `data-track-index`. The runtime and every timeline are built after the swap, so tweens,
   transforms, opacity, `object-fit` and clip visibility all apply to the `<img>` directly.
   Nothing is copied from one element to another at capture time.
2. Once the scene is loaded, `window.__clap.scan()` answers each media element with its place
   on the project timeline: the start of its scene plus `data-start`, its length cut at the
   end of its scene, `data-in` and `data-volume`. A source that does not resolve to a file
   inside the bundle fails the render.
3. `src/stock.rs` runs one ffmpeg per video, for the frames the render will show and no
   others: `-ss <data-in + offset> -i <file> -vf fps=<fps> -frames:v <count> -q:v 2`, into
   `<bundle>/.clap-frames-XXXXXX/<n>/%06d.jpg`. The directory is inside the bundle, so the
   guard already allows it. Each extraction has a deadline (60 s plus 250 ms per frame) and
   is killed when the caller cancels. The directory may not pass `Job::frame_budget` bytes
   (8 GiB by default) and is removed when the render ends, however it ends.
4. In the capture loop, between `seek` and the shot, `window.__clap.show` sets the `src` of
   each active `<img>` to the still for that frame and resolves when every `load` event has
   fired. With `--disable-checker-imaging` the still is decoded before the frame is drawn.
   A file shorter than its clip holds its last frame.

Two limits follow from the swap. A CSS or timeline selector that names the tag (`video { … }`,
`tl.to("video", …)`) does not reach the `<img>`: select a video clip by id or class. A
timeline target passed as an element object captured before `DOMContentLoaded` still points at
the removed `<video>`.

## Audio

Every `<audio>` and every `<video>` with a sound stream is one input of a single ffmpeg filter
graph, in the same ffmpeg process that encodes the picture (`src/audio.rs`). Each input is
opened at its media offset for its length (`-ss`, `-t`), brought to 48 kHz stereo, scaled by
`data-volume` (1 without it, above 1 amplifies), delayed to its place with `adelay` in samples,
and all are summed with `amix=normalize=0`. The mix is padded and cut to exactly the length of
the picture and encoded as AAC at 192 kb/s. An element with the `muted` attribute or a volume
of 0 is left out. A scene with no sound gives an MP4 with no audio stream. Whether a file has
a sound stream is read from `ffmpeg -i`, so the renderer still needs no `ffprobe`.

## Encoding

`ffmpeg` reads the PNG frames on stdin (`image2pipe`) and writes H.264 (`libx264`, `medium`,
CRF 18, `yuv420p`, `+faststart`). It is started with an argument vector, never through a
shell, and every path is made absolute so it cannot be read as an option or a protocol.
The bit-exact flags and the dropped metadata make the file itself reproducible: two renders
of the title fixture give byte-identical MP4s.

## Tests

`cargo test -p clap-render` renders for real:

- `fixtures/title`, one second at 30 fps, 1280x720: `ffprobe` counts 30 frames of `h264`
  `yuv420p` at 30/1; two runs give the same 30 frame hashes, all different from each other,
  and the same MP4 bytes
- a scene compiled by `facile-scene` with a generated video under a moving, rotating
  transform, a title tween and a sine tone, two seconds at 640x360: 60 frames, an `aac`
  stream, a two second file, the same 60 frame hashes on two runs, and no extracted frame
  left in the bundle
- a generated video whose brightness is its frame number, shown from `data-in="0.2"`: the
  brightness of each of the 15 output frames is the one of the source frame of that instant
- the `Screenshot` path renders 6 frames
- a page that asks for an HTTP image, a `fetch`, a WebSocket and `file:///etc/hostname`
  reaches none of them: a local listener sees no connection and the URLs are in `blocked`
- a `seek` that throws is `Error::Scene`, a `Break` is `Error::Cancelled`, and neither leaves
  a file

The tests print a banner and pass when Chrome, ffmpeg or ffprobe is missing.

## The render worker

Render Chrome loads HTML that users wrote, so it runs in its own service, `clap-render`: the
binary `worker` (`apps/worker`) from the same image as the API. The worker opens no socket and
reads no database. The only thing it shares with `clap-api` is one directory.

### The directory contract

`CLAP_RENDERS_DIR`, `/data/renders` in the image. One directory per job, named by the job id
(1 to 64 letters, digits, `-` and `_`):

| Path | Written by | Meaning |
|---|---|---|
| `<id>/bundle/` | API | The compiled scene: `index.html` and every file it loads. Write it first. |
| `<id>/job.json` | API | Queues the job. Write it last, with a temporary name and a rename. |
| `<id>/cancel` | API | Any file with this name asks the worker to stop. |
| `<id>/status.json` | worker | Where the job is. Rewritten atomically. |
| `<id>/out.mp4` | worker | The video, present only once the job is `done`. |
| `.worker` | worker | The heartbeat: an RFC 3339 timestamp, rewritten every second. |

`job.json`:

```json
{"id": "r_01", "width": 1920, "height": 1080, "fps": 30, "range": [0, 12.5], "created_at": "2026-11-16T10:00:00Z"}
```

`range` is `[start, end]` in seconds, or `null` for the whole scene. `id` must equal the
directory name.

`status.json`:

```json
{"state": "running", "frame": 42, "frames": 375, "error": null, "updated_at": "2026-11-16T10:00:07Z"}
```

`state` is `running`, `done`, `failed` or `cancelled`. `frames` is 0 until the scene has
loaded. A job is queued while it has a `job.json` and no `status.json`. The worker writes the
status when it starts the job, twice a second while it runs, and once at the end.

Both sides must be uid 65532, and the API must leave `bundle/` writable by that uid: the
extracted video frames go under it for the length of the render.

### What the worker does

- It takes the queued job with the oldest `created_at`, one at a time.
- It checks `job.json` against its own limits, whatever the API allowed: the frame at most
  1920x1080 in either orientation and even, at most 30 fps, at most 600 s of video, and one
  hour of wall clock. A scene longer than the limit fails as soon as it has loaded. The
  limits are `CLAP_RENDER_MAX_WIDTH`, `CLAP_RENDER_MAX_HEIGHT`, `CLAP_RENDER_MAX_FPS`,
  `CLAP_RENDER_MAX_SECONDS`, `CLAP_RENDER_TIMEOUT_SECONDS` and `CLAP_RENDER_FRAMES_MAX_BYTES`.
- It renders `bundle/index.html` to `out.mp4.tmp` and renames it to `out.mp4`.
- A `cancel` file ends the job as `cancelled` within half a second plus one frame.
- A job found `running` when no job runs was cut short by a crash or a restart. It is marked
  `failed` with a message that says so, and its partial files are removed. Nothing is retried.
- It follows no symbolic link: a job directory, `bundle`, `job.json` and `status.json` that are
  links are refused, a link where it must write is removed first, and the guard refuses any
  bundle file that resolves outside the bundle.
- `worker healthcheck` exits 0 when the heartbeat is less than 30 s old.

### Isolation

The `clap-render` service of `docker-compose.yml`:

| Setting | Why |
|---|---|
| `network_mode: none` | No route to the database, the Docker network or the internet. |
| `security_opt: seccomp=deploy/chrome-seccomp.json` | Chrome's sandbox needs it, see below. |
| `security_opt: no-new-privileges:true`, `cap_drop: ALL` | Nothing in the container can gain a capability. |
| `read_only: true`, `tmpfs: /tmp` | The only writable places are `/tmp` (Chrome's profile) and the renders directory. |
| `user: 65532:65532` | Not root. |
| volume `clap_data`, `subpath: renders`, at `/data/renders` | The worker sees the renders directory and nothing else of `/data`: no upload, no other project. |
| 4 CPUs, 4 GiB, 1024 PIDs | A render cannot starve the host. |

Chrome's own sandbox is on. Chrome builds it from user, PID and network namespaces and a
`chroot`, which Docker's default seccomp profile refuses to a container without
`CAP_SYS_ADMIN`. `deploy/chrome-seccomp.json` is Docker's default profile (moby
`docker-v29.7.2`) with one rule added that allows `clone`, `unshare` and `chroot`
unconditionally. With the default profile Chrome exits with `No usable sandbox!`. Never set
`--no-sandbox` or `CLAP_CHROME_SANDBOX=off` in the compose file. CI sets it because GitHub's
runners refuse user namespaces to Chrome.

The subpath mount needs Docker Engine 26 or newer, and `renders` must exist in the volume
before the container starts: the one-shot `clap-data-owner` service creates it. A tool that adds
a network to every service of the compose file breaks `network_mode: none`; leave this service
out of it.

## Not here yet

- **The job table.** The API's `modules/renders` owns it: it writes the job directory, reads
  `status.json`, serves `out.mp4` and removes old job directories. The worker never deletes a
  job directory.
- **Speed.** PNG capture of the fixture took about 85 ms per 1080p frame on the development
  host, and `libx264` more than that under load. JPEG capture and several Chrome processes
  over frame ranges are the known levers; neither is needed to be correct.

Back to the [documentation index](README.md).
