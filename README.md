# AIHarness

AI Control Center designed to facilitate multi-agent orchestration, context management, and cost optimization.

## Documentation

Full documentation is available in the [Docs/](Docs/index.md) directory.

- **[Architecture](Docs/Architecture/Overview.md)**: High-level overview and system design.
- **[Tools](Docs/Manual/Tools.md)**: Reference for available tools.
- **[Roadmap](Docs/Planning/Version-Roadmap.md)**: Version milestones and planning.
- **[Contributing](Docs/Contributing/AI-Guidelines.md)**: Guidelines for AI contributors.

## Quick Start

### Installation

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri dev
```

### Build & Install

```bash
# Build the application
npm run build:app

# Install to /Applications
./scripts/install-local.sh
```

### MCP Setup

To use AIHarness as an MCP server for other tools:

1. Install the app to `/Applications` using the script above.
2. Configure your MCP client to use the binary:

```json
{
  "mcpServers": {
    "aiharness": {
      "command": "/Applications/AIHarness.app/Contents/MacOS/aiharness",
      "args": ["--mcp-stdio-proxy"],
      "env": {
        "AIH_PORT": "8787"
      }
    }
  }
}
```

(Note: The app must be running for the proxy to connect).
