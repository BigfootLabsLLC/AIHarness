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

# Check if Claude config exists
if [ -f "$CONFIG_FILE" ]; then
    echo ""
    echo "Claude Desktop config found at:"
    echo "  $CONFIG_FILE"
    echo ""
    echo "Add the following to your config:"
    echo ""
    cat <<EOF
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
EOF
else
    echo ""
    echo "Claude Desktop config not found at expected location."
    echo "Create the file manually at:"
    echo "  ~/Library/Application Support/Claude/claude_desktop_config.json"
    echo ""
    echo "With this content:"
    cat <<EOF
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
EOF
fi

echo ""
echo "=== Installation Complete ==="
echo ""
echo "Next steps:"
echo "1. Add the MCP server config to Claude Desktop (see above)"
echo "2. Restart Claude Desktop"
echo "3. Test with: 'Use aiharness to read my todo list'"
echo ""
echo "Data directory: $DATA_DIR"
echo "Binary location: $INSTALL_DIR/aiharness-mcp-server"
