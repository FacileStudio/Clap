#!/usr/bin/env sh
set -eu

cd "$(git rev-parse --show-toplevel)"

: "${TEST_DATABASE_URL:=postgres://postgres:postgres@127.0.0.1:${POSTGRES_PORT:-5672}/postgres?sslmode=disable}"
export TEST_DATABASE_URL
if ! pg_isready -d "$TEST_DATABASE_URL" -q 2>/dev/null; then
  echo "check: no Postgres at TEST_DATABASE_URL; the database tests will skip (run: mise run db-up)" >&2
fi

cli_version="v$(sed -n 's/^version = "\(.*\)"/\1/p' apps/cli/Cargo.toml | head -n 1)"
if [ "$cli_version" != "$(sh scripts/version.sh)" ]; then
  echo "check: apps/cli is $cli_version but apps/client/package.json is $(sh scripts/version.sh); a release tag must match both" >&2
  exit 1
fi

echo "==> workspace (apps/api, apps/cli, apps/worker, crates/render, crates/timeline)"
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build -p worker --locked
cargo test --workspace --locked

echo "==> apps/client"
(
  cd apps/client
  [ -d node_modules ] || bun install --frozen-lockfile >/dev/null
  bun run check
  bun run test
  bun run build
)

echo "==> e2e"
(cd apps/client && sh e2e/landing.sh)
