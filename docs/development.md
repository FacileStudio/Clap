# Clap: development

How to run Clap locally, test it, and pass the gate before a commit.

## Prerequisites

- Rust 1.98.1 (`rust-toolchain.toml`) with `rustfmt` and `clippy`
- Docker, for Postgres 16, and `pg_isready`
- bun 1.4 for the client, `filet` 0.21 or later, `mise` and `lefthook`
- `chrome-headless-shell`, `ffmpeg` and `ffprobe` for the render tests
- Network access to GitHub: `tronc-rs` and `porte-rs` come from their public repos by tag

## Setup

```sh
mise trust && mise install
mise run db-up
```

`db-up` starts `postgres:16-alpine` as `clap_dev_postgres` on `127.0.0.1:5672` (password
`postgres`) with `fsync` off, which is fine for a disposable local database. `mise run db-down`
stops it.

## Run

```sh
mise run dev
```

The API listens on `:4000` with `DATA_DIR=.local/data` and no Registre: the sign-in routes
answer `oidc_enabled: false`. Uploads need `ffmpeg` and `ffprobe` on `PATH` (or `CLAP_FFMPEG`
and `CLAP_FFPROBE`); without them an asset ends `failed`. Renders stay `queued` until a render
worker reads `DATA_DIR/renders`. The routes are in [api.md](api.md). From `apps/client`, `bun run dev` serves the client and proxies
`/api` to the API.

## Tests

```sh
export TEST_DATABASE_URL='postgres://postgres:postgres@127.0.0.1:5672/postgres?sslmode=disable'
cargo test --workspace --locked
```

| Where | Covers |
|---|---|
| `apps/api/src/**` unit tests | The environment rules and caps, the CSP, name and content checks, the `ffprobe` and `ffmpeg` arguments, the render spec, the frame tokens |
| `apps/api/tests/shell.rs` | Probes, `/version`, the session routes, the API reference drift check, the migrations equal to porte's and tronc's schemas |
| `apps/api/tests/projects.rs`, `authorization.rs`, `directory.rs` | The `rev` rule (`409`), invalid sources (`422`), every route against every role, cross-space `404`, Registre snapshots and erasure |
| `apps/api/tests/assets.rs`, `media.rs`, `frame.rs`, `events.rs` | Streamed uploads, names, caps, a real clip probed with a poster and a proxy (needs `ffmpeg`), `Range`, the frame's headers and signed links, the SSE streams |
| `apps/api/tests/render_jobs.rs`, `renders.rs` | The job directory and `job.json`, status mirroring from a hand-written `status.json`, cancel, the sweep |
| `apps/api/tests/share.rs`, `public.rs`, `public_media.rs` | Share links, the public routes, 404 for anything not live, the poster (needs `ffmpeg`), the embed framing headers |
| `apps/api/tests/render_e2e.rs` | One real render: the API and the `worker` binary over one renders directory. Needs the binary (`check.sh` builds it), Chrome and ffmpeg; skips with a banner without them. Chrome's sandbox is off in that test's worker only |
| `apps/api/tests/machine.rs` | `POST /api/renders` with tokens signed by a test key (`tests/testdata/idp_test_key.pem`, used nowhere else) that a fake issuer publishes |
| `apps/api/tests/mcp.rs`, `mcp_tools.rs` | tronc's MCP conformance kit, the eleven tools, scope gates, the delete confirmation |
| `crates/render/src/**` unit tests | The URL guard, the frame grid, the Chrome flags, the ffmpeg arguments |
| `crates/render/tests/render.rs` | Real renders of `fixtures/title`: frame count, size, identical runs, the network block, cancel |

Without `TEST_DATABASE_URL` the database tests print a skip banner and pass. Without Chrome,
ffmpeg or ffprobe the render tests do the same, and so does `apps/api/tests/media.rs` without
ffmpeg. Read the output.

The render tests find their binaries through `CLAP_CHROME`, `CLAP_FFMPEG` and `CLAP_FFPROBE`,
then `PATH`, then, for Chrome, the newest `chromium_headless_shell-*` build in Playwright's
cache (`~/.cache/ms-playwright`). To install that build:
`bunx playwright-core install chromium-headless-shell`.

The client's unit tests run with `cd apps/client && bun run test` (vitest, jsdom).

## The gate

```sh
sh scripts/check.sh
filet check .
```

`check.sh` runs `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --locked --
-D warnings` and `cargo test --workspace --locked`, then `bun run check`, `bun run test` and
`bun run build` in `apps/client`. The end-to-end stage prints a skip line until the editor
exists. `lefthook` runs the gate on pre-push. Never raise a filet limit to pass.

Back to the [documentation index](README.md).
