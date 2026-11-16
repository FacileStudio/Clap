# Clap: API

The HTTP API under `/api`, the MCP server at `/mcp`, the sandboxed frame, uploads, and the file
protocol the API shares with the render worker. The machine-readable list of routes is at
`/docs` (`/docs/openapi.json`); a test fails when a mounted route is missing from it.

Clap's projects have the shape of Diapo's decks, on purpose: the same routes, the same `rev`
rule, the same error bodies, the same frame. Clap adds media, renders and timeline edits.

## Conventions

- **Auth** is porte's: the `session` cookie of a browser sign-in, or `Authorization: Bearer
  <token>` for the CLI. A cookie write (`POST`, `PUT`, `PATCH`, `DELETE`) must carry the suite's
  CSRF header (`porte::CSRF_HEADER_NAME`), or it answers `403`.
- **Registre bearer tokens.** When Registre is configured, a bearer shaped as a JWT is checked
  by porte: issued by Registre for the audience `OIDC_MACHINE_AUDIENCE` (the app's client id
  unless set), mapped to the account by `(issuer, sub)`, and gated on `clap.read` for `GET` and
  `clap.write` for every other method. That is how another suite app calls Clap for a person
  after an RFC 8693 token exchange. An account that never signed in to Clap is created by its
  first `POST /renders`, not by these routes: until then they answer `401`.
- **The CLI.** With `OIDC_CLI_AUDIENCE=facile-cli`, `POST /auth/oidc/device/exchange` trades a
  `facile-cli` device-grant token for a session token.
- **Errors** use the suite envelope from tronc: `{"error": {"code", "message"}}`.
- **`None` is `404`.** One function, `modules::projects::access::access`, answers what a caller
  may do with a project. Every project, asset and render query goes through it. A project the
  caller cannot see answers `404`, never `403`. `403` means the caller sees it and may not do
  this to it.

| Access | Who | May |
|---|---|---|
| `viewer` | (no role maps to it yet; share links will) | read the project, its frame, its assets and its renders |
| `editor` | a `member` of the project's space | also save the source, rename, manage assets, render, cancel a render |
| `manager` | an `admin` or `owner` of the space; the owner of a personal project | also move, trash, restore, delete |

A project with no space is personal: only its owner reaches it. An instance admin gets no bypass.

## Routes

| Route | Needs | Does |
|---|---|---|
| `GET /projects` | session | Every project the caller reaches, newest first, without sources. `?space_id=` keeps one space (`404` when not a member). `?trashed=true` lists the trash |
| `POST /projects` | editor in the space | Body `{title?, space_id?, source?}`. `201` with the project at `rev` 1. No `source` gives a one-scene starter. The title is the one given, else the source's `title`, else `Untitled` |
| `GET /projects/{id}` | viewer | `{id, space_id, title, source, rev, created_at, updated_at, trashed_at, access}` |
| `PUT /projects/{id}/source` | editor | Body `{source, rev}`. Saves and answers the project at `rev + 1`. See "The rev rule" |
| `GET /projects/{id}/timeline` | viewer | `{rev, timeline}`: the scenes and clips of the saved source |
| `POST /projects/{id}/edits` | editor | Body `{rev, ops}`: timeline operations. Answers `{source, rev, timeline, clips}`. See "Timeline edits" |
| `PATCH /projects/{id}` | editor; manager for `space_id` and `trashed` | Body `{title?, space_id?, trashed?}`. `space_id: null` moves the project to the caller's personal projects. Never changes `rev` |
| `DELETE /projects/{id}` | manager | Deletes a trashed project for good, with its revisions, assets and renders. `412` when the project is not in the trash |
| `GET /projects/{id}/events` | viewer | Server-sent events. See "Events" |
| `GET /projects/{id}/frame` | viewer | The project compiled to one HTML document. `?scene=n` compiles scene `n` alone, from 0. See "The sandboxed frame" |
| `GET /projects/{id}/assets` | viewer | `{assets: [...]}`, by name. See "Assets" |
| `POST /projects/{id}/assets` | editor | Multipart: a `file` part, and an optional `name` part. `201` with the asset in state `probing`. The same name replaces the asset |
| `GET /projects/{id}/assets/{name}` | viewer, or a frame link | The bytes, with `Range` (`206`). `?variant=poster` or `?variant=proxy` reads the derived file |
| `DELETE /projects/{id}/assets/{name}` | editor | `204` |
| `GET /projects/{id}/renders` | viewer | `{renders: [...]}`, newest first |
| `POST /projects/{id}/renders` | editor | Body `{width?, height?, fps?, range?}`. `202` with the queued render. See "Renders" |
| `POST /renders` | a Registre bearer with `clap.write` | A render of a posted bundle. See "Posted bundles" |
| `GET /renders/{id}` | viewer of its project | The render: state, progress, `download_url` once done |
| `GET /renders/{id}/events` | viewer | Server-sent `status` events until the render is final |
| `GET /renders/{id}/file` | viewer | The MP4, with `Range`. `412` until the render is done |
| `DELETE /renders/{id}` | editor | A waiting or running render is asked to stop: `202` with the render. A finished one is removed with its file: `204` |
| `GET /spaces` | session | The caller's spaces and roles, mirrored from Registre |
| `GET /assistant` | session | `{name, accent}` of the caller's Bulle assistant; `Bulle` on any failure |
| `GET /auth/session` | none | `{user, bulle_url}`; `user` is `null` when signed out |
| `GET /mcp/pending`, `POST /mcp/pending/{id}/confirm`, `POST /mcp/pending/{id}/dismiss` | the person's own browser session | The confirm page's API for agent actions that wait for the person |
| `POST /registre/sync`, `POST /registre/deletions` | Registre | The directory nudge and the signed account deletion. Mounted only when Registre is configured |

porte adds the sign-in routes (`/auth/oidc`, `/auth/oidc/callback`, `/auth/oidc/exchange`,
`/auth/oidc/device/exchange`, `/auth/sync-profile`, `/auth/backchannel-logout`, `/auth/logout`,
`/auth/me`, `/auth/config`).

Not built yet: share links, `/oembed`, `/embed/{token}`.

## The rev rule

A project is a row holding one source text and a revision number, `rev`. Every write names the
`rev` it read:

- The project is still at that `rev`: the source is saved, `rev` goes up by one, the new source
  is kept in `project_revisions` with its author and origin (`web` for a browser session, `cli`
  for a token, `mcp` for an agent), and open editors are told.
- The project moved on: `409`, nothing changes. The body is tronc's `already_exists` envelope
  plus the `rev` the project is at now:

  ```json
  {"error": {"code": "already_exists", "message": "the project is at rev 5, not 3: read it again, then retry"}, "rev": 5}
  ```

The web editor, `clap push`, the timeline edits and the MCP tools all go through
`modules::projects::write::save`. There is no merge: the client reloads and the person decides.

## Invalid sources

Every source is parsed with `scene::parse` before it is stored. A source that does not parse is
refused with `422` and changes nothing. The body is tronc's `invalid_argument` envelope plus
`errors`, one entry per problem in line order: the 1-based `line`, a stable `kind`, and the
`message` of `docs/format.md` in the Diapo repository.

```json
{
  "error": {"code": "invalid_argument", "message": "line 4: unknown key `colour`"},
  "errors": [{"line": 4, "kind": "unknown_key", "message": "unknown key `colour`"}]
}
```

Kinds: `empty`, `unclosed_frontmatter`, `bad_frontmatter_line`, `unknown_key`, `duplicate_key`,
`project_key_in_scene`, `bad_value`, `bad_step_marker`, `step_marker_outside_block`,
`step_out_of_range`, `remote_url`, `bad_asset_path`. Diapo answers the same body without
`kind`. A source over `SOURCE_MAX_BYTES` is `413`.

## Timeline edits

The timeline is the model the crate `clap-timeline` reads from the source: scenes, and in each
the clips, with their id, kind, start, duration, track and lines (`docs/timeline.md`).

`GET /projects/{id}/timeline` answers `{rev, timeline}` for the source as it is saved.

`POST /projects/{id}/edits` takes `{rev, ops}`. Each op is an object tagged by `op`:
`move_clip {clip, start, track?}`, `trim_clip {clip, start?, duration?, media_in?}`,
`split_clip {clip, at}`, `delete_clip {clip}`,
`add_clip {scene, kind, src?, start, duration, track, text?}`,
`set_scene_duration {scene, duration}`, `reorder_scene {scene, to}`, `delete_scene {scene}`,
`add_scene {at, duration, body}`. The ops apply in order to the source read at `rev`; the result
is saved under the rev rule, as one revision, and the route answers
`{source, rev, timeline, clips}`: the new source, its `rev`, its timeline, and the ids, as they
are now, of the clips the ops named or added.

- A stale `rev` is `409`, as for a save. An `ops` entry that is not an operation is `400`.
- An op that does not apply is `422` and nothing is saved. The body is tronc's
  `invalid_argument` envelope plus `timeline_error`, the engine's code: `unknown_clip`,
  `unknown_scene`, `overlap`, `out_of_bounds`, `invalid_op`, `invalid_result` or `parse`.

  ```json
  {"error": {"code": "invalid_argument", "message": "clip `title` would overlap clip `sub` on track 1"}, "timeline_error": "overlap"}
  ```

- A clip without `data-clip` or `id` has a derived id, `clip-<scene>-<n>`, which shifts when a
  clip before it is added or deleted. Read the timeline again after every edit.

One function holds the calls: `modules::projects::timeline::edits`.

## Events

`GET /projects/{id}/events` is a `text/event-stream` with two events:

```
event: changed
data: {"rev":2}

