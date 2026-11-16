#!/usr/bin/env sh
set -eu

cd "$(dirname "$0")/.."

if [ -z "$(ls -d "${PLAYWRIGHT_BROWSERS_PATH:-$HOME/.cache/ms-playwright}"/chromium_headless_shell-* 2>/dev/null)" ]; then
  echo "e2e: no Playwright Chromium, SKIPPING the browser run (install: bunx playwright-core install chromium-headless-shell)" >&2
  exit 0
fi
if ! command -v ffmpeg >/dev/null 2>&1; then
  echo "e2e: no ffmpeg: the upload, drop, drag and split tests will be SKIPPED" >&2
fi

[ -n "${E2E_SKIP_BUILD:-}" ] || bun run build
bun test --timeout 180000 ./e2e/clap.e2e.ts
