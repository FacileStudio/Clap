# Clap: the timeline

`crates/timeline` (package `clap-timeline`) reads a `clap.md` into the scenes and clips an
editor draws, and writes edits back into the text. The drag timeline in the browser and the
MCP edit tool both go through it, so a drag and an agent edit leave the same kind of diff.

The file format itself is `facile-scene`'s; its reference is `docs/format.md` in
`FacileStudio/Diapo`. This crate adds nothing to the format except two attributes it reads:
`data-in` and `data-clip`, the stage attributes `data-x`, `data-y`, `data-w`, `data-h`,
`data-rot`, `data-fit`, `data-opacity`, `data-enter`, `data-exit`, `data-enter-duration` and
`data-exit-duration`, and two scene keys, `transition` and `transition_duration`.

`facile-scene` refuses frontmatter keys it does not define, so `parse` checks the two scene keys
and hands the scene parser the source with them folded into the scene's `class` line, as
`clap-transition-<name>` and `clap-transition-ms-<milliseconds>`, every line keeping its number.
The compiled section carries the classes and the runtime reads the transition from them. Every
place that parses a Clap source calls `clap_timeline::parse`, never `scene::parse`.

## The public API

```rust
pub fn read(source: &str) -> Result<Timeline, Error>;
pub fn apply(source: &str, ops: &[Op]) -> Result<String, Error>;
pub fn parse(source: &str) -> Result<scene::Project, scene::Errors>;

pub struct Timeline { pub title: String, pub width: u32, pub height: u32, pub duration: f64,
                      pub scenes: Vec<Scene> }
impl Timeline { pub fn clip(&self, id: &str) -> Option<&Clip> }
pub struct Scene { pub index: usize, pub start: f64, pub duration: f64, pub fixed_duration: bool,
                   pub layout: String, pub lines: Span, pub transition: Option<String>,
                   pub transition_duration: Option<f64>, pub clips: Vec<Clip> }
pub struct Clip { pub id: String, pub kind: Kind, pub src: Option<String>, pub start: f64,
                  pub duration: f64, pub track: i64, pub media_in: f64, pub scene: usize,
                  pub lines: Span, pub look: Look }
pub struct Look { pub frame: Option<Geometry>, pub fit: Option<String>, pub opacity: Option<f64>,
                  pub volume: Option<f64>, pub enter: Option<String>, pub exit: Option<String>,
                  pub enter_duration: Option<f64>, pub exit_duration: Option<f64> }
pub struct Geometry { pub x: f64, pub y: f64, pub w: f64, pub h: f64, pub rot: f64 }
pub struct Update { pub x: Option<Option<f64>>, /* and y, w, h, rot, fit, opacity, volume,
                    enter, exit, enter_duration, exit_duration */ }
pub const CLIP_TRANSITIONS: [&str; 11];
pub const SCENE_TRANSITIONS: [&str; 7];
pub const FITS: [&str; 3];
pub struct Span { pub first: usize, pub last: usize }
pub enum Kind { Video, Audio, Image, Text, Html }
pub enum Error { Parse(String), UnknownClip(String), UnknownScene(usize),
                 Overlap { clip, with, track }, OutOfBounds(String), Invalid(String),
                 InvalidResult(String) }
impl Error { pub fn code(&self) -> &'static str }

pub const RUNTIME_JS: &str;
pub const STAGE_CSS: &str;
pub fn with_runtime(html: &str) -> String;
pub fn starter(locale: &str) -> String;
pub fn starter_language(locale: &str) -> &'static str;
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
     "track": 0, "media_in": 1.0, "scene": 0, "lines": {"first": 12, "last": 12},
     "frame": {"x": 0.0, "y": 0.0, "w": 1280.0, "h": 720.0, "rot": 0.0}, "fit": "cover",
     "opacity": null, "volume": null, "enter": "fade", "exit": null, "enter_duration": null,
     "exit_duration": null}], "transition": null, "transition_duration": null}]}
```

- A **clip** is an element that carries `data-start`, in any layout. Elements inside a fenced
  code block, a comment, a `<script>` or a `<style>` are not clips.
- `start` and `duration` are seconds from the start of the clip's scene. Without
  `data-duration` a clip lasts to the end of its scene.
- `track` is `data-track-index`, 0 without it. `media_in` is `data-in`, 0 without it.
- `kind` is `video`, `audio` or `image` by tag; `text` for an element that holds text and no
  other element; `html` for everything else.