event: asset
data: {"name":"assets/clip.mp4","state":"ready"}
```

- The first event arrives at once: a `changed` that says where the project is now.
- A `changed` follows every saved source, whichever door wrote it, and every `PATCH` (same `rev`).
- An `asset` follows every upload (`probing`, or `ready` for a font), every finished probe
  (`ready` or `failed`), a finished proxy (`ready` again) and every delete (`deleted`).
- Before each event the server checks the reader still reads the project. If not, the stream ends.
- After a lag the stream answers one `changed`: list the assets again.

`GET /renders/{id}/events` sends `status` events, each carrying the render as `GET /renders/{id}`
answers it. The first says where the render is now; the stream ends after `done`, `failed` or
`cancelled`.

The broadcast lives in the process, which is one reason the app runs as one replica.

## Assets

An asset is a file of one project, named by the relative path the source writes: the source says
`<video src="assets/clip.mp4">`, the asset is named `assets/clip.mp4`.

```json
{"name": "assets/clip.mp4", "mime": "video/mp4", "size": 1048576, "sha256": "…", "kind": "video",
 "duration_ms": 2000, "width": 1920, "height": 1080, "fps": 25.0, "has_audio": true,
 "state": "ready", "error": null, "created_at": "…", "url": "/api/projects/…/assets/assets/clip.mp4",
 "poster_url": "…?variant=poster", "proxy_url": "…?variant=proxy"}
