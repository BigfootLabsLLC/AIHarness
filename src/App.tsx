import ServerStatus from './components/ServerStatus';
import ToolCallLog from './components/ToolCallLog';
import ContextPanel from './components/ContextPanel';

function App() {
  return (
    <div className="min-h-screen bg-gray-50">
      {/* Header */}
      <header className="bg-white shadow-sm border-b border-gray-200">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-2xl font-bold text-gray-900">AIHarness</h1>
              <p className="text-sm text-gray-500">MCP Server Control Center</p>
            </div>
            <div className="text-sm text-gray-400">
              v0.1.0
            </div>
          </div>
        </div>
      </header>

      {/* Main content */}
      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Left column - Server Status */}
          <div className="lg:col-span-1">
            <ServerStatus />
          </div>

          {/* Middle column - Tool Call Log */}
          <div className="lg:col-span-1">
            <ToolCallLog />
          </div>

          {/* Right column - Context Files */}
          <div className="lg:col-span-1">
            <ContextPanel />
          </div>
        </div>

        {/* Instructions */}
        <div className="mt-8 panel">
          <h2 className="panel-header">Getting Started</h2>
          <div className="prose prose-sm text-gray-600">
            <p>
              AIHarness MCP Server is running. To use it with Claude Desktop:
            </p>
            <ol className="list-decimal list-inside space-y-2 mt-2">
              <li>
                Add the following to your Claude Desktop config (
                <code>~/Library/Application Support/Claude/claude_desktop_config.json</code>):
              </li>
            </ol>
            <pre className="mt-3 p-3 bg-gray-100 rounded text-xs overflow-x-auto">
{`{
  "mcpServers": {
    "aiharness": {
      "command": "/path/to/aiharness-mcp-server",
      "env": {
        "AIH_DATA_DIR": "/path/to/data"
      }
    }
  }
}`}
            </pre>
            <p className="mt-3">
              Restart Claude Desktop, then ask: "Use the aiharness tool to read my todo list"
            </p>
          </div>
        </div>
      </main>
    </div>
  );
}

export default App;