- `lines` are 1-based source lines, both ends included.
- `frame` is the box of `data-x`, `data-y`, `data-w`, `data-h` and `data-rot` in frame pixels,
  `null` without any of them; one that is missing reads as the full frame's. `fit`, `opacity`,
  `volume`, `enter`, `exit`, `enter_duration` and `exit_duration` are the attribute, `null`
  without it or when the runtime would ignore its value. In Rust they sit in `clip.look`.
- `transition` and `transition_duration` are the scene's frontmatter keys, `null` without.
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
| `add_clip` | `scene`, `kind`, `src?`, `start`, `duration`, `track`, `text?` | Adds one line after the last clip of the scene, with id `<kind>-<n>` and a stage class: `clap-fill` on a video or an image, which also get the full-frame box (`data-x="0" data-y="0"`, the stage size, `data-rot="0"`) and `data-fit="cover"`, `clap-title` on a text, `clap-layer` on an HTML clip, none on an audio. `src` is required for `video`, `audio` and `image`. `text` is escaped for a `text` clip and written as HTML for an `html` clip. |
| `duplicate_clip` | `clip`, `scene?`, `start`, `track?` | Copies the element as written into scene `scene` (its own without), at `start`, on `track` (its own without), as one line after the last clip of that scene. The copy is named by `data-clip`: `<id>-copy`, else `<id>-copy-<n>` from 2. |
| `set_scene_duration` | `scene`, `duration` | Sets the `duration` key, adding the key or the frontmatter block when missing. |
| `reorder_scene` | `scene`, `to` | Moves the scene's lines so that it sits at index `to`. |
| `delete_scene` | `scene` | Removes the scene's lines. The last scene cannot be removed. |
| `add_scene` | `at`, `duration`, `body` | Inserts a scene at index `at`. A body that starts with `<` gets `layout: raw`. |
| `duplicate_scene` | `scene` | Inserts a copy of the scene's lines right after it. Every clip of the copy gets a `data-clip` named as `duplicate_clip` names a copy; the project keys of the first scene stay where they are. |
| `update_clip` | `clip`, `x?`, `y?`, `w?`, `h?`, `rot?`, `fit?`, `opacity?`, `volume?`, `enter?`, `exit?`, `enter_duration?`, `exit_duration?` | Sets each attribute given (`data-x`, ..., `data-enter-duration`); a JSON `null` removes it with the spaces before it. `w`, `h` and the durations are above 0, `opacity` and `volume` from 0 to 1, `fit` one of `FITS`, `enter` and `exit` one of `CLIP_TRANSITIONS`. |
| `set_scene_transition` | `scene`, `transition`, `duration?` | Sets the `transition` key, and `transition_duration` when `duration` is a number. A `null` transition removes both keys, a `null` duration that one; a frontmatter block left empty goes too. `transition` is one of `SCENE_TRANSITIONS`. |

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
a clip, add then delete a scene, an `update_clip` then the one that removes what it wrote, and a
transition set then cut. The inverse of `update_clip` names the previous value of each field, or
`null`; the inverse of `set_scene_transition` names the previous transition and duration. `split_clip` and `delete_clip` have no inverse op: undo
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

## The stage runtime

`facile-scene`'s runtime hides a clip outside its time but neither places clips nor drives
media. `RUNTIME_JS` wraps `window.__scene.seek` and runs after every seek, also the seeks a
parent posts as `scene:seek`. `with_runtime(html)` adds it to a compiled document.

**Layout and transitions, in the preview and under the renderer.** Each clip that carries a
stage attribute gets an inline `position: absolute` box from `data-x`, `data-y`, `data-w` and
`data-h` as percentages of the stage, `rotate(data-rot)`, `object-fit` from `data-fit` (`cover`
without it) and `data-opacity`. Its enter and exit presets apply at the seek time: a progress
`p` from 0 to 1 over the preset's duration (0.5 s without one, at most half the clip), eased
cubic in-out, drives opacity, a translation by the clip's own size, a scale, a blur or an
`inset()` clip path. A scene transition (0.6 s without `transition_duration`, at most the
scene) does the same to the incoming section. `fade`, the slides, the wipes and `zoom` show the
scene before under it: the seek first draws that scene at `start + duration - 0.0001`, then the
asked time, and keeps the earlier section visible, so its clips stand where they ended. `dip`
fades the scene before out over the last half of the duration and this one in over the first
half, over a black stage. Nothing depends on a clock, so a scrub and a frame-by-frame render
show the same frame.

