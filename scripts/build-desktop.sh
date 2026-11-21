#!/usr/bin/env bash
set -euo pipefail

# Build the desktop app with the Rust gateway embedded in the sidecar.
# Usage: scripts/build-desktop.sh [linux|win|dir]
#
# Linux AppImage/deb builds are expected to run on a generic Linux host
# (Ubuntu CI). On NixOS, use `nix build .#twatch-electron` instead.

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET="${1:-linux}"

cd "$ROOT/web"
npm ci
npm run build

cd "$ROOT"
cargo build --release -p twatchd --features embed-web

mkdir -p electron/resources
case "$(uname -s)" in
  MINGW* | MSYS* | CYGWIN*)
    cp target/release/twatchd.exe electron/resources/twatchd.exe
    ;;
  *)
    cp target/release/twatchd electron/resources/twatchd
    ;;
esac

cd "$ROOT/electron"
npm ci

case "$TARGET" in
  linux)
    npx electron-builder --linux AppImage deb
    ;;
  win)
    npx electron-builder --win portable nsis
    ;;
  dir)
    npx electron-builder --dir
    ;;
  *)
    echo "Unknown target: $TARGET (expected linux, win or dir)" >&2
    exit 2
    ;;
esac
