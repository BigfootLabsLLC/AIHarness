#!/bin/bash
# Launch AIHarness GUI app from build

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_PATH="$SCRIPT_DIR/src-tauri/target/release/bundle/macos/AIHarness.app"

echo "=== AIHarness Launcher ==="
echo ""

# Check if app exists
if [ ! -d "$APP_PATH" ]; then
    echo "App not found at:"
    echo "  $APP_PATH"
    echo ""
    echo "Building now (this may take a few minutes)..."
    echo ""
    cd "$SCRIPT_DIR"
    npm run tauri build
    echo ""
    echo "Build complete!"
else
    echo "Found app at:"
    echo "  $APP_PATH"
fi

echo ""
echo "Launching AIHarness..."
echo ""

# Kill any existing instance first
if pgrep -f "AIHarness" > /dev/null; then
    echo "Stopping existing instance..."
    pkill -f "AIHarness" || true
    sleep 1
fi

# Open the app
open "$APP_PATH"

echo "AIHarness started!"
echo ""
echo "If the app doesn't appear, check:"
echo "  1. System Preferences > Security & Privacy - allow AIHarness"
echo "  2. Or right-click the app and select 'Open'"
echo ""