**Media, in the preview only.** Every shown `<video>` and `<audio>` goes to
`data-in + (t - scene start - data-start)`, the others pause, `data-volume` applies, multiplied
by a linear ramp for `data-enter="fade"` and `data-exit="fade"`. The video of a scene held under
a transition stays on its last frame. A parent posts `{type: "clap:playing", playing}` to let
media play between seeks. Under the renderer the renderer swaps frames in itself and reads
`window.__clapLayout.sound(el)` and `.hold(el)` for the fades and the held frames (see
[Rendering](rendering.md)).

**Messages to the editor, in the preview only.** After every seek the document posts to its
parent

```json
{"type": "clap:boxes", "t": 2.5, "clips": [
  {"id": "image-1", "kind": "image", "x": 100, "y": 100, "w": 200, "h": 200, "rot": 0}]}
```

with each clip shown at `t` (not hidden by its time, not an `<audio>`): `kind` is `video`,
`image`, `text` or `html`; the box is the clip's attributes in stage pixels, unanimated, or for
a clip without any, its measured bounding box with `rot` 0. Ids follow the rules of "Clip ids".
The parent may post `{type: "clap:ghost", id, x, y, w, h, rot}` to draw one clip at a temporary
box during a drag; it lasts until the next `clap:ghost`, and one without a valid box (`w` and
`h` above 0, every value finite) clears it. Each ghost redraws and posts `clap:boxes` again.
The client reads `clap:boxes` with `stageBoxes` (`apps/client/src/lib/project/stage.ts`), which
treats the frame as untrusted.

**Editing on the stage.** The editor draws the boxes over the preview
(`components/editor/StageOverlay.svelte`, pure helpers in `apps/client/src/lib/stage/`). A drag
moves, a corner resizes (a video or a picture keeps its ratio; Shift frees it, or keeps it for a
text), the round handle turns (Shift steps by 15 degrees), and arrows nudge by 1 pixel, 10 with
Shift, while the stage has the focus. Edges and centres snap to the stage edges, centre and
thirds and to the other boxes; Alt holds the snap off. The gesture shows live through
`clap:ghost` and ends as one `update_clip` with the whole box, so one undo puts it back; the ghost
stays until the reloaded preview draws the new box. A file or a media bin item dropped on the
stage is added at the playhead above what plays there, followed in the same edit by an
`update_clip` naming the id `add_clip` will give it (`<kind>-<n>`, the first free one), with a box
centred on the drop point at the media's ratio (its own size, at most half the stage) and
`fit: contain`. The Clip tab edits the same values by number. A right-click on a box, or Shift+F10
while the stage has the focus, opens its menu: bring forward or send backward (the clip's track,
which the runtime stacks by), fill or fit the frame, centre, reset the size, copy, duplicate and
delete.

## The stage

`STAGE_CSS` is the stylesheet `with_runtime` puts at the end of the document's head, so the
preview, the API's frame and every render (API and `clap render`) draw the same picture. A raw
scene becomes a black stage with white text, and every element with `data-track-index` is
positioned, so the track the scene runtime writes as `z-index` stacks the clips. It defines the
classes `add_clip` writes, documented in the guide's "The stage": `clap-fill`, `clap-layer`,
`clap-title`, `clap-caption`, `clap-lower`. Stage rules sit in `:where()` and each class is one
class selector, so the theme's element rules lose to them and any author rule wins.

## The starter

`starter(locale)` is the source a new project begins with: one raw scene of 4 seconds, a
centered title that rises in (`clap-title`) and a hint line under it (`clap-caption`), animated
with `scene.timeline`. `locale` is an `Accept-Language` value or a tag; the starter is written in
French, English, German or Spanish, French when none matches. The API answers it for
`POST /projects` without a source, and `clap new` without `--template` writes it.

## The format guide

`CLAP_GUIDE` is Clap's section of the format guide: time-based scenes, clips, tracks, media,
determinism and the timeline operations. `format_guide()` answers `scene::FORMAT_GUIDE` followed
by it. `clap guide` prints it, and the API's `clap_format_guide` tool should answer the same
constant instead of its own copy (`apps/api/src/mcp/guide.md` holds the same text today).

Back to the [documentation index](README.md).
