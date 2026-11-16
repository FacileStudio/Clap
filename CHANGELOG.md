# Changelog

All notable changes to Clap. Versions follow SemVer.

## [Unreleased]

### Added

- The app shell: a Rust API on tronc-rs and porte-rs with Registre sign-in, health probes,
  `/version` and the API reference, and a SvelteKit client with a sign-in screen and an empty
  projects page.
- `clap-render`: renders a directory holding a seekable `index.html` to an H.264 MP4 with
  `chrome-headless-shell` and `ffmpeg`, deterministic on Linux.
