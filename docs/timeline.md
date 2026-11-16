# Clap: the timeline

`crates/timeline` (package `clap-timeline`) reads a `clap.md` into the scenes and clips an
editor draws, and writes edits back into the text. The drag timeline in the browser and the
MCP edit tool both go through it, so a drag and an agent edit leave the same kind of diff.

The file format itself is `facile-scene`'s; its reference is `docs/format.md` in
`FacileStudio/Diapo`. This crate adds nothing to the format except two attributes it reads:
`data-in` and `data-clip`.

## The public API

```rust
pub fn read(source: &str) -> Result<Timeline, Error>;
pub fn apply(source: &str, ops: &[Op]) -> Result<String, Error>;

pub struct Timeline { pub title: String, pub width: u32, pub height: u32, pub duration: f64,
                      pub scenes: Vec<Scene> }
impl Timeline { pub fn clip(&self, id: &str) -> Option<&Clip> }
pub struct Scene { pub index: usize, pub start: f64, pub duration: f64, pub fixed_duration: bool,
                   pub layout: String, pub lines: Span, pub clips: Vec<Clip> }
pub struct Clip { pub id: String, pub kind: Kind, pub src: Option<String>, pub start: f64,
                  pub duration: f64, pub track: i64, pub media_in: f64, pub scene: usize,
                  pub lines: Span }
pub struct Span { pub first: usize, pub last: usize }
pub enum Kind { Video, Audio, Image, Text, Html }
pub enum Error { Parse(String), UnknownClip(String), UnknownScene(usize),
                 Overlap { clip, with, track }, OutOfBounds(String), Invalid(String),
                 InvalidResult(String) }
impl Error { pub fn code(&self) -> &'static str }

pub const RUNTIME_JS: &str;
pub fn with_runtime(html: &str) -> String;
pub const CLAP_GUIDE: &str;
pub fn format_guide() -> String;
```

`Timeline` and its parts are `Serialize`; `Op` and `Kind` are also `Deserialize`. The crate has
no I/O and no platform code, and `cargo check -p clap-timeline --target wasm32-unknown-unknown`
passes, so the client can run it in the browser. No JavaScript binding is shipped: until one
is, the API serves the JSON.

## The model

```json
{"title": "Launch", "width": 1280, "height": 720, "duration": 14.0, "scenes": [
  {"index": 0, "start": 0.0, "duration": 6.0, "fixed_duration": true, "layout": "raw",
   "lines": {"first": 7, "last": 22}, "clips": [
    {"id": "shot", "kind": "video", "src": "assets/shot.mp4", "start": 0.0, "duration": 4.0,
     "track": 0, "media_in": 1.0, "scene": 0, "lines": {"first": 12, "last": 12}}]}]}
```

- A **clip** is an element that carries `data-start`, in any layout. Elements inside a fenced
  code block, a comment, a `<script>` or a `<style>` are not clips.
- `start` and `duration` are seconds from the start of the clip's scene. Without
  `data-duration` a clip lasts to the end of its scene.
- `track` is `data-track-index`, 0 without it. `media_in` is `data-in`, 0 without it.
- `kind` is `video`, `audio` or `image` by tag; `text` for an element that holds text and no
  other element; `html` for everything else.
- `lines` are 1-based source lines, both ends included.
- `fixed_duration` says whether the scene's frontmatter sets `duration`. When it does not, the
  scene lasts as long as its last clip and grows with it.

### Clip ids

The id is `data-clip`, else `id`, else `clip-<scene>-<n>` for the n-th clip of its scene, from
0. When two clips would share an id, the later one gets the derived id. A derived id moves when
a clip before it is added or removed, so read the timeline again after every edit. `add_clip`
and `split_clip` always write an explicit id.

## The ops

JSON, tagged by `op`. Times are seconds.

