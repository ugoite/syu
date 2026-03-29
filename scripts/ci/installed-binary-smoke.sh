#!/usr/bin/env bash
# FEAT-QUALITY-001

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
temp_root=""
app_pid=""

cleanup() {
  if [[ -n "${app_pid:-}" ]]; then
    kill "$app_pid" >/dev/null 2>&1 || true
    wait "$app_pid" 2>/dev/null || true
  fi
  if [[ -n "${temp_root:-}" && -d "${temp_root}" ]]; then
    rm -rf "${temp_root}"
  fi
}

resolve_binary_name() {
  case "$(uname -s)" in
    MINGW* | MSYS* | CYGWIN*) printf 'syu.exe\n' ;;
    *) printf 'syu\n' ;;
  esac
}

resolve_package_version() {
  python3 - <<'PY'
from pathlib import Path
import tomllib

cargo_toml = Path("Cargo.toml")
with cargo_toml.open("rb") as handle:
    data = tomllib.load(handle)
print(data["package"]["version"])
PY
}

reserve_port() {
  python3 - <<'PY'
import socket

sock = socket.socket()
sock.bind(("127.0.0.1", 0))
print(sock.getsockname()[1])
sock.close()
PY
}

wait_for_app_payload() {
  local port="$1"

  PORT="$port" python3 - <<'PY'
import json
import os
import sys
import time
import urllib.request

port = os.environ["PORT"]
url = f"http://127.0.0.1:{port}/api/app-data.json"

for _ in range(80):
    try:
        with urllib.request.urlopen(url, timeout=1) as response:
            payload = json.load(response)
        if payload.get("workspace_root") and payload.get("spec_root"):
            sys.exit(0)
    except Exception:
        time.sleep(0.1)

sys.exit(1)
PY
}

main() {
  local install_root binary_name installed_binary expected_version actual_version workspace port app_log

  trap cleanup EXIT
  cd "$repo_root"

  temp_root="$(mktemp -d)"
  install_root="${temp_root}/install"
  binary_name="$(resolve_binary_name)"
  installed_binary="${install_root}/bin/${binary_name}"
  expected_version="$(resolve_package_version)"
  workspace="${temp_root}/workspace"

  cargo install --path "$repo_root" --root "$install_root" --force

  actual_version="$("${installed_binary}" --version)"
  test "${actual_version}" = "syu ${expected_version}"

  "${installed_binary}" init "$workspace" >/dev/null
  test -f "${workspace}/syu.yaml"
  test -d "${workspace}/docs/syu"

  "${installed_binary}" validate "$workspace" >/dev/null

  port="$(reserve_port)"
  app_log="${temp_root}/app.log"
  "${installed_binary}" app "$workspace" --bind 127.0.0.1 --port "$port" >"$app_log" 2>&1 &
  app_pid="$!"

  wait_for_app_payload "$port"
}

main "$@"
