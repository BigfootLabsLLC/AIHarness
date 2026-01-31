#!/bin/bash
# Launch AIHarness GUI app from build

APP_PATH="/Users/danbaker/Projects/AIHarness/AIHarness/src-tauri/target/release/bundle/macos/AIHarness.app"

if [ ! -d "$APP_PATH" ]; then
    echo "App not found at $APP_PATH"
    echo "Building first..."
    cd /Users/danbaker/Projects/AIHarness/AIHarness
    npm run tauri build
fi

open "$APP_PATH"