```

- **Upload.** The `file` part streams to `DATA_DIR/uploads/` chunk by chunk, hashed and counted
  as it passes. Nothing but the first 2 KiB is held in memory, and the route has no framework
  body limit: the stream itself stops at `ASSET_MAX_BYTES`. An upload that is refused or
  dropped leaves no file. The route is not resumable: a dropped upload starts again.
- **Name.** A relative path of at most 200 bytes. Each segment holds ASCII letters, digits, `.`,
  `_` and `-`, and never starts with a dot. So no `..`, no absolute path, no backslash and no
  scheme. A `name` part that breaks the rule is `400`. Without one the asset is named
  `assets/<file name>`, with every other character turned into `-`.
- **Type.** Sniffed from the bytes, never taken from the upload: MP4, QuickTime, WebM, Matroska,
  MP3, AAC, FLAC, WAV, Ogg, PNG, JPEG, GIF, WebP, AVIF, SVG, WOFF, WOFF2, TTF, OTF. Anything
  else, HTML and scripts included, is `400`. `kind` is `video`, `audio`, `image`, or `other`
  for a font.
- **Size.** `ASSET_MAX_BYTES` per file (2 GiB) and `SPACE_ASSETS_MAX_BYTES` (20 GiB) for all
  the projects of one space, or all the personal projects of one person; past either, `413`.
- **Storage.** `DATA_DIR/assets/<first two hex>/<sha256>`, once per content, with the poster
  and the proxy at `DATA_DIR/derived/<first two hex>/<sha256>.poster.jpg` and `.proxy.mp4`. A
  daily sweep removes files no row names, after an hour of grace.
- **Probe.** One background task per asset, one at a time across the process, runs `ffprobe`
  (JSON output, 30 s limit), then for a video `ffmpeg` for a JPEG poster (60 s) and for a 720p
  H.264 and AAC proxy (one hour). Both binaries get an argument vector, never a shell string,
  read no standard input and open only local files (`-protocol_whitelist file`). The asset is
  `ready` once probed, and the proxy arrives later. A file that only looks like media is
  `failed`, with the reason in `error`. A video with no picture stream becomes `audio`. At boot,
  assets left `probing` start again.
- **Serving.** `ServeFile` answers `Range` with `206`. Every asset response carries
  `Content-Security-Policy: sandbox; default-src 'none'; style-src 'unsafe-inline'` and
  `X-Content-Type-Options: nosniff`: an SVG opened on its own runs no script.

The binaries are `CLAP_FFMPEG` and `CLAP_FFPROBE`, or `ffmpeg` and `ffprobe` on `PATH`.

## The sandboxed frame

A compiled project runs the author's HTML and JavaScript. It must never run in the app's
origin, where it could read the session and call the API as the person.

`GET /projects/{id}/frame` is the only route that answers user HTML with an HTML content type.
The client shows it in `<iframe sandbox="allow-scripts" src="/api/projects/{id}/frame">` and
drives it with `postMessage` (`scene:seek`, `scene:step`). The response carries:

```
Content-Security-Policy: sandbox allow-scripts; default-src 'none'; img-src 'self' data: blob:;
  media-src 'self' blob:; style-src 'unsafe-inline'; script-src 'unsafe-inline';
  font-src 'self' data:; base-uri 'none'; form-action 'none'; frame-ancestors 'self'
