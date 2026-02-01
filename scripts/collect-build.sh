#!/bin/bash
# Collect build outputs into the build/ folder for easy access

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
BUILD_DIR="$ROOT_DIR/build"
SOURCE_APP="$ROOT_DIR/src-tauri/target/release/bundle/macos/AIHarness.app"

mkdir -p "$BUILD_DIR"

if [ ! -d "$SOURCE_APP" ]; then
  echo "Build output not found:"
  echo "  $SOURCE_APP"
  echo "Run 'npm run tauri build' first."
  exit 1
fi

DEST_APP="$BUILD_DIR/AIHarness.app"

if [ -d "$DEST_APP" ]; then
  TS=$(date +"%Y%m%d-%H%M%S")
  mv "$DEST_APP" "$BUILD_DIR/AIHarness.app.$TS"
fi

cp -R "$SOURCE_APP" "$DEST_APP"

echo "Build output available at:"
echo "  $DEST_APP"
