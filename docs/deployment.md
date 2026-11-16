# Clap: deployment

How Clap ships: one image, an API container and a render container, one Traefik router, one
hostname, one replica of each.
Nothing is deployed yet; this page describes what the repository builds.

## Topology

```
clap.facile.studio ──▶ Traefik (web → websecure) ──▶ clap-api:4000 ──▶ clap-db:5432
                                                              │
                                                     volume clap_data at /data
                                                              │ subpath renders
                                                     clap-render (no network)
```

`/api/*` is the API, `/docs` the API reference, everything else the client. There is no
`PathPrefix` and no `stripprefix`.

## Build

```sh
docker build -t clap .
```

The root `Dockerfile` builds the client with bun, and the API and the render worker with
`cargo build --release --locked -p api -p worker`. The version comes from `apps/client/package.json` through
`scripts/version.sh`, and the build fails when it cannot be read.

Unlike Docs, the runtime image is not distroless: rendering needs `chrome-headless-shell`,
`ffmpeg` and fonts. The runtime stage is `debian:bookworm-slim`, pinned by digest, with:

- `ffmpeg`, `fonts-liberation`, `fonts-noto-color-emoji` and the shared libraries Chrome
  links, from Debian
- `chrome-headless-shell` from Chrome for Testing, at `/opt/chrome-headless-shell`
  (`CLAP_CHROME`). A separate stage downloads the exact version named by `CHROME_VERSION` and
  refuses it unless its SHA-256 equals `CHROME_SHA256`
- `tini` as PID 1. Chrome leaves helper processes behind; without an init they pile up as
  zombies until the container runs out of PIDs
- the API running as uid 65532, as in the other suite images

The `/data` volume is created by a root container, so `docker-compose.yml` runs a one-shot
`clap-data-owner` service (busybox) that creates `/data/renders` and hands `/data` to 65532
before `clap-api` and `clap-render` start.

## Bumping Chrome

1. Pick a version from
   https://googlechromelabs.github.io/chrome-for-testing/last-known-good-versions-with-downloads.json
2. Download its `chrome-headless-shell-linux64.zip` and run `sha256sum` on it.
3. Set `CHROME_VERSION` and `CHROME_SHA256` in `Dockerfile` and in
   `.github/workflows/ci.yml`, together.
4. Run `CLAP_CHROME=<the new binary> cargo test -p clap-render`. A new Chrome can change
   pixels: renders made before and after the bump may differ.

## The render service

`clap-render` is a second service built from the same image, started with the `worker` binary.
It has no network, a read-only root, Chrome's sandbox on, and one mount: the `renders`
subdirectory of the `clap_data` volume at `/data/renders`. `clap-api` mounts the whole volume at
`/data`, so the API reads and writes the same jobs at `/data/renders`. The contract between the
two, the seccomp profile and the limits are in [Rendering](rendering.md#the-render-worker).

## Migrations

Migrations run at boot, behind sqlx's lock. `0001_porte.sql` is porte's schema verbatim.

## Data

Everything the API stores outside Postgres is under `/data`, the `clap_data` volume:
`assets/` (uploads, by content), `derived/` (posters and proxies), `uploads/` (uploads in
flight, emptied at boot), `renders/` (one directory per render job) and `outputs/` (finished
MP4s). The render worker needs `/data/renders` and nothing else: mount that path of the volume
into it. `clap-data-owner` creates it. The job files are described in [api.md](api.md).

`MCP_RESOURCE` (the public URL of `/mcp`) with `MCP_STATE_KEY` turns the MCP server on.

## Health

`/health` and `/ready` (also under `/api`). The compose healthcheck re-execs the binary:
`["CMD", "/api", "healthcheck"]`.

## Bumping a library

`tronc-rs` and `porte-rs` come from their public GitHub repos, pinned by tag. To move one,
change its `tag` in `apps/api/Cargo.toml` (tronc appears twice, and porte pins its own tronc:
keep them on the same tag so `cargo tree -i tronc` shows one copy), run
`cargo update -p <crate>` and commit `Cargo.lock`. `@facile/muse` moves in
`apps/client/package.json`, followed by `bun install`. Run `sh scripts/check.sh` before the
commit.

Back to the [documentation index](README.md).
