# AIHarness Usage Guide

## Starting the GUI Application

### Method 1: Install to Applications
1. Open `AIHarness_0.1.0_aarch64.dmg`
2. Drag `AIHarness.app` to Applications folder
3. Launch from Applications or Spotlight

### Method 2: Run Without Installing
1. Unzip `AIHarness.app.zip`
2. Double-click `AIHarness.app`
3. If macOS warns about untrusted developer:
   - Right-click the app
   - Select "Open"
   - Click "Open" in the dialog

## GUI Features

### Server Status Panel
- **Start Server**: Begins MCP server on localhost
- **Stop Server**: Stops the MCP server
- **Status Indicator**: Shows running/stopped state

### Tool Call Log
- Shows real-time tool executions
- Click any call to expand and see:
  - Arguments sent to the tool
  - Results returned
  - Execution time
- Use "Clear" to reset the log

### Context Files Panel
- Shows files currently in AI context
- Click "+ Add" to add files for AI access
- Click the X to remove files from context

## Using with Claude Desktop

### One-Time Setup

1. **Install MCP Server**:
   ```bash
   ./install-mcp.sh
   ```

2. **Or manually copy**:
   ```bash
   mkdir -p ~/.local/bin
   cp aiharness-mcp-server ~/.local/bin/
   chmod +x ~/.local/bin/aiharness-mcp-server
   mkdir -p ~/.aiharness
   ```

3. **Configure Claude Desktop**:
   
   Edit `~/Library/Application Support/Claude/claude_desktop_config.json`:
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

4. **Restart Claude Desktop**

### Example Prompts

Once configured, you can ask Claude:

```
Read my README.md file
```
```
List all files in my project directory
```
```
Search for "TODO" in my codebase
```
```
Write a summary to /tmp/summary.txt
```

### How It Works

1. You ask Claude to use a tool
2. Claude sends a request to the MCP server
3. AIHarness executes the tool
4. Results are returned to Claude
5. Claude continues the conversation with the results

## Standalone MCP Server

Run the server directly:

```bash
./aiharness-mcp-server
```

The server:
- Reads JSON-RPC requests from stdin
- Writes responses to stdout
- Logs to stderr

### Testing Manually

```bash
# Start server
./aiharness-mcp-server

# Send a request (in another terminal)
echo '{"jsonrpc":"2.0","method":"initialize","id":1}' | ./aiharness-mcp-server
```

## Troubleshooting

### "App can't be opened"
macOS Gatekeeper is blocking the app.

**Solution**: Right-click the app → "Open" → "Open"

### "Permission denied" on MCP server
The binary isn't executable.

**Solution**: `chmod +x aiharness-mcp-server`

### MCP server not found
Path in Claude config is incorrect.

**Solution**: 
1. Find the full path: `which aiharness-mcp-server`
2. Update config with correct path
3. Restart Claude Desktop

### "File not found" errors
AIHarness requires **absolute paths**.

**Correct**: `/Users/me/project/file.txt`  
**Incorrect**: `file.txt` or `./file.txt`

### Can't connect to server
The MCP server may not be running or configured.

**Check**:
1. Is AIHarness GUI showing "Running"?
2. Is the path in Claude config correct?
3. Did you restart Claude after config changes?

## Data Storage

- **Database**: `~/.aiharness/aiharness.db`
- **Context files**: Stored as absolute paths in database
- **Tool history**: Last 100 calls kept in memory (GUI only)

## Uninstalling

1. Delete the app: `rm -rf /Applications/AIHarness.app`
2. Delete MCP server: `rm ~/.local/bin/aiharness-mcp-server`
3. Delete data: `rm -rf ~/.aiharness`
4. Remove from Claude config
