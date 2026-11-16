# Changelog

All notable changes to Clap. Versions follow SemVer.

## [Unreleased]

## [0.1.1] - 2026-11-16

### Added

- Files dragged from the desktop onto the timeline upload and land at the drop time and track.
  While a file or a media bin item is aimed, a ghost clip shows where it will land and for how
  long; while it uploads, the ghost stays with the file's first frame and the upload progress.
  The browser reads the length and the first frame itself, so a clip is placed before the
  server's probe ends, and the media bin can place a file while it is still being analysed.

### Changed

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
