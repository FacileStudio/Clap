# Clap

A video editor for the Facile suite whose project is one text file, at `clap.facile.studio`.
Cargo workspace: the Rust API in `apps/api`, the render worker in `apps/worker`, the renderer
in `crates/render`, the timeline model in `crates/timeline`, the SvelteKit client in
`apps/client`. One image, two services. `PLAN.md` holds the plan and the steps left.

## Stack

- **API** (`apps/api`): Rust 1.98 (edition 2024), axum 0.8, sqlx 0.9 on Postgres 16
- **Libraries**: `tronc` (tronc-rs v0.5.1), `porte` (porte-rs v0.8.1), from their public
  `github.com/FacileStudio` repos by tag (bump: `docs/deployment.md`). Never copy their code
  here; report a gap instead
- **Render** (`crates/render`, package `clap-render`): `chrome-headless-shell` over CDP and
  `ffmpeg`, no async runtime. Read `docs/rendering.md` before changing it
- **Timeline** (`crates/timeline`, package `clap-timeline`): a `clap.md` read into scenes and
  clips over `facile-scene` (from `FacileStudio/Diapo` by tag), and edit ops written back as
  minimal text patches. Read `docs/timeline.md` before changing it
- **Worker** (`apps/worker`, binary `worker`): the isolated render service. No network, no
  database, one shared directory
- **CLI** (`apps/cli`, package `clap-cli`, binary `clap`): the sibling of Diapo's `diapo`, same
  config, sign-in, output and sync rules. Every HTTP call is in `src/api`. Read `docs/cli.md`
  and the suite CLI standard before changing it
- **Client** (`apps/client`): SvelteKit 2, Svelte 5 runes, Tailwind 4, `@facile/muse`

## Commands

```sh
mise run db-up              # Postgres 16 on 127.0.0.1:5672 (container clap_dev_postgres)
sh scripts/check.sh         # the gate: fmt, clippy -D warnings, tests, then the client
filet check .               # after the gate, before every commit
filet docker .              # after any Dockerfile change
mise run dev                # the API on :4000 against the local database
cargo run -p api -- migrate status
cargo test -p clap-render   # real renders; needs Chrome and ffmpeg
cargo test -p clap-timeline # golden and property tests of the edit ops
cargo run -p clap-cli -- <command>   # the clap command line from a checkout
```

The database tests read `TEST_DATABASE_URL` (check.sh defaults it to the local container), and
the render tests need `chrome-headless-shell`, `ffmpeg` and `ffprobe`. Both print a skip banner
when their dependency is missing: read the output before trusting a green run.

## Invariants

- **One replica.** `docker-compose.yml` pins `replicas: 1`; renders run one at a time.
- **Rendering is isolated.** Only the `clap-render` service starts Chrome. It has no network,
  and Chrome's sandbox stays on under `deploy/chrome-seccomp.json`: never set `--no-sandbox`
  or `CLAP_CHROME_SANDBOX=off` in production config. The API and the worker talk only through
  files under `/data/renders` (`docs/rendering.md`).
- **Edits are minimal text patches.** `clap-timeline` changes the attribute values and lines
  an op concerns and nothing else; every result must parse with `scene::parse`.
- **Auth is porte's.** `apps/api/src/auth` wires porte as its `docs/architecture.md` says:
  sessions, Registre sign-in, directory sync, deletions with erasure, the MCP verifier.
- **One `access()`.** `modules::projects::access::access` authorizes every project, asset and
  render query; `None` is always `404`. Read `docs/api.md` before changing a route.
- **User HTML is served by one route,** `GET /api/projects/{id}/frame`, with the `sandbox
  allow-scripts` policy and no cookie. Never give it `allow-same-origin` or `connect-src`.
- **The API never launches Chrome.** It writes `DATA_DIR/renders/<id>/` and reads the worker's
  `status.json` (`docs/api.md`, "Renders"). Every access to `renders/` goes through
  `modules::renders::jobfs` (`openat2`, no links, fd checks); never open a path there directly.
- **Render Chrome reaches nothing.** It loads `file://` URLs inside the bundle directory and
  refuses every other request (`crates/render/src/guard.rs`), with the proxy and resolver
  flags of `chrome.rs` behind that. Never loosen either.
- **ffmpeg takes an argument vector.** No shell string, and no user text that is not one
  whole element (`crates/render/src/encode.rs`).
- **The page never plays.** Every frame is `window.__scene.seek(t)` then a capture. Chrome
  never decodes video or sound: ffmpeg extracts the frames and mixes the audio.
- **Chrome is pinned** by version and SHA-256 in `Dockerfile` and `.github/workflows/ci.yml`.
  Move both together.
- **Migrations** are append-only `NNNN_name.sql`; `0001_porte.sql` is `porte::pg::SCHEMA`
  verbatim, and a unit test compares them.
- **No borders in the UI.** Separate with fill steps, spacing and shadows.
- **The CLI version is the client's.** `apps/cli/Cargo.toml` and `apps/client/package.json`
  carry the same version; the gate and the release workflow check it against the tag.
- No inline comments, no comments in function bodies; doc comments on public items.