X-Frame-Options: SAMEORIGIN
X-Content-Type-Options: nosniff
Cache-Control: no-store
Referrer-Policy: no-referrer
```

- `sandbox` without `allow-same-origin` gives the document an opaque origin, even when someone
  opens the URL in a tab. It has no cookies, no storage and no same-origin access to the app.
- There is no `connect-src`: `default-src 'none'` stops `fetch`, XHR, WebSocket and beacons.
- The response never carries `Set-Cookie`: a layer removes it whatever an inner layer set.

### Signed asset links

A document with an opaque origin sends no cookie, so it cannot load the project's media through
the session. The frame route rewrites every project file the source names to a signed link:

```
/api/projects/{id}/assets/{name}?frame=<expires>.<hex hmac>[&variant=proxy]
```

- The token is HMAC-SHA256 over `{project id}:{expires}`, `expires` in seconds since the epoch.
  The key is derived from `PORTE_TOKEN_KEY` under the label `clap/frame-assets/v1`, so links
  survive a restart; without that key the process uses a random one.
- A token reads every asset of one project, for one hour. It opens nothing else: no other
  project, no source, no write.
- A link with a token needs no session and is answered with `Access-Control-Allow-Origin: *` and
  `Cross-Origin-Resource-Policy: cross-origin`, which a font loaded from an opaque origin needs.
  A bad or expired token is `404`. No token and no session is `401`.
- A video that has a proxy is linked with `&variant=proxy`: the preview plays the 720p file.
  The render uses the original.
- A file the project does not hold keeps its path as written, and loads nothing.
- The document also carries Clap's media runtime (`clap_timeline::with_runtime`): after each
  `seek` it puts every shown `<video>` and `<audio>` at `data-in` plus the time into the clip
  and applies `data-volume`. It is one more inline script, which the policy already allows.

This is Diapo's design, with one more query parameter.

## Renders

The API never launches Chrome. It writes a job directory on the volume it shares with the
`clap-render` worker, and reads back what the worker writes there.

```
DATA_DIR/renders/<id>/bundle/index.html   the compiled scene                      API
DATA_DIR/renders/<id>/bundle/media/…      the original media the scene names      API
DATA_DIR/renders/<id>/job.json            written last, by rename: the job is whole  API
DATA_DIR/renders/<id>/status.json         progress, written by rename             worker
DATA_DIR/renders/<id>/out.mp4             the result                              worker
DATA_DIR/renders/<id>/cancel              asks the worker to stop                 API
DATA_DIR/renders/.worker                  the heartbeat, an RFC 3339 timestamp    worker
DATA_DIR/outputs/<id>.mp4                 where a finished render is kept         API
```

`job.json`, exactly:

```json
{"id": "<uuid>", "width": 1920, "height": 1080, "fps": 30, "range": [0.5, 1.5], "created_at": "<rfc3339>"}
```

`range` is `[start, end]` in seconds, or `null` for the whole project. `status.json`:

```json
{"state": "running|done|failed|cancelled", "frame": 12, "frames": 60, "error": null, "updated_at": "<rfc3339>"}
```

**Queueing.** `POST /projects/{id}/renders` checks the request against the caps, then compiles
the project as it is saved now.

- `width` and `height` default to the project's `size`. Both are even and at least 16; the long
  side is at most `RENDER_MAX_WIDTH` (1920) and the short side at most `RENDER_MAX_HEIGHT`
  (1080), so a portrait 1080x1920 passes.
- `fps` defaults to 30 and is at most `RENDER_MAX_FPS` (30).
- `range` lies inside the project. The rendered length is at most `RENDER_MAX_SECONDS` (600).
- Past a cap: `400`. A source that names a file the project does not hold: `412`, listing the
  names. More than `RENDER_QUEUE_MAX` (10) renders waiting or running on the instance: `429`.
- The worker's heartbeat must be less than 30 s old, or the route answers `503 unavailable`
  and queues nothing. A render already waiting when the worker goes silent carries a `notice`
  in `GET /renders/{id}`.
- `index.html` is the compiled project with Clap's media runtime, as in the preview.
- In `index.html` every project file is a relative path under `media/`, so no asset name can
  shadow `index.html`. Each media file is a hard link to the original, or a copy when the volume
  refuses links.

**Mirroring.** One task reads the `status.json` of every unfinished render each second, copies
`state`, `frame`, `frames` and `error` into the `renders` table, and tells the open streams.

- On `done` it moves `out.mp4` to `DATA_DIR/outputs/<id>.mp4` and deletes the job directory.
- On `failed` and `cancelled` the job directory stays 24 hours, for whoever looks into it.
- The worker never deletes a job directory. The API does.
- The files in a job directory are written next to a browser that ran user HTML, so they are
  not trusted: `status.json` and `out.mp4` must be regular files (a symbolic link fails the
  render), a status over 64 KiB or with an unknown state is ignored, the error text is cut at
  2000 characters.

**Sweep.** At boot and every ten minutes: an unfinished render whose `job.json` is gone, or
that waited more than six hours, is `failed` and loses its directory at once, so a worker that comes back cannot pick it up; a
job directory or an output file no row names is removed, and so is the directory of a render
that failed or was cancelled more than 24 hours ago.

### Posted bundles

`POST /api/renders` renders a scene another suite app compiled, such as a Diapo deck exported
to MP4. It answers `202` with the same render a project render gets, and `GET /renders/{id}`,
`/events`, `/file` and `DELETE` work on it with the same bearer.

- **Auth.** `Authorization: Bearer <token>`: a Registre access token for the audience `clap`
  (`OIDC_MACHINE_AUDIENCE`), exchanged for the signed-in person (it carries `act`), with the
  scope `clap.write`. porte verifies it offline and creates the person's Clap account on a
  first call, as for MCP. No token, a bad one, another audience, or the app audience without
  `act`: `401`. No `clap.write`: `403`. A session cookie or a session token is not accepted
  here.
- **Body.** `multipart/form-data` with:
  - one `spec` part, the JSON `{"width": 1920, "height": 1080, "fps": 30, "range": [0, 12.5]}`.
    `width` and `height` are required; `fps` defaults to 30; `range` is optional;
  - one `file` part per file of the bundle, whose `filename` is its relative path:
    `index.html`, `assets/chart.png`. `index.html` is required and must expose
    `window.__scene` as `scene::compile` writes it. It is rendered as posted: Clap adds nothing.
- **Paths.** The asset name rule: relative, at most 200 bytes, segments of ASCII letters,
  digits, `.`, `_` and `-` that do not start with a dot. Anything else is `400`.
- **Caps.** The same size and frame-rate caps as a project render, the same queue cap (`429`)
  and the same worker check (`503`). A `range` is at most `RENDER_MAX_SECONDS` long; without
  one the worker fails a scene longer than that once it has loaded. The bundle holds at most
  `RENDER_BUNDLE_MAX_BYTES` (1 GiB, `413`) and 2000 files. It streams to disk.
- **Ownership.** The render belongs to the account the token acts for: `project_id` is `null`,
  `rev` is 0, and only that account reads, downloads or cancels it. Anyone else gets `404`.

```sh
curl -X POST https://clap.facile.studio/api/renders -H "Authorization: Bearer $TOKEN" \
  -F 'spec={"width":1920,"height":1080,"fps":30}' \
  -F 'file=@index.html;filename=index.html' -F 'file=@chart.png;filename=assets/chart.png'
