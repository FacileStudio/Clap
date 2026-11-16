#!/usr/bin/env sh
set -eu

cd "$(dirname "$0")/.."
root="$(git rev-parse --show-toplevel)"

: "${E2E_ADMIN_URL:=${TEST_DATABASE_URL:-postgres://postgres:postgres@127.0.0.1:${POSTGRES_PORT:-5672}/postgres?sslmode=disable}}"
export E2E_ADMIN_URL

if [ -z "$(ls -d "${PLAYWRIGHT_BROWSERS_PATH:-$HOME/.cache/ms-playwright}"/chromium_headless_shell-* 2>/dev/null)" ]; then
  echo "e2e: no Playwright Chromium, SKIPPING the browser run (install: bunx playwright-core install chromium-headless-shell)" >&2
  exit 0
fi
if ! command -v ffmpeg >/dev/null 2>&1; then
  echo "e2e: no ffmpeg: the upload, drop, drag and split tests will be SKIPPED" >&2
fi

if [ "${E2E_STACK:-}" = mock ]; then
  echo "e2e: E2E_STACK=mock, running against the in-memory API" >&2
elif [ ! -d "$root/apps/api/src/modules/projects" ] || ! pg_isready -d "$E2E_ADMIN_URL" -q 2>/dev/null; then
  echo "e2e: no project API in this tree or no Postgres, running against the in-memory MOCK API (run: mise run db-up)" >&2
  export E2E_STACK=mock
else
  (cd "$root" && cargo build --locked -p api --bin api --example e2e_seed)
fi

[ -n "${E2E_SKIP_BUILD:-}" ] || bun run build
bun test --timeout 180000 ./e2e/clap.e2e.ts
