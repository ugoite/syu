#!/usr/bin/env bash
# FEAT-QUALITY-001

set -euo pipefail

ensure_app_dependencies() {
  npm ci
}

snapshot_wasm_bindings() {
  if [[ ! -d src/wasm ]]; then
    printf '__missing__\n'
    return 0
  fi

  while IFS= read -r path; do
    sha256sum "$path"
  done < <(find src/wasm -maxdepth 1 -type f | LC_ALL=C sort)
}

check_browser_app_freshness() {
  local repo_root
  local app_dir
  local before_snapshot
  local after_snapshot

  repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
  app_dir="${repo_root}/app"
  before_snapshot="$(mktemp)"
  after_snapshot="$(mktemp)"
  trap 'rm -f "${before_snapshot:-}" "${after_snapshot:-}"' EXIT

  cd "$app_dir"
  snapshot_wasm_bindings >"$before_snapshot"
  ensure_app_dependencies
  npm run build:wasm
  snapshot_wasm_bindings >"$after_snapshot"

  cd "$repo_root"
  if ! cmp -s "$before_snapshot" "$after_snapshot"; then
    echo "Checked-in browser Wasm bindings under app/src/wasm are stale." >&2
    echo "Commit the regenerated app/src/wasm output after reviewing the build diff." >&2
    git --no-pager status --short -- app/src/wasm >&2
    git --no-pager diff --stat -- app/src/wasm >&2
    exit 1
  fi

  cd "$app_dir"
  npm run check
  npm run build

  if [[ ! -f dist/index.html ]]; then
    echo "Browser app build did not produce app/dist/index.html." >&2
    exit 1
  fi
}

check_browser_app_freshness "$@"