```

A render is `{id, project_id, rev, width, height, fps, range, state, frame, frames, error, size,
created_at, started_at, finished_at, download_url, notice}`. `state` is `queued`, `running`, `done`,
`failed` or `cancelled`.

## MCP

`/mcp` is a `tronc::mcp::App`, on when `MCP_RESOURCE` is set, with RFC 9728 metadata and
porte's verifier for Registre tokens. Scopes: `clap.read` for the read tools, `clap.write` for
the others. The tools call the same service functions as REST, so they follow the same access
rule and the same `rev` rule, store revisions with origin `mcp`, and emit the same events.
Every answer about a project carries its `url`, `https://<host>/projects/<id>`.

| Tool | Scope | Input | Answers |
|---|---|---|---|
| `clap_list_projects` | read | `{space_id?, trashed?, cursor?, limit?}` | `{projects: [{id, title, space_id, rev, updated_at, url}], next_cursor?, total}` |
| `clap_get_project` | read | `{project_id}` | `{id, title, space_id, rev, source, truncated, scenes, duration, assets, trashed, url}` |
| `clap_list_assets` | read | `{project_id}` | `{assets: [{name, kind, mime, size, duration_ms, width, height, fps, has_audio, state}], url}` |
| `clap_format_guide` | read | `{}` | `{guide}`: `scene::FORMAT_GUIDE`, then Clap's section (`apps/api/src/mcp/guide.md`) |
| `clap_render_status` | read | `{render_id}` | `{id, project_id, rev, state, frame, frames, error, download_url, url}` |
| `clap_create_project` | write | `{title, source?, space_id?}` | `{id, title, rev, scenes, url}` |
| `clap_write_project` | write | `{project_id, source, rev}` | `{id, title, rev, scenes, url}` |
| `clap_edit_project` | write | `{project_id, old_string, new_string, rev}` | `{id, title, rev, scenes, url}`; refuses zero or several matches |
| `clap_timeline_edit` | write | `{project_id, rev, ops}` | `{id, title, rev, scenes, duration, clips, url}`; `clips` are the ids of the clips it changed or added |
| `clap_render` | write | `{project_id, width?, height?, fps?, range?}` | the same as `clap_render_status` |
| `clap_delete_project` | write | `{project_id}` | a confirmation: the person agrees on `/pending`, then the project goes to the trash |

