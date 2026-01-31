import { useState } from 'react';
import { useServerStore } from '../stores/serverStore';
import type { ToolCall } from '../types';

function ToolCallLog() {
  const { toolCalls, clearToolCalls } = useServerStore();
  const [expandedId, setExpandedId] = useState<string | null>(null);

  const toggleExpand = (id: string) => {
    setExpandedId(expandedId === id ? null : id);
  };

  const formatTime = (timestamp: string) => {
    return new Date(timestamp).toLocaleTimeString();
  };

  const truncate = (str: string, len: number) => {
    if (str.length <= len) return str;
    return str.slice(0, len) + '...';
  };

  return (
    <div className="panel">
      <div className="flex items-center justify-between mb-4 pb-2 border-b border-gray-200">
        <h2 className="text-lg font-semibold text-gray-800">Tool Calls</h2>
        {toolCalls.length > 0 && (
          <button
            onClick={clearToolCalls}
            className="text-xs text-gray-500 hover:text-gray-700"
          >
            Clear
          </button>
        )}
      </div>

      {toolCalls.length === 0 ? (
        <div className="text-center py-8 text-gray-400">
          <p className="text-sm">No tool calls yet</p>
          <p className="text-xs mt-1">Tool calls will appear here when AI uses them</p>
        </div>
      ) : (
        <div className="space-y-3 max-h-96 overflow-y-auto">
          {toolCalls.map((call) => (
            <ToolCallItem
              key={call.id}
              call={call}
              isExpanded={expandedId === call.id}
              onToggle={() => toggleExpand(call.id)}
              formatTime={formatTime}
              truncate={truncate}
            />
          ))}
        </div>
      )}
    </div>
  );
}

interface ToolCallItemProps {
  call: ToolCall;
  isExpanded: boolean;
  onToggle: () => void;
  formatTime: (timestamp: string) => string;
  truncate: (str: string, len: number) => string;
}

function ToolCallItem({ call, isExpanded, onToggle, formatTime, truncate }: ToolCallItemProps) {
  return (
    <div 
      className={`border rounded-lg overflow-hidden ${
        call.success 
          ? 'border-gray-200 bg-white' 
          : 'border-red-200 bg-red-50'
      }`}
    >
      {/* Header */}
      <button
        onClick={onToggle}
        className="w-full px-4 py-3 flex items-center justify-between hover:bg-gray-50 transition-colors"
      >
        <div className="flex items-center space-x-3">
          <span className={`w-2 h-2 rounded-full ${
            call.success ? 'bg-green-500' : 'bg-red-500'
          }`} />
          <span className="font-medium text-sm text-gray-900">
            {call.toolName}
          </span>
        </div>
        <div className="flex items-center space-x-3">
          {call.durationMs && (
            <span className="text-xs text-gray-400">
              {call.durationMs}ms
            </span>
          )}
          <span className="text-xs text-gray-400">
            {formatTime(call.timestamp)}
          </span>
          <svg
            className={`w-4 h-4 text-gray-400 transform transition-transform ${
              isExpanded ? 'rotate-180' : ''
            }`}
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
          </svg>
        </div>
      </button>

      {/* Expanded content */}
      {isExpanded && (
        <div className="px-4 pb-4 border-t border-gray-100">
          {/* Arguments */}
          <div className="mt-3">
            <div className="text-xs font-semibold text-gray-500 uppercase tracking-wide mb-1">
              Arguments
            </div>
            <pre className="text-xs bg-gray-100 rounded p-2 overflow-x-auto">
              {JSON.stringify(call.arguments, null, 2)}
            </pre>
          </div>

          {/* Result */}
          <div className="mt-3">
            <div className="text-xs font-semibold text-gray-500 uppercase tracking-wide mb-1">
              Result
            </div>
            <pre className={`text-xs rounded p-2 overflow-x-auto max-h-48 ${
              call.success ? 'bg-gray-100' : 'bg-red-100 text-red-800'
            }`}>
              {truncate(call.content, 1000)}
            </pre>
          </div>
        </div>
      )}

      {/* Collapsed preview */}
      {!isExpanded && (
        <div className="px-4 pb-3">
          <p className="text-xs text-gray-500 truncate">
            {truncate(call.content, 80)}
          </p>
        </div>
      )}
    </div>
  );
}

export default ToolCallLog;
