import { useEffect, useRef } from 'react';
import { useServerStore } from '../stores/serverStore';
import { Terminal, Trash2 } from 'lucide-react';

function EventLog() {
  const { rawLogs, clearRawLogs, status, toolCalls } = useServerStore();
  const containerRef = useRef<HTMLDivElement>(null);
  const prevStatus = useRef(status);
  const prevToolCallsLength = useRef(toolCalls.length);

  // Auto-scroll to bottom when new logs arrive
  useEffect(() => {
    if (containerRef.current) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight;
    }
  }, [rawLogs]);

  // Log status changes internally (we could also send these from Rust)
  useEffect(() => {
    if (prevStatus.current !== status) {
      console.log(`[EventLog] Status changed: ${prevStatus.current} -> ${status}`);
      prevStatus.current = status;
    }
  }, [status]);

  // Log when new tool calls arrive
  useEffect(() => {
    if (toolCalls.length > prevToolCallsLength.current) {
      const newCount = toolCalls.length - prevToolCallsLength.current;
      console.log(`[EventLog] ${newCount} new tool call(s) received`);
      prevToolCallsLength.current = toolCalls.length;
    }
  }, [toolCalls]);

  const getMessageColor = (source: string, message: string) => {
    // Try to parse as JSON event
    if (message.includes('"event"')) {
      if (message.includes('"tool_call_start"')) return 'text-yellow-400';
      if (message.includes('"tool_call_end"')) return 'text-green-400';
    }
    if (source.includes('stderr')) return 'text-gray-300';
    return 'text-blue-300';
  };

  return (
    <div className="panel">
      <div className="panel-header flex justify-between items-center">
        <div className="flex items-center gap-2">
          <Terminal className="w-5 h-5 text-gray-500" />
          <h2 className="text-lg font-semibold text-gray-800">Raw Event Log</h2>
        </div>
        <div className="flex items-center gap-4">
          <span className="text-sm text-gray-500">{rawLogs.length} lines</span>
          <button
            onClick={clearRawLogs}
            className="p-1.5 hover:bg-gray-200 rounded text-gray-500 hover:text-red-500 transition-colors"
            title="Clear logs"
          >
            <Trash2 className="w-4 h-4" />
          </button>
        </div>
      </div>

      {/* Raw log entries */}
      <div 
        ref={containerRef}
        className="max-h-64 overflow-y-auto font-mono text-xs bg-gray-900 p-2"
      >
        {rawLogs.length === 0 ? (
          <div className="text-center py-8 text-gray-500">
            <p>No raw events yet</p>
            <p className="text-xs mt-1">Start the server and make tool calls to see raw events</p>
          </div>
        ) : (
          <div className="space-y-0.5">
            {rawLogs.map((log, index) => {
              const timestamp = log.timestamp 
                ? log.timestamp.split('T')[1]?.replace('Z', '').split('.')[0] || log.timestamp
                : '';
              return (
                <div key={index} className="flex items-start gap-2">
                  <span className="text-gray-600 flex-shrink-0">[{timestamp}]</span>
                  <span className="text-gray-500 flex-shrink-0">[{log.source}]</span>
                  <span className={`break-all ${getMessageColor(log.source, log.message)}`}>
                    {log.message}
                  </span>
                </div>
              );
            })}
          </div>
        )}
      </div>

      {/* Tool calls summary */}
      {toolCalls.length > 0 && (
        <div className="mt-2 pt-2 border-t border-gray-200">
          <h3 className="text-sm font-medium text-gray-700 mb-1">Tool Calls</h3>
          <div className="space-y-1 max-h-32 overflow-y-auto">
            {toolCalls.slice(0, 5).map((call) => (
              <div 
                key={call.id} 
                className="flex items-center gap-2 text-xs p-1.5 bg-gray-50 rounded"
              >
                <span className={call.success ? 'text-green-600' : 'text-red-600'}>
                  {call.success ? '✓' : '✗'}
                </span>
                <span className="font-medium">{call.tool_name}</span>
                <span className="text-gray-400">{call.duration_ms}ms</span>
              </div>
            ))}
            {toolCalls.length > 5 && (
              <p className="text-xs text-gray-400 text-center">
                +{toolCalls.length - 5} more
              </p>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

export default EventLog;
