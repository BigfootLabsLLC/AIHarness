#!/bin/bash
# Deploy the current release binary to a stable location for MCP use

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
BIN_DIR="$ROOT_DIR/bin"
SOURCE_BIN="$ROOT_DIR/src-tauri/target/release/aiharness"

# Check if source exists
if [ ! -f "$SOURCE_BIN" ]; then
    echo "Error: Release binary not found at:"
    echo "  $SOURCE_BIN"
    echo "Please run 'npm run build:app' first."
    exit 1
fi

# Create bin dir
mkdir -p "$BIN_DIR"

# Copy binary
echo "Deploying binary to $BIN_DIR/aiharness..."
cp "$SOURCE_BIN" "$BIN_DIR/aiharness"

echo "Success! MCP server deployed."
echo "Path: $BIN_DIR/aiharness"
