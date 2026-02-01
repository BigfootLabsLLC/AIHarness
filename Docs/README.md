# AIHarness

AI Control Center with HTTP tool API for AI tools.

## Architecture

Single-process application with built-in HTTP server:

```
┌─────────────────────────────────────────┐
│           AIHarness App                 │
│  ┌─────────┐  ┌─────────────────────┐  │
│  │   GUI   │  │   HTTP Server       │◄─┼── AI tools connect here
│  │ (human) │  │   (port 8787)       │  │
│  └────┬────┘  └──────────┬──────────┘  │
│       │                  │              │
│       └──────┬───────────┘              │
│              ↓                          │
│       ┌──────────────┐                  │
│       │  Tool Engine │                  │
│       │  (Rust)      │                  │
│       └──────────────┘                  │
│              │                          │
│       emits events                      │
│              ↓                          │
│       ┌──────────────┐                  │
│       │  SQLite DB   │                  │
│       └──────────────┘                  │
└─────────────────────────────────────────┘
```

## HTTP API

### Endpoints

- `GET /` - Health check
- `GET /tools` - List available tools
- `POST /call` - Execute a tool
- `POST /mcp` - MCP JSON-RPC over HTTP
- `GET /events` - Get event history

The HTTP server auto-starts when the app launches in normal UI mode. You can stop/start it from the UI.

### Example

```bash
# List tools
curl http://127.0.0.1:8787/tools

# Read a file
curl -X POST http://127.0.0.1:8787/call \
  -H "Content-Type: application/json" \
  -d '{"name":"read_file","arguments":{"path":"/tmp/test.txt"}}'
```

## Tools

- `read_file` - Read file contents
- `write_file` - Write content to a file  
- `list_directory` - List directory contents
- `search_files` - Search for text in files

## Development

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build release
npm run tauri build

# Build and collect app in build/
npm run build:app

# Output: build/AIHarness.app
```

`npm run build:app` runs the Rust test suite before building the app.

## MCP Stdio Proxy (Optional)

When an MCP client only supports stdio, launch AIHarness in proxy mode. It forwards
JSON-RPC over stdio to the running app's HTTP MCP endpoint.

```bash
# Requires the main app to be running
AIH_PORT=8787 aiharness --mcp-stdio-proxy
```

## License

MIT
