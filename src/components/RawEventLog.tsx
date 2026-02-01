import { useEffect, useRef, useState } from 'react';
import { useServerStore } from '../stores/serverStore';
import { Terminal, Trash2, Play, Square } from 'lucide-react';

export default function RawEventLog() {
  const { rawLogs, clearRawLogs, status, toolCalls, startServer, stopServer } = useServerStore();
  const containerRef = useRef<HTMLDivElement>(null);
  const [autoScroll, setAutoScroll] = useState(true);

  // Auto-scroll when new logs arrive
  useEffect(() => {
    if (autoScroll && containerRef.current) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight;
    }
  }, [rawLogs, autoScroll]);

  const getMessageStyle = (message: string) => {
    if (message.includes('"tool_call_start"')) return 'text-yellow-400 font-medium';
    if (message.includes('"tool_call_end"')) return 'text-green-400 font-medium';
    if (message.includes('"error"')) return 'text-red-400';
    return 'text-gray-300';
  };

  const formatTimestamp = (timestamp: string) => {
    try {
      const date = new Date(timestamp);
      return date.toLocaleTimeString('en-US', { 
        hour12: false,
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit'
      });
    } catch {
      return timestamp?.split('T')[1]?.split('.')[0] || '--:--:--';
    }
  };

  return (
    <div className="panel">
      {/* Header */}
      <div className="panel-header">
        <div className="flex items-center gap-2">
          <Terminal className="w-5 h-5 text-gray-500" />
          <h2 className="text-lg font-semibold text-gray-800">Raw Event Log</h2>
          <span className={`ml-2 px-2 py-0.5 text-xs rounded-full ${
            status === 'running' ? 'bg-green-100 text-green-700' :
            status === 'starting' ? 'bg-yellow-100 text-yellow-700' :
            status === 'error' ? 'bg-red-100 text-red-700' :
            'bg-gray-100 text-gray-600'
          }`}>
            {status}
          </span>
        </div>
        <div className="flex items-center gap-2">
          <label className="flex items-center text-xs text-gray-500 mr-2 cursor-pointer">
            <input
              type="checkbox"
              checked={autoScroll}
              onChange={(e) => setAutoScroll(e.target.checked)}
              className="mr-1"
            />
            Auto-scroll
          </label>
          <span className="text-sm text-gray-500 px-2">{rawLogs.length} lines</span>
          <button
            onClick={clearRawLogs}
            className="p-1.5 hover:bg-gray-200 rounded text-gray-500 hover:text-red-500 transition-colors"
            title="Clear logs"
          >
            <Trash2 className="w-4 h-4" />
          </button>
          {status === 'running' ? (
            <button
              onClick={stopServer}
              className="p-1.5 hover:bg-red-100 rounded text-red-600 transition-colors"
              title="Stop server"
            >
              <Square className="w-4 h-4" />
            </button>
          ) : (
            <button
              onClick={startServer}
              disabled={status === 'starting'}
              className="p-1.5 hover:bg-green-100 rounded text-green-600 transition-colors disabled:opacity-50"
              title="Start server"
            >
              <Play className="w-4 h-4" />
            </button>
          )}
        </div>
      </div>

      {/* Log display */}
      <div 
        ref={containerRef}
        className="h-64 overflow-y-auto bg-gray-900 p-3 font-mono text-xs"
        onScroll={() => {
          if (containerRef.current) {
            const { scrollTop, scrollHeight, clientHeight } = containerRef.current;
            const isAtBottom = scrollHeight - scrollTop - clientHeight < 10;
            setAutoScroll(isAtBottom);
          }
        }}
      >
        {rawLogs.length === 0 ? (
          <div className="h-full flex flex-col items-center justify-center text-gray-500">
            <Terminal className="w-8 h-8 mb-2 opacity-50" />
            <p>No events yet</p>
            <p className="text-xs mt-1">Start the server to see tool events</p>
          </div>
        ) : (
          <div className="space-y-1">
            {rawLogs.map((log, index) => (
              <div key={index} className="flex items-start gap-2 hover:bg-gray-800 rounded px-1 -mx-1">
                <span className="text-gray-600 flex-shrink-0">
                  {formatTimestamp(log.timestamp)}
                </span>
                <span className={`break-all ${getMessageStyle(log.message)}`}>
                  {log.message}
                </span>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Stats footer */}
      {(rawLogs.length > 0 || toolCalls.length > 0) && (
        <div className="mt-3 pt-3 border-t border-gray-200 flex justify-between text-xs text-gray-500">
          <div className="flex gap-4">
            <span>Events: {rawLogs.length}</span>
            <span>Tool calls: {toolCalls.length}</span>
            <span>Successful: {toolCalls.filter(c => c.success).length}</span>
            <span>Failed: {toolCalls.filter(c => !c.success).length}</span>
          </div>
          <div>
            {status === 'running' ? (
              <span className="text-green-600">● Server running</span>
            ) : (
              <span className="text-gray-400">○ Server stopped</span>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
