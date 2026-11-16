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
answer `oidc_enabled: false`. From `apps/client`, `bun run dev` serves the client and proxies
`/api` to the API.

## Tests

```sh
export TEST_DATABASE_URL='postgres://postgres:postgres@127.0.0.1:5672/postgres?sslmode=disable'
cargo test --workspace --locked
```

| Where | Covers |
|---|---|
| `apps/api/src/**` unit tests | The environment rules, the CSP, and `0001_porte.sql` equal to `porte::pg::SCHEMA` |
| `apps/api/tests/shell.rs` | The real router on Postgres: probes, `/version`, the session routes, the API reference drift check |
| `crates/render/src/**` unit tests | The URL guard, the frame grid, the Chrome flags, the ffmpeg arguments |
| `crates/render/tests/render.rs` | Real renders of `fixtures/title`: frame count, size, identical runs, the network block, cancel |

Without `TEST_DATABASE_URL` the database tests print a skip banner and pass. Without Chrome,
ffmpeg or ffprobe the render tests do the same. Read the output.

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
