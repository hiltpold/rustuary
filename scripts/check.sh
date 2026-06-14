#!/usr/bin/env bash
set -euo pipefail

printf "==> Repo hygiene\n"
./scripts/check_repo_hygiene.sh

printf "==> Rust core\n"
if command -v cargo >/dev/null 2>&1; then
  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets -- -D warnings
  cargo test --workspace
  RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
else
  echo "cargo not found; skipping Rust checks"
fi

printf "==> Go services\n"
if command -v go >/dev/null 2>&1; then
  gofmt_files="$(cd services/api && gofmt -l .)"
  if [ -n "$gofmt_files" ]; then
    echo "gofmt required for:" >&2
    echo "$gofmt_files" >&2
    exit 1
  fi
  (cd services/api && go test ./... && go vet ./...)
  if command -v golangci-lint >/dev/null 2>&1; then
    (cd services/api && golangci-lint run)
  else
    echo "golangci-lint not found; skipping Go lint"
  fi
  if command -v govulncheck >/dev/null 2>&1; then
    (cd services/api && govulncheck ./...)
  else
    echo "govulncheck not found; skipping Go vulnerability check"
  fi
else
  echo "go not found; skipping Go checks"
fi

printf "==> UI\n"
if command -v pnpm >/dev/null 2>&1 && [ -d apps/workbench/node_modules ]; then
  (cd apps/workbench && pnpm check && pnpm lint)
else
  echo "pnpm or node_modules not found; skipping UI checks"
fi

printf "==> Done\n"
