# Clap plan

Written 2026-11-16. Companion plans in `FacileStudio/Diapo`: `PLAN.md` (the scene format this
app shares) and `PLAN-suite-wiring.md` (Registre, Bulle, DNS, Dokploy, release). Research:
wiki page `syntheses/video-and-slides-as-code`.

## Goal

Ship Clap at `clap.facile.studio`: a video editor whose project is one text file, edited on
the web with a preview and a timeline, by Bulle and agents through MCP, and by the `clap`
CLI, and rendered to MP4 by a Rust worker.

## Why

The owner wants video editing and motion design that AI can do well. Models write HTML, CSS
and JS reliably, and HeyGen reports that plain HTML beat a React framework for agent-made
video (https://www.heygen.com/research/html-to-video). A project that is a text file also
gives the web app and the CLI one thing to sync.

## Checked against

Same pages and Docs files as `Diapo/PLAN.md`, read 2026-11-16.

## Owner decisions (2026-11-16)

- Rust backend on `tronc-rs` and `porte-rs`, SvelteKit on muse, Rust CLI, mise, filet.
- Own Rust render worker. No Node and no HyperFrames dependency in production; the element
  attributes stay compatible with HyperFrames (`data-start`, `data-duration`,
  `data-track-index`).
- Editor v1: source + preview + Bulle, and a drag-and-drop timeline. No transcript editing.
- Free app. AI through Bulle with `ai.access`. MCP free.
- Public repo, AGPL-3.0-only.
- Logo (owner, 2026-11-16): the Solar glyph `solar:clapperboard-open-bold-duotone`, used for the
  app mark, favicon, landing, Suite wall and Bulle's suite icons.

## Approach

Same chassis as Diapo (the Docs shape) and the same `facile-scene` crate, pinned by tag from
`FacileStudio/Diapo`. Clap adds three things: media assets, a timeline model, and rendering.

**Timeline model.** `facile-scene` already parses scenes with `duration` and elements with
`data-start`, `data-duration`, `data-track-index`. Clap adds `crates/timeline`: it reads a
`clap.md` into `{scenes[{duration, clips[{id, kind, src, start, duration, track, in, out}]}]}`
and writes edits back as minimal text patches (move, trim, split, delete, add clip, reorder
scene). The drag timeline in the browser and the MCP edit tools both call it, so a drag and
an agent edit produce the same kind of diff and the file stays hand-readable.

**Preview.** The compiled composition runs in a sandboxed frame and is driven through
`window.__scene.seek(t)`. Source video plays in the frame with a `<video>` element whose
`currentTime` is set on seek; good enough for editing, not frame-exact. Frame-exact output is
the renderer's job.

**Render.** `crates/render` drives `chrome-headless-shell` over CDP with
`--deterministic-mode --enable-begin-frame-control` and
`HeadlessExperimental.beginFrame`: seek, capture, next frame. It never plays. Before capture,
FFmpeg extracts each source video to frames at the target fps and the worker swaps each
`<video>` for the right frame, the technique HeyGen describes at
https://www.heygen.com/research/html-to-video. FFmpeg encodes H.264 + AAC and mixes audio
from `<audio>` and video clips. Deterministic capture works only on Linux, which is where
the server runs; the CLI on macOS falls back to plain screenshots with a readiness check.

**Limits.** One render at a time (ruche has 12 cores and 122 containers), 1080p and 30 fps
maximum, 10 minutes maximum, uploads capped at 2 GiB per file and 20 GiB per space, all env
tunable. Renders are free in v1; the cap is there for the machine.

## Render isolation (decided 2026-11-16, after wave A)

Chrome's own sandbox does not start under Docker's default seccomp profile, and render Chrome
loads HTML that users wrote. The sandbox stays on. Rendering runs in a second service,
`clap-render`, built from the same image:

- `network_mode: none`. It reaches nothing: no database, no Docker network, no internet.
- `security_opt: seccomp=deploy/chrome-seccomp.json`, a profile that allows the user
  namespaces Chrome's sandbox needs. `--no-sandbox` is never set in production.
- It shares only the `/data/renders` volume with `clap-api`. The API writes
  `/data/renders/<job id>/job.json` and `bundle/` (the compiled scene and its media); the
  worker (`apps/worker`, binary `worker`) polls for queued jobs, renders one at a time, and
  writes `status.json` (state, frame, frames, error) and `out.mp4` atomically. The API reads
  `status.json`; a `cancel` file asks the worker to stop.
- Diapo's PDF and thumbnail capture will follow the same pattern.

## Repo layout (all new)

```
Cargo.toml                      workspace
crates/timeline/                clip model over facile-scene, text-patch edits
crates/render/                  CDP capture, ffmpeg extract/encode/mix, job runner
apps/api/                       modules/{projects,assets,renders,share,assistant,events}, mcp/
apps/api/migrations/            0001_porte.sql, 0002_clap.sql, 0003_mcp_actions.sql
apps/cli/                       binary `clap`
apps/client/                    SvelteKit 2, Svelte 5, Tailwind 4, @facile/muse
scripts/check.sh, Dockerfile, docker-compose.yml, mise.toml, filet.yml, lefthook.yml
docs/                           api.md, rendering.md, deployment.md, development.md
```

## API surface

Under `/api`, porte auth, CSRF on cookie writes. Projects mirror Diapo's decks exactly
(`GET/POST /projects`, `GET /projects/{id}`, `PUT /projects/{id}/source` with `rev` and
`409`, `PATCH`, `/events` SSE, `/frame`, `/assets`, share). Clap adds:

| Route | Does |
|---|---|
| `POST /projects/{id}/assets` | resumable multipart upload; probes with `ffprobe`; makes a poster and a 720p proxy |
| `GET /projects/{id}/assets/{name}` | range requests |
| `POST /projects/{id}/edits` | body `{rev, ops[]}` using the timeline ops; returns the new source and `rev` |
| `POST /projects/{id}/renders` | body `{width, height, fps, range?}`; returns a job |
| `GET /renders/{id}`, `GET /renders/{id}/events` | status and progress (SSE), download URL |
| `DELETE /renders/{id}` | cancel |
| `POST /renders` (bearer, audience `clap`) | render a posted scene bundle; used by Diapo's MP4 export |

MCP at `/mcp` (scopes `clap.read`, `clap.write`): `clap_list_projects`, `clap_get_project`,
`clap_create_project`, `clap_write_project`, `clap_edit_project` (string replacement),
`clap_timeline_edit` (the ops), `clap_list_assets`, `clap_format_guide`, `clap_render`,
`clap_render_status`, `clap_delete_project` (confirmation card).

## Steps

Wave A, one agent, in parallel with Diapo's wave A:

1. Scaffold the repo from the Docs shape, identical to Diapo step 1, without the scene
   dependency yet. `[auth]` `[migrations]` `[deploy]`
   Verify: `sh scripts/check.sh` exits 0; `docker build .` succeeds.

Wave B, after `FacileStudio/Diapo` tags `v0.1.0`, parallel agents in worktrees:

2. `apps/api/src/modules/projects` + `events` + frame route: the Diapo deck module's shape
   with `kind = clap`. `[migrations]` `[auth]`
   Verify: `cargo test -p api`, with stale-`rev` and cross-space tests.
3. `apps/api/src/modules/assets`: uploads, sniffing, `ffprobe`, poster and proxy through
   `ffmpeg`, range serving, quotas. Verify: test uploads a 2 s fixture clip and reads back
   duration, poster and a 206 response.
4. `crates/timeline`: model and patch ops, property-tested (an op then its inverse returns
   the original text). Verify: `cargo test -p clap-timeline`.
5. `crates/render` + `modules/renders`: capture loop, video frame swap, audio mix, encode,
   job table, progress, cancel, the network block on Chrome. Verify: render a 3 s fixture
   with a title tween and a video clip; test asserts 90 frames, the right duration, and a
   pixel hash equal across two runs.
6. `apps/api/src/mcp`: the eleven tools. `[contract: names fixed here]`
   Verify: `cargo test -p api --test mcp`.
7. `apps/client` shell and source editor: project list, media bin with upload, split view
   (source, preview with scrubber and play), render dialog with progress, Bulle sidebar
   (`app=clap`). `[ui: muse, no borders]` `[owner]`
   Verify: `bun run check && bun run test`; e2e uploads a clip, previews, starts a render.
8. `apps/cli`: `clap new|serve|render|login|logout|pull|push|--version`, `--json`.
   `render` runs `crates/render` locally (needs Chrome and FFmpeg, says so when missing);
   `render --remote` uses the server. `[cli]` Verify: `cargo test -p clap-cli`;
   `clap render crates/render/fixtures/title` writes an MP4 `ffprobe` accepts.

Wave C:

9. Drag-and-drop timeline: tracks, clips, move, trim, split, snap, zoom, undo/redo, all
   through `POST /edits`. Keyboard: space, J/K/L, arrow frame-step, S to split.
   `[ui]` Verify: e2e drags a clip and asserts the `data-start` in the source changed.
10. Motion starters: a small library of raw scenes (title, lower third, caption, chart race,
    logo reveal) that `clap new --template` and the editor's insert menu use, and that
    `clap_format_guide` lists so Bulle reuses them. Verify: each renders in the fixture test.
11. Share links for finished renders. `[auth]` Verify: authorization test.
12. Landing page, same rules as Diapo step 13, with a mise-en-scene demo of the editor.
    `[ui]` Verify: five widths, `pricing-check`.
13. Docs, `CLAUDE.md`, `CHANGELOG.md`. Verify: `filet check .`.

Wave D: release and deploy, in `Diapo/PLAN-suite-wiring.md`.

## Prisme link cards (contract requested by the Prisme session, 2026-11-16)

Prisme v0.24 renders rich cards for Clap links. Part of the share-link step:

- `GET /oembed?url=<encoded share URL>&format=json`: oEmbed 1.0, no cookies, no auth. `200`
  only for a live share link, `404` for anything else, so a private item never leaks.
  JSON: `version "1.0"`, `type "rich"`, `provider_name "Clap"`, `title`, `thumbnail_url`,
  `thumbnail_width`, `thumbnail_height`, `html` (an iframe to the embed page), `width`,
  `height`, plus `duration_ms`.
- `GET /embed/{token}`: read-only player, framable only by
  `frame-ancestors 'self' https://prisme.facile.studio`. The path carries the share token,
  never the internal id.
- The public share page (`/v/{token}`) sets `og:title` and `og:image`.
- Thumbnail: the poster frame, served at `/v/{token}/thumbnail.jpg`.

Verify: a test fetches `/oembed` for a shared and an unshared item and gets 200 and 404; the
embed response carries the `frame-ancestors` header above.

## Boundaries

- Ask first: lifting a render or storage limit in production; a second replica; GPU encode.
- Never: run user HTML in the app origin; let render Chrome reach the network or the Docker
  network (it loads only a bundle the worker wrote to disk); shell out to `ffmpeg` with a
  user string that is not an argv element; store uploads outside `/data`.

## Done when

`sh scripts/check.sh` and `filet check .` exit 0; `https://clap.facile.studio` serves the
landing and the app; a project with one uploaded clip and one animated title renders to an
MP4 that plays; the same project round-trips through `clap pull` and `clap push`; Bulle adds
a title scene from the sidebar and starts a render; `mcp-live-smoke` passes with
`MCP_TOOL=clap_list_projects`.

## Risks

- The largest scope in the batch. The drag timeline and the renderer are each a real
  project. If one slips, the source editor with preview and server render still ships.
- Render load on ruche. One job at a time, `nice`d, with a hard timeout.
- Untrusted HTML in headless Chrome is the main security exposure. The bundle is loaded from
  `file://` inside a container with no network; `/facile-review` and a security review run
  before the first public release.
- Preview is not frame-exact. The UI says "preview" and the render is the reference.
- Disk. Extracted frames are large: a 1 minute 1080p clip is about 1800 JPEGs. The worker
  writes them to a tmp dir under `/data/tmp`, bounded and swept.
- The binary is named `clap`, like the Rust argument parser crate. No conflict at runtime;
  the CLI package is named `clap-cli` to keep Cargo clear.

## Skip

- Transcript editing (owner, 2026-11-16).
- Nuage import, Echo recording import, Scribe captions. Planned next; each is a token
  exchange plus an import route.
- Keyframe curve editor, effects, colour grading, multi-cam.
- OpenTimelineIO import and export.
