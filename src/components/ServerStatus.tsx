import { useServerStore } from '../stores/serverStore';
import type { ServerStatus as Status } from '../types';

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
  const { status, port, error, setStatus, setPort, setError } = useServerStore();

  const handleToggleServer = () => {
    if (status === 'running') {
      // Stop server
      setStatus('stopped');
      setPort(null);
      setError(null);
    } else if (status === 'stopped') {
      // Start server
      setStatus('starting');
      // Simulate server start (in real implementation, this would call Tauri command)
      setTimeout(() => {
        setStatus('running');
        setPort(8080);
      }, 1000);
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

      {/* Port info */}
      {port && (
        <div className="mt-4 text-sm">
          <span className="text-gray-500">Port:</span>
          <code className="ml-2 px-2 py-1 bg-gray-100 rounded text-gray-700 font-mono">
            {port}
          </code>
        </div>
      )}

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
              <div className="text-2xl font-bold text-gray-900">-</div>
              <div className="text-xs text-gray-500 uppercase tracking-wide">Clients</div>
            </div>
            <div>
              <div className="text-2xl font-bold text-gray-900">-</div>
              <div className="text-xs text-gray-500 uppercase tracking-wide">Uptime</div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default ServerStatus;
