# Changelog

All notable changes to Clap. Versions follow SemVer.

## [Unreleased]

### Added

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
