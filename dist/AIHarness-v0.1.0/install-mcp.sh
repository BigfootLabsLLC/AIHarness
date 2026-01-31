#!/bin/bash
# Install AIHarness MCP Server for use with Claude Desktop

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MCP_BINARY="$SCRIPT_DIR/aiharness-mcp-server"
INSTALL_DIR="$HOME/.local/bin"
DATA_DIR="$HOME/.aiharness"
CONFIG_FILE="$HOME/Library/Application Support/Claude/claude_desktop_config.json"

echo "=== AIHarness MCP Server Installer ==="
echo ""

# Check if running on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo "Error: This installer is for macOS only."
    exit 1
fi

# Create directories
echo "Creating directories..."
mkdir -p "$INSTALL_DIR"
mkdir -p "$DATA_DIR"

# Copy binary
echo "Installing MCP server to $INSTALL_DIR..."
cp "$MCP_BINARY" "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/aiharness-mcp-server"

echo ""
echo "Installation complete!"
echo ""
echo "Add this to your Claude Desktop config:"
echo "  $CONFIG_FILE"
echo ""
cat <<EOCONFIG
{
  "mcpServers": {
    "aiharness": {
      "command": "$INSTALL_DIR/aiharness-mcp-server",
      "env": {
        "AIH_DATA_DIR": "$DATA_DIR"
      }
    }
  }
}
EOCONFIG

echo ""
echo "Then restart Claude Desktop!"
