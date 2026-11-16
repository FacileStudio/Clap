#!/usr/bin/env sh
set -eu

cd "$(dirname "$0")/.."

shells="${PLAYWRIGHT_BROWSERS_PATH:-$HOME/.cache/ms-playwright}"
if [ -z "${CLAP_CHROME:-}" ] && [ -z "$(ls -d "$shells"/chromium_headless_shell-* 2>/dev/null)" ]; then
  echo "e2e: no Chromium (CLAP_CHROME or a Playwright download), SKIPPING the landing run" >&2
  exit 0
fi

[ -d build ] || bun run build
bun test --timeout 180000 ./e2e/landing.e2e.ts