A stale `rev`, an invalid source and a missing project answer a tool error that says what to
call next. `download_url` is a link the person opens in their browser, signed in.

## Environment

| Variable | Default | Meaning |
|---|---|---|
| `DATA_DIR` | `/data` | Uploads, posters, proxies, render jobs, rendered files |
| `SOURCE_MAX_BYTES` | `524288` | The largest project source |
| `ASSET_MAX_BYTES` | `2147483648` | The largest upload |
| `SPACE_ASSETS_MAX_BYTES` | `21474836480` | The assets of one space together |
| `CLAP_FFMPEG`, `CLAP_FFPROBE` | `ffmpeg`, `ffprobe` on `PATH` | The media binaries |
| `RENDER_MAX_WIDTH`, `RENDER_MAX_HEIGHT` | `1920`, `1080` | The long and the short side of a render |
| `RENDER_MAX_FPS` | `30` | The highest frame rate |
| `RENDER_MAX_SECONDS` | `600` | The longest render |
| `RENDER_QUEUE_MAX` | `10` | Renders waiting or running at once |
| `RENDER_BUNDLE_MAX_BYTES` | `1073741824` | The files of one posted bundle together |
| `OIDC_MACHINE_AUDIENCE` | `OIDC_CLIENT_ID` | The audience a Registre bearer token must carry |
| `OIDC_CLI_AUDIENCE` | unset | `facile-cli`: mounts the CLI's device exchange |
| `MCP_RESOURCE` | unset | The public URL of `/mcp`; turns MCP on |
| `MCP_STATE_KEY` | unset | The HMAC key of MCP confirmations; required with `MCP_RESOURCE` |
| `BULLE_URL`, `BULLE_CLIENT_ID` | `https://bulle.facile.studio`, `bulle` | The assistant the client frames |

Back to the [documentation index](README.md).
