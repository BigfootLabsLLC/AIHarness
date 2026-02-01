#!/bin/bash
# Install the built application to /Applications for system-wide use and stable MCP path

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
# Using the direct target output to ensure we get the freshest build
BUILD_OUTPUT="$ROOT_DIR/src-tauri/target/release/bundle/macos/AIHarness.app"
INSTALL_DIR="/Applications"
APP_NAME="AIHarness.app"
DEST_PATH="$INSTALL_DIR/$APP_NAME"

# Check if build exists
if [ ! -d "$BUILD_OUTPUT" ]; then
    echo "Error: Build output not found at:"
    echo "  $BUILD_OUTPUT"
    echo "Please run 'npm run build:app' first."
    exit 1
fi

echo "Installing AIHarness to $INSTALL_DIR..."

# Remove existing install if present
if [ -d "$DEST_PATH" ]; then
    echo "Removing existing installation..."
    rm -rf "$DEST_PATH"
fi

# Copy new version
echo "Copying application bundle..."
cp -R "$BUILD_OUTPUT" "$DEST_PATH"

# Verify
if [ -d "$DEST_PATH" ]; then
    echo "Success! AIHarness installed to $DEST_PATH"
    echo ""
    echo "For MCP Configuration (Stable Path):"
    echo "  $DEST_PATH/Contents/MacOS/aiharness"
else
    echo "Error: Installation failed."
    exit 1
fi
