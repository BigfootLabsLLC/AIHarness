# AIHarness v0.1.0 - MCP Server

AIHarness is a local MCP (Model Context Protocol) server that provides AI assistants with secure access to your files and tools.

## What's Included

| File | Description |
|------|-------------|
| `AIHarness_0.1.0_aarch64.dmg` | macOS installer (Apple Silicon) |
| `AIHarness.app.zip` | Standalone macOS app (unzip and run) |
| `aiharness-mcp-server` | Standalone MCP server binary |

## Quick Start

### Option 1: Install via DMG (Recommended)

1. Double-click `AIHarness_0.1.0_aarch64.dmg`
2. Drag `AIHarness.app` to your Applications folder
3. Launch AIHarness from Applications

### Option 2: Run Standalone

1. Unzip `AIHarness.app.zip`
2. Double-click `AIHarness.app`

### Option 3: Use MCP Server with Claude Desktop

1. Copy the MCP server binary to a permanent location:
   ```bash
   mkdir -p ~/.local/bin
   cp aiharness-mcp-server ~/.local/bin/
   chmod +x ~/.local/bin/aiharness-mcp-server
   ```

2. Create data directory:
   ```bash
   mkdir -p ~/.aiharness
   ```

3. Edit Claude Desktop config (`~/Library/Application Support/Claude/claude_desktop_config.json`):
   ```json
   {
     "mcpServers": {
       "aiharness": {
         "command": "/Users/YOUR_USERNAME/.local/bin/aiharness-mcp-server",
         "env": {
           "AIH_DATA_DIR": "/Users/YOUR_USERNAME/.aiharness"
         }
       }
     }
   }
   ```

4. Restart Claude Desktop

5. Test it:
   ```
   Use the aiharness tool to list the current directory
   ```

## Available Tools

### `read_file`
Read the contents of a file.
- **Input**: `{"path": "/absolute/path/to/file"}`
- **Limit**: 1MB max file size

### `write_file`
Write content to a file (creates directories if needed).
- **Input**: `{"path": "/absolute/path/to/file", "content": "text to write"}`

### `list_directory`
List files and directories.
- **Input**: `{"path": "/absolute/path", "recursive": false}`

### `search_files`
Search for text in files.
- **Input**: `{"path": "/absolute/path", "pattern": "search text", "recursive": true}`

## Security

- **All paths must be absolute** - no relative paths allowed
- **Local only** - Server binds to localhost, no external network access
- **File size limits** - Files over 1MB are rejected (prevents context overflow)

## Requirements

- macOS 11.0+ (Apple Silicon)
- Claude Desktop (for MCP integration)

## Support

- Version: 0.1.0
- License: See LICENSE file
- Issues: https://github.com/BigfootLabsLLC/AIHarness/issues