| Op | Fields | Effect on the text |
|---|---|---|
| `move_clip` | `clip`, `start`, `track?` | Sets `data-start`, and `data-track-index` when `track` is given. |
| `trim_clip` | `clip`, `start?`, `duration?`, `media_in?` | Sets `data-start`, `data-duration`, `data-in`. |
| `split_clip` | `clip`, `at` | Shortens the clip to `at` and adds a copy of the element after it, from `at`, with id `<id>-b`. For a video or an audio the copy's `data-in` moves forward by the same amount. |
| `delete_clip` | `clip` | Removes the element, and its lines when it is alone on them. |
| `add_clip` | `scene`, `kind`, `src?`, `start`, `duration`, `track`, `text?` | Adds one line after the last clip of the scene, with id `<kind>-<n>`. `src` is required for `video`, `audio` and `image`. `text` is escaped for a `text` clip and written as HTML for an `html` clip. |
| `set_scene_duration` | `scene`, `duration` | Sets the `duration` key, adding the key or the frontmatter block when missing. |
| `reorder_scene` | `scene`, `to` | Moves the scene's lines so that it sits at index `to`. |
| `delete_scene` | `scene` | Removes the scene's lines. The last scene cannot be removed. |
| `add_scene` | `at`, `duration`, `body` | Inserts a scene at index `at`. A body that starts with `<` gets `layout: raw`. |

```json
[{"op": "move_clip", "clip": "shot", "start": 1.5, "track": 2},
 {"op": "add_scene", "at": 1, "duration": 3, "body": "<h1 id=\"new\">New</h1>"}]
```

`apply` runs the ops in order, each on the result of the one before. It answers the new source
only when every op applied and every intermediate result parses with `scene::parse`; otherwise
it answers the first error and no text.

### What an edit may change

Only the attribute values or the lines the op concerns. Everything else survives byte for byte:
indentation, attribute order, quotes, comments, line endings (`\r\n` files stay `\r\n`), a
missing final newline. A new attribute is written after `data-start`. A number is written with
at most three decimals and no trailing zero; a value that already holds the number is left as
written.

An op followed by its inverse returns the original text when the original wrote its numbers
that way. The property tests check this for move, trim, reorder, set duration, add then delete
a clip, and add then delete a scene. `split_clip` and `delete_clip` have no inverse op: undo
them by restoring the previous source.

### The first scene and the headmatter

The headmatter holds the project keys and the first scene's keys. When an op changes which
scene is first, the project keys, with the comments and blank lines of the block, stay at the
top; the scene keys (`layout`, `steps`, `duration`, `class`) travel with their scene. A scene
whose body opens with a line that looks like `key: value` gets an empty frontmatter block
(`---` twice) when it moves away from the top, so the line stays body text.

### Errors

| `code()` | Variant | When |
|---|---|---|
| `parse` | `Parse` | The source is not a valid project. |
| `unknown_clip` | `UnknownClip` | No clip has that id. |
| `unknown_scene` | `UnknownScene` | No scene has that index. |
| `overlap` | `Overlap` | The overlap policy, below. |
| `out_of_bounds` | `OutOfBounds` | A negative start, a duration of 0 or less, a split outside the clip, a clip that would end after a scene with a fixed duration, or a scene made shorter than one of its clips. |
| `invalid_op` | `Invalid` | The op contradicts itself: a video without `src`, a number that is not finite, deleting the only scene. |
| `invalid_result` | `InvalidResult` | The edit applied and the result does not parse, for example a remote `src` or a scene body that holds a `---` line. |

**The overlap policy.** Two clips of one scene that both carry `data-track-index` may not share
a track and a moment. A clip without `data-track-index` is never checked: titles and overlays
written by hand usually have none and overlap on purpose. Sources that already overlap are
read without complaint; the rule applies to what an op places.

## The preview runtime

`facile-scene`'s runtime hides a clip outside its time but does not drive media. `RUNTIME_JS`
does, for the preview: after each `window.__scene.seek(t)` it sets every shown `<video>` and
`<audio>` to `data-in + (t - scene start - data-start)`, pauses the others, and applies
`data-volume`. `with_runtime(html)` adds it to a compiled document. A parent frame posts
`{type: "clap:playing", playing: true}` to let media play between seeks. Under the renderer the
script does nothing: the renderer swaps frames in itself (see [Rendering](rendering.md)).

## The format guide

`CLAP_GUIDE` is Clap's section of the format guide: time-based scenes, clips, tracks, media,
determinism and the timeline operations. `format_guide()` answers `scene::FORMAT_GUIDE` followed
by it. `clap guide` prints it, and the API's `clap_format_guide` tool should answer the same
constant instead of its own copy (`apps/api/src/mcp/guide.md` holds the same text today).

Back to the [documentation index](README.md).
