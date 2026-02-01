import { useEffect } from 'react';
import { useServerStore } from '../stores/serverStore';
import type { ServerStatusState as Status } from '../types';

const statusConfig: Record<Status, { label: string; className: string }> = {
  stopped: { 
    label: 'Stopped', 
    className: 'bg-gray-100 text-gray-600 border-gray-200' 
  },
  starting: { 
    label: 'Starting...', 
    className: 'bg-yellow-50 text-yellow-700 border-yellow-200' 
  },
  running: { 
    label: 'Running', 
    className: 'bg-green-50 text-green-700 border-green-200' 
  },
  error: { 
    label: 'Error', 
    className: 'bg-red-50 text-red-700 border-red-200' 
  },
};

function ServerStatus() {
  const { status, port, error, startServer, stopServer, initialize } = useServerStore();

  // Initialize on mount
  useEffect(() => {
    initialize();
  }, [initialize]);

  const handleToggleServer = async () => {
    if (status === 'running') {
      await stopServer();
    } else if (status === 'stopped' || status === 'error') {
      await startServer();
    }
  };

  const statusStyle = statusConfig[status];

  return (
    <div className="panel">
      <h2 className="panel-header">Server Status</h2>
      
      {/* Status badge */}
      <div className={`inline-flex items-center px-3 py-1 rounded-full text-sm font-medium border ${statusStyle.className}`}>
        <span className={`w-2 h-2 rounded-full mr-2 ${
          status === 'running' ? 'bg-green-500 animate-pulse' : 
          status === 'starting' ? 'bg-yellow-500 animate-pulse' :
          status === 'error' ? 'bg-red-500' : 'bg-gray-400'
        }`} />
        {statusStyle.label}
      </div>

      {/* Error message */}
      {error && (
        <div className="mt-4 p-3 bg-red-50 border border-red-200 rounded-lg text-sm text-red-700">
          {error}
        </div>
      )}

      {/* Control button */}
      <button
        onClick={handleToggleServer}
        disabled={status === 'starting'}
        className={`mt-6 w-full btn ${
          status === 'running' 
            ? 'btn-danger' 
            : status === 'starting'
            ? 'opacity-50 cursor-not-allowed bg-gray-300 text-gray-700'
            : 'btn-primary'
        }`}
      >
        {status === 'running' ? 'Stop Server' : 
         status === 'starting' ? 'Starting...' : 
         'Start Server'}
      </button>

      {/* Stats */}
      {status === 'running' && (
        <div className="mt-6 pt-4 border-t border-gray-200">
          <div className="grid grid-cols-2 gap-4 text-center">
            <div>
              <div className="text-2xl font-bold text-gray-900">Active</div>
              <div className="text-xs text-gray-500 uppercase tracking-wide">Status</div>
            </div>
            <div>
              <div className="text-2xl font-bold text-gray-900">HTTP</div>
              <div className="text-xs text-gray-500 uppercase tracking-wide">Port {port}</div>
            </div>
          </div>
        </div>
      )}

      {/* Instructions */}
      <div className="mt-4 text-xs text-gray-500">
        {status === 'running' ? (
          <p>HTTP server running on port {port}. AI tools can connect via http://127.0.0.1:{port}</p>
        ) : (
          <p>Start the server to enable HTTP API for AI tools.</p>
        )}
      </div>
    </div>
  );
}

export default ServerStatus;
