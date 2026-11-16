# Changelog

All notable changes to Clap. Versions follow SemVer.

## [Unreleased]

### Fixed

- Images no longer flicker in the preview after a drop or an edit. The preview keeps the
  previous frame until the new one has decoded the images and loaded the videos it shows (2.5 s
  at most), and asset links signed within the same ten minutes keep one expiry, so a reloaded
  preview reuses the files the browser already holds instead of fetching them again.

## [0.2.1] - 2026-11-16

### Security

- Uploads are decoded in the render worker's sandbox, never in `clap-api`, which holds the
  database URL and the sign-in secrets. The API copies each upload into a media job; the worker
  (no network, its own uid, the seccomp profile, its CPU and memory caps) runs `ffprobe` and
  makes the poster, filmstrip, waveform and proxy, and the API reads each file back as
  untrusted: no link, one hard link, a size cap per file, and a re-checked waveform. The
  argument vectors live in a shared crate, `crates/media`, so the two sides cannot drift.
  Media jobs go before renders in the worker's queue and do not count toward
  `RENDER_QUEUE_MAX`.

## [0.2.0] - 2026-11-16

### Added

- Clips are edited on the stage: click to select, drag to move, corner handles to resize (media
  keeps its ratio, Shift frees it), a handle to rotate, arrows to nudge, with snapping guides on
  the stage edges, centre, thirds and the other clips. Each gesture is one undoable edit.
- Media dropped on the stage, from the bin or the desktop, lands at the playhead in a box centred
  on the drop point, fitted whole.
- A Clip tab sets a clip's box, rotation, fit, opacity, volume, start, length and its enter and
  exit presets, with "Fill the stage" and "Centre" actions.
- The timeline draws each clip's enter and exit ramps, and a chip at each scene boundary picks the
  scene transition and its length.
- The timeline draws what a clip shows: a video its filmstrip from the frame its trim starts
  on, a sound its waveform, an image its picture. The API makes a filmstrip (`strip_url`,
  `strip_tiles`, `strip_step`) and a waveform (`peaks_url`) for each upload, and for the media
  already uploaded at the next boot.

## [0.1.1] - 2026-11-16

### Added

- Files dragged from the desktop onto the timeline upload and land at the drop time and track.
  While a file or a media bin item is aimed, a ghost clip shows where it will land and for how
  long; while it uploads, the ghost stays with the file's first frame and the upload progress.
  The browser reads the length and the first frame itself, so a clip is placed before the
  server's probe ends, and the media bin can place a file while it is still being analysed.
- Clips have a box on the stage: `data-x`, `data-y`, `data-w`, `data-h` and `data-rot` in frame
  pixels, with `data-fit` (cover, contain, fill) and `data-opacity`. The timeline model answers
  them as `frame`, `fit`, `opacity`, `volume`, `enter`, `exit` and their durations.
- Transitions computed from the seek time alone, so the preview and the render agree frame for
  frame: eleven clip presets for `data-enter` and `data-exit` (fade, slides, zooms, wipes, blur,
  pop; 0.5 s by default, eased), and seven scene transitions in the frontmatter key `transition`
  with `transition_duration` (a real crossfade with the scene before, a dip through black, push
  slides, wipes and zoom; 0.6 s by default). A fading clip fades its sound too, in the render's
  mix and in the preview.
- Timeline ops `update_clip` (a JSON `null` removes an attribute) and `set_scene_transition`,
  undoable from the editor and available to the MCP `clap_timeline_edit` tool.
- After every seek the preview posts `clap:boxes`, the stage boxes of the clips shown, and draws
  a dragged clip where a `clap:ghost` message says.

### Changed

- `add_clip` writes an explicit full-frame box and `data-fit="cover"` on a video or an image.
- `facile-scene` is pinned to Diapo v0.2.0. Clap sources are parsed with
  `clap_timeline::parse`, which accepts the two transition keys the scene format does not know.
