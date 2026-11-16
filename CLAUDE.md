# Clap

A video editor for the Facile suite whose project is one text file, at `clap.facile.studio`.
Cargo workspace: the Rust API in `apps/api`, the renderer in `crates/render`, the SvelteKit
client in `apps/client`, one container. `PLAN.md` holds the plan and the steps left.

## Stack

- **API** (`apps/api`): Rust 1.98 (edition 2024), axum 0.8, sqlx 0.9 on Postgres 16
- **Libraries**: `tronc` (tronc-rs v0.5.1), `porte` (porte-rs v0.8.1), from their public
  `github.com/FacileStudio` repos by tag (bump: `docs/deployment.md`). Never copy their code
  here; report a gap instead
- **Render** (`crates/render`, package `clap-render`): `chrome-headless-shell` over CDP and
  `ffmpeg`, no async runtime. Read `docs/rendering.md` before changing it
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
```

The database tests read `TEST_DATABASE_URL` (check.sh defaults it to the local container), and
the render tests need `chrome-headless-shell`, `ffmpeg` and `ffprobe`. Both print a skip banner
when their dependency is missing: read the output before trusting a green run.

## Invariants

- **One replica.** `docker-compose.yml` pins `replicas: 1`; renders run one at a time.
- **Auth is porte's.** `apps/api/src/auth` wires porte as its `docs/architecture.md` says.
  Directory sync, account deletions and the MCP verifier are not wired yet: they need the
  spaces tables of `0002_clap.sql` (PLAN.md step 2) and must land before the first deploy.
- **Render Chrome reaches nothing.** It loads `file://` URLs inside the bundle directory and
  refuses every other request (`crates/render/src/guard.rs`), with the proxy and resolver
  flags of `chrome.rs` behind that. Never loosen either.
- **ffmpeg takes an argument vector.** No shell string, and no user text that is not one
  whole element (`crates/render/src/encode.rs`).
- **The page never plays.** Every frame is `window.__scene.seek(t)` then a capture.
- **Chrome is pinned** by version and SHA-256 in `Dockerfile` and `.github/workflows/ci.yml`.
  Move both together.
- **Migrations** are append-only `NNNN_name.sql`; `0001_porte.sql` is `porte::pg::SCHEMA`
  verbatim, and a unit test compares them.
- **No borders in the UI.** Separate with fill steps, spacing and shadows.
- No inline comments, no comments in function bodies; doc comments on public items.
