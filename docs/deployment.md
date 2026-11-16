# Clap: deployment

How Clap ships: one image, one container, one Traefik router, one hostname, one replica.
Nothing is deployed yet; this page describes what the repository builds.

## Topology

```
clap.facile.studio ──▶ Traefik (web → websecure) ──▶ clap-api:4000 ──▶ clap-db:5432
                                                              │
                                                     volume clap_data at /data
```

`/api/*` is the API, `/docs` the API reference, everything else the client. There is no
`PathPrefix` and no `stripprefix`.

## Build

```sh
docker build -t clap .
```

The root `Dockerfile` builds the client with bun and the API with `cargo build --release
--locked -p api`. The version comes from `apps/client/package.json` through
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
`clap-data-owner` service (busybox) that hands `/data` to 65532 before `clap-api` starts.

## Bumping Chrome

1. Pick a version from
   https://googlechromelabs.github.io/chrome-for-testing/last-known-good-versions-with-downloads.json
2. Download its `chrome-headless-shell-linux64.zip` and run `sha256sum` on it.
3. Set `CHROME_VERSION` and `CHROME_SHA256` in `Dockerfile` and in
   `.github/workflows/ci.yml`, together.
4. Run `CLAP_CHROME=<the new binary> cargo test -p clap-render`. A new Chrome can change
   pixels: renders made before and after the bump may differ.

## Chrome's sandbox in a container

`clap-render` keeps Chrome's sandbox on unless `CLAP_CHROME_SANDBOX=off`. Docker's default
seccomp profile refuses the user namespaces that sandbox needs, so in the container Chrome
starts only with the sandbox off or with a seccomp profile that allows them. The image sets
neither. This is an open decision for the release wave: rendering untrusted HTML without the
sandbox leaves the container as the only boundary.

## Migrations

Migrations run at boot, behind sqlx's lock. `0001_porte.sql` is porte's schema verbatim.

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
