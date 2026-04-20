#!/usr/bin/env bash
# FEAT-CONTRIB-004

set -euo pipefail

usage() {
  cat >&2 <<'EOF'
Usage: scripts/ci/bootstrap-contributor-tooling.sh [--app] [--website] [--vscode] [--playwright] [--all]

Without flags, installs the default optional contributor surfaces:
  --app       Install browser-app npm dependencies
  --website   Install docs-site npm dependencies

Additional opt-in flags:
  --vscode      Install VS Code extension npm dependencies
  --playwright  Install Playwright Chromium after app dependencies are ready
  --all         Install app, website, vscode, and Playwright tooling
EOF
}

repo_root() {
  cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd
}

log_step() {
  printf '\n[bootstrap] %s\n' "$1"
}

install_app_deps() {
  log_step "Installing browser-app dependencies."
  scripts/ci/pinned-npm.sh install app
  npm --prefix app ci
}

install_website_deps() {
  log_step "Installing docs-site dependencies."
  scripts/ci/pinned-npm.sh install website
  bash scripts/ci/install-docs-site-deps.sh
}

install_vscode_deps() {
  log_step "Installing VS Code extension dependencies."
  scripts/ci/pinned-npm.sh install editors/vscode
  npm --prefix editors/vscode ci
}

install_playwright_browser() {
  log_step "Installing Playwright Chromium for local browser-app end-to-end coverage."
  npx --prefix app playwright install --with-deps chromium
}

print_next_steps() {
  local installed_app=$1
  local installed_website=$2
  local installed_vscode=$3
  local installed_playwright=$4

  printf '\n[bootstrap] Ready.\n'
  if [[ "$installed_app" -eq 1 ]]; then
    printf '[bootstrap] Next app checks: scripts/ci/check-browser-app-freshness.sh'
    if [[ "$installed_playwright" -eq 1 ]]; then
      printf ' && npm --prefix app run test:e2e'
    fi
    printf '\n'
  fi
  if [[ "$installed_website" -eq 1 ]]; then
    printf '[bootstrap] Next docs-site check: npm --prefix website run build\n'
  fi
  if [[ "$installed_vscode" -eq 1 ]]; then
    printf '[bootstrap] Next VS Code extension check: npm --prefix editors/vscode test\n'
  fi
}

main() {
  local root
  local install_app=0
  local install_website=0
  local install_vscode=0
  local install_playwright=0

  root="$(repo_root)"
  cd "$root"

  if [[ $# -eq 0 ]]; then
    install_app=1
    install_website=1
  fi

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --app)
        install_app=1
        ;;
      --website)
        install_website=1
        ;;
      --vscode)
        install_vscode=1
        ;;
      --playwright)
        install_playwright=1
        install_app=1
        ;;
      --all)
        install_app=1
        install_website=1
        install_vscode=1
        install_playwright=1
        ;;
      -h|--help)
        usage
        exit 0
        ;;
      *)
        usage
        exit 1
        ;;
    esac
    shift
  done

  if [[ "$install_app" -eq 1 ]]; then
    install_app_deps
  fi
  if [[ "$install_website" -eq 1 ]]; then
    install_website_deps
  fi
  if [[ "$install_vscode" -eq 1 ]]; then
    install_vscode_deps
  fi
  if [[ "$install_playwright" -eq 1 ]]; then
    install_playwright_browser
  fi

  print_next_steps "$install_app" "$install_website" "$install_vscode" "$install_playwright"
}

main "$@"
