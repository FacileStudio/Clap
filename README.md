# Clap

[![ci](https://github.com/FacileStudio/Clap/actions/workflows/ci.yml/badge.svg)](https://github.com/FacileStudio/Clap/actions/workflows/ci.yml)

A video editor for the Facile suite whose project is one text file. People edit it on the web,
agents edit it over MCP, the `clap` CLI syncs it, and a Rust worker renders it to MP4.

Clap is being built. `PLAN.md` holds the plan; today the repository has the app shell and the
renderer.

## What exists

- `apps/api`: the Rust API. Registre sign-in through porte, health probes, `/version`, the API
  reference at `/docs`, and the built client served as a single-page app.
- `apps/client`: the SvelteKit client. A sign-in screen and an empty projects page.
- `crates/render`: `clap-render`, which turns a seekable HTML scene into an H.264 MP4 with
  `chrome-headless-shell` and `ffmpeg`. See [docs/rendering.md](docs/rendering.md).

- `crates/timeline`: `clap-timeline`, the clip model and the edit operations. See
  [docs/timeline.md](docs/timeline.md).
- `apps/worker`: the isolated render service.
- `apps/cli`: the `clap` command line. See [docs/cli.md](docs/cli.md).

## Install

```sh
curl -fsSL https://raw.githubusercontent.com/FacileStudio/Clap/main/install.sh | bash
```

Installs to `~/.local/bin` via [facile](https://github.com/FacileStudio/facile), the suite
installer. Pass `--bin-dir <dir>` to change that, `--source` to build from source, `--no-skill`
to skip AI agent skill registration.

Already have `facile`:

```sh
facile install clap
```

No release is published yet: until the first tag, run `cargo run -p clap-cli -- <command>`.

## Stack

| Layer | Tech |
|---|---|
| API | Rust 1.98, axum 0.8, sqlx 0.9, PostgreSQL 16, tronc-rs 0.5.1, porte-rs 0.8.1 |
| Client | SvelteKit 2, Svelte 5 (runes), Tailwind CSS 4, muse (`@facile/muse` 0.28.0) |
| Render | chrome-headless-shell 157.0.8103.78 over the DevTools Protocol, ffmpeg |
| Deploy | Docker Compose, one container and one replica behind Traefik |

## Local development

```sh
mise trust && mise install
mise run db-up
mise run dev
```

The API listens on <http://localhost:4000>. Run the client from `apps/client` with
`bun run dev`. More in [docs/development.md](docs/development.md).

## The gate

```sh
sh scripts/check.sh
filet check .
```

## License

AGPL-3.0-only. See [LICENSE](LICENSE).
