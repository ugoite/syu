#!/usr/bin/env bash
# FEAT-INSTALL-001

set -euo pipefail

DEFAULT_REPOSITORY="ugoite/syu"
tmp_dir=""

cleanup_tmp_dir() {
  if [[ -n "${tmp_dir:-}" ]]; then
    rm -rf "$tmp_dir"
  fi
}

require_command() {
  local command_name="$1"
  if ! command -v "$command_name" >/dev/null 2>&1; then
    echo "required command not found: $command_name" >&2
    exit 1
  fi
}

find_python() {
  if command -v python3 >/dev/null 2>&1; then
    command -v python3
    return 0
  fi

  if command -v python >/dev/null 2>&1; then
    command -v python
    return 0
  fi

  echo "python3 or python is required by the installer" >&2
  exit 1
}

resolve_repository() {
  if [[ -n "${SYU_REPOSITORY:-}" ]]; then
    printf '%s\n' "$SYU_REPOSITORY"
    return 0
  fi

  if [[ -n "${GITHUB_REPOSITORY:-}" ]]; then
    printf '%s\n' "$GITHUB_REPOSITORY"
    return 0
  fi

  printf '%s\n' "$DEFAULT_REPOSITORY"
}

resolve_target_triple() {
  local os_name arch_name

  os_name="$(uname -s)"
  arch_name="$(uname -m)"

  case "$arch_name" in
    x86_64 | amd64) arch_name="x86_64" ;;
    arm64 | aarch64) arch_name="aarch64" ;;
    *)
      echo "unsupported architecture: $arch_name" >&2
      exit 1
      ;;
  esac

  case "$os_name" in
    Darwin) printf '%s\n' "${arch_name}-apple-darwin" ;;
    Linux) printf '%s\n' "${arch_name}-unknown-linux-gnu" ;;
    MINGW* | MSYS* | CYGWIN*) printf '%s\n' "${arch_name}-pc-windows-msvc" ;;
    *)
      echo "unsupported operating system: $os_name" >&2
      exit 1
      ;;
  esac
}

resolve_release_tag() {
  local repository="$1"
  local version="${SYU_VERSION:-latest}"
  local python_bin api_url

  if [[ "$version" != "latest" ]]; then
    printf '%s\n' "$version"
    return 0
  fi

  python_bin="$(find_python)"
  api_url="https://api.github.com/repos/${repository}/releases/latest"

  curl -fsSL "$api_url" | "$python_bin" -c '
import json
import sys

payload = json.load(sys.stdin)
tag_name = payload.get("tag_name")
if not tag_name:
    raise SystemExit("failed to resolve latest release tag")
print(tag_name)
'
}

extract_archive() {
  local archive_path="$1"
  local destination_dir="$2"
  local python_bin

  mkdir -p "$destination_dir"

  case "$archive_path" in
    *.tar.gz)
      tar -C "$destination_dir" -xzf "$archive_path"
      ;;
    *.zip)
      python_bin="$(find_python)"
      "$python_bin" - "$archive_path" "$destination_dir" <<'PY'
import sys
import zipfile
from pathlib import Path

archive_path = Path(sys.argv[1])
destination_dir = Path(sys.argv[2])

with zipfile.ZipFile(archive_path) as archive:
    archive.extractall(destination_dir)
PY
      ;;
    *)
      echo "unsupported archive format: $archive_path" >&2
      exit 1
      ;;
  esac
}

install_syu() {
  local repository tag target install_dir archive_name archive_url binary_name tmp_dir extracted_dir

  require_command curl
  require_command install

  repository="$(resolve_repository)"
  tag="$(resolve_release_tag "$repository")"
  target="$(resolve_target_triple)"
  install_dir="${SYU_INSTALL_DIR:-$HOME/.local/bin}"
  binary_name="syu"
  archive_name="syu-${target}.tar.gz"

  if [[ "$target" == *windows* ]]; then
    archive_name="syu-${target}.zip"
    binary_name="syu.exe"
  else
    require_command tar
  fi

  archive_url="https://github.com/${repository}/releases/download/${tag}/${archive_name}"
  tmp_dir="$(mktemp -d)"
  extracted_dir="${tmp_dir}/extract"

  trap cleanup_tmp_dir EXIT

  curl -fsSL "$archive_url" -o "${tmp_dir}/${archive_name}"
  extract_archive "${tmp_dir}/${archive_name}" "$extracted_dir"

  mkdir -p "$install_dir"
  install -m 0755 "${extracted_dir}/${binary_name}" "${install_dir}/${binary_name}"

  echo "installed ${binary_name} to ${install_dir}/${binary_name}"
}

install_syu "$@"
