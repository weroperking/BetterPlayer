#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
NODE_MODULES_DIR="$ROOT_DIR/node_modules"
TAURI_DIR="$ROOT_DIR/src-tauri"

usage() {
  cat <<'USAGE'
BetterPlayer helper

Usage:
  ./scripts/betterplayer.sh <command>

Commands:
  setup      Install/download project dependencies when missing or stale.
  dev        Ensure dependencies exist, then run the Tauri development app.
  frontend   Ensure dependencies exist, then run only the Vite frontend server.
  build      Ensure dependencies exist, then build the frontend and Tauri app.
  check      Run TypeScript and Rust compile checks without creating installers.
  test       Run available automated checks for the MVP codebase.
  clean      Remove generated frontend/Rust build output.
  doctor     Print local tool and runtime status.
  help       Show this help text.

Notes:
  - This script installs project packages, not OS-level toolchains.
  - Install Node.js 20+, npm, Rust stable, and the Tauri system prerequisites first.
  - On Windows MVP runs, place libmpv-wrapper.dll and libmpv-2.dll in src-tauri/lib/.
USAGE
}

log() { printf '\033[1;34m[BetterPlayer]\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[BetterPlayer warning]\033[0m %s\n' "$*" >&2; }
fail() { printf '\033[1;31m[BetterPlayer error]\033[0m %s\n' "$*" >&2; exit 1; }

need_command() {
  command -v "$1" >/dev/null 2>&1 || fail "Missing '$1'. Install it, then rerun: ./scripts/betterplayer.sh setup"
}

check_toolchain() {
  need_command node
  need_command npm
  need_command cargo
  need_command rustc
}

node_major() {
  node -p "Number(process.versions.node.split('.')[0])" 2>/dev/null || echo 0
}

ensure_toolchain() {
  check_toolchain
  local major
  major="$(node_major)"
  if [ "$major" -lt 20 ]; then
    fail "Node.js 20+ is required; found $(node --version)."
  fi
}

npm_needs_install() {
  [ ! -d "$NODE_MODULES_DIR" ] && return 0
  [ ! -f "$ROOT_DIR/package-lock.json" ] && return 0
  [ "$ROOT_DIR/package.json" -nt "$ROOT_DIR/package-lock.json" ] && return 0
  return 1
}

install_node_deps() {
  if npm_needs_install; then
    log "Installing Node dependencies with npm install..."
    (cd "$ROOT_DIR" && npm install)
  else
    log "Node dependencies already installed."
  fi
}

install_rust_deps() {
  log "Fetching Rust crates for the Tauri backend..."
  (cd "$TAURI_DIR" && cargo fetch)
}

warn_about_libmpv() {
  if [ ! -f "$TAURI_DIR/lib/libmpv-wrapper.dll" ] || [ ! -f "$TAURI_DIR/lib/libmpv-2.dll" ]; then
    warn "Windows libmpv DLLs are missing from src-tauri/lib/. The MVP UI/checks can run, but real mpv playback needs libmpv-wrapper.dll and libmpv-2.dll there."
  fi
}

setup() {
  ensure_toolchain
  install_node_deps
  install_rust_deps
  warn_about_libmpv
}

run_dev() {
  setup
  log "Starting BetterPlayer via Tauri dev..."
  (cd "$ROOT_DIR" && npm run tauri -- dev)
}

run_frontend() {
  ensure_toolchain
  install_node_deps
  log "Starting Vite frontend only..."
  (cd "$ROOT_DIR" && npm run dev)
}

run_build() {
  setup
  log "Building frontend and Tauri bundle..."
  (cd "$ROOT_DIR" && npm run tauri -- build)
}

run_check() {
  ensure_toolchain
  install_node_deps
  log "Running TypeScript check..."
  (cd "$ROOT_DIR" && npx tsc --noEmit)
  log "Running Rust check..."
  (cd "$TAURI_DIR" && cargo check)
}

run_test() {
  ensure_toolchain
  install_node_deps
  log "Running frontend build check..."
  (cd "$ROOT_DIR" && npm run build)
  log "Running Rust tests..."
  (cd "$TAURI_DIR" && cargo test)
}

run_clean() {
  log "Removing generated build output..."
  rm -rf "$ROOT_DIR/dist" "$TAURI_DIR/target"
}

run_doctor() {
  printf 'BetterPlayer doctor\n'
  printf '  root: %s\n' "$ROOT_DIR"
  printf '  node: %s\n' "$(command -v node >/dev/null 2>&1 && node --version || echo missing)"
  printf '  npm: %s\n' "$(command -v npm >/dev/null 2>&1 && npm --version || echo missing)"
  printf '  rustc: %s\n' "$(command -v rustc >/dev/null 2>&1 && rustc --version || echo missing)"
  printf '  cargo: %s\n' "$(command -v cargo >/dev/null 2>&1 && cargo --version || echo missing)"
  printf '  node_modules: %s\n' "$([ -d "$NODE_MODULES_DIR" ] && echo present || echo missing)"
  printf '  libmpv-wrapper.dll: %s\n' "$([ -f "$TAURI_DIR/lib/libmpv-wrapper.dll" ] && echo present || echo missing)"
  printf '  libmpv-2.dll: %s\n' "$([ -f "$TAURI_DIR/lib/libmpv-2.dll" ] && echo present || echo missing)"
}

case "${1:-help}" in
  setup) setup ;;
  dev) run_dev ;;
  frontend) run_frontend ;;
  build) run_build ;;
  check) run_check ;;
  test) run_test ;;
  clean) run_clean ;;
  doctor) run_doctor ;;
  help|-h|--help) usage ;;
  *) usage; fail "Unknown command: $1" ;;
esac