- A raw scene is a black stage on which clips stack by track, through a stylesheet the preview,
  the frame and every render share (`clap_timeline::STAGE_CSS`). `add_clip` gives a video or an
  image a full-frame `clap-fill`, a text a centered `clap-title` that reads on video, and an
  HTML clip a `clap-layer`. The guide documents the classes.
- A new project starts with a welcome scene in the caller's language (French by default; English,
  German or Spanish from `Accept-Language`, the `lang` field, or the CLI's locale) instead of a
  bare "New project" heading.

### Fixed

- A render no longer fails now and then with "the first frame came back empty" on a loaded
  host: the capture primes one frame before frame zero.
- Rendered videos set their text in a sans-serif face again: the render image mapped `sans-serif`
  and `system-ui` to a monospace font.
- Tabs match the button height and keep even padding, so the editor panel tabs line up (muse v0.32.0).

## [0.1.0] - 2026-11-16

### Added

- Share links: one live link per project, shown once and stored hashed, that opens the latest
  finished render at `/v/{token}`, a framable player at `/embed/{token}`, oEmbed at `/oembed`
  for Prisme, and a poster for every finished render.

- Projects: one source text per project in the scene format, with `rev`-checked saves (`409` on
  a stale `rev`, `422` with each invalid line), revisions, trash, spaces mirrored from Registre,
  and server-sent events for open editors.
- The sandboxed preview frame, whose media load through signed, cookie-less links.
- Media: uploads streamed to disk up to 2 GiB, typed by their content, probed with `ffprobe`,
  with a poster and a 720p proxy made by `ffmpeg`, served with `Range`.
- Renders: the API queues a job directory for the render worker, mirrors its progress, keeps
  the finished MP4 and serves it. Limits: 1920x1080, 30 fps, 10 minutes.
- MCP at `/mcp` with eleven tools (`clap_list_projects` to `clap_delete_project`), scopes
  `clap.read` and `clap.write`, and deletion behind a confirmation.
- `POST /api/renders`: another suite app posts a compiled bundle with a Registre token it
  exchanged for the person, and gets a render job.
- Timeline edits through `clap-timeline`: `GET /api/projects/{id}/timeline`,
  `POST /api/projects/{id}/edits` and the `clap_timeline_edit` tool.
- Registre directory sync, account deletion with erasure, and `GET /api/assistant`.

- The app shell: a Rust API on tronc-rs and porte-rs with Registre sign-in, health probes,
  `/version` and the API reference, and a SvelteKit client with a sign-in screen and an empty
  projects page.
- `clap-render`: renders a directory holding a seekable `index.html` to an H.264 MP4 with
  `chrome-headless-shell` and `ffmpeg`, deterministic on Linux.
- `clap-timeline`: reads a `clap.md` into scenes and clips and applies nine edit ops (move,
  trim, split, delete and add a clip; set the duration of, reorder, delete and add a scene) as
  minimal text patches.
- `clap-render` shows `<video>` clips from frames ffmpeg extracts beforehand, and mixes
  `<audio>` and video sound into an AAC track.
- The render worker (`apps/worker`, service `clap-render`): renders queued jobs from
  `/data/renders` one at a time with no network, a read-only root and Chrome's sandbox on,
  under the seccomp profile `deploy/chrome-seccomp.json`. `clap-api` and `clap-render` share
  the `renders` directory of the `clap_data` volume; the contract is in `docs/rendering.md`.
- Motion starters (`starters/`): five ready-made raw scenes (title, lower third, caption, chart
  race, logo reveal), each rendered by a test.
- The `clap` command line (`apps/cli`): `new`, `starters`, `serve` with a player, `check`,
  `guide`, `timeline`, `edit`, `render` here or on the server, `login`, `logout`, `whoami`,
  `list`, `pull`, `push`, `open`. It mirrors Diapo's `diapo`. Reference: `docs/cli.md`.
- `clap_timeline::CLAP_GUIDE` and `format_guide()`: Clap's section of the format guide, shared
  by the command line and the API.
