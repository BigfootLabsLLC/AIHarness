import { useState } from 'react';
import { useServerStore } from '../stores/serverStore';
import type { ContextFile } from '../types';

// Mock data for demonstration
const mockFiles: ContextFile[] = [
  {
    id: '1',
    path: '/Users/user/project/README.md',
    addedAt: new Date().toISOString(),
  },
  {
    id: '2', 
    path: '/Users/user/project/src/main.rs',
    addedAt: new Date(Date.now() - 3600000).toISOString(),
    lastReadAt: new Date().toISOString(),
  },
];

function ContextPanel() {
  const { contextFiles, setContextFiles, addContextFile, removeContextFile } = useServerStore();
  const [newPath, setNewPath] = useState('');
  const [isAdding, setIsAdding] = useState(false);

  // Load mock data on first render if empty
  if (contextFiles.length === 0 && mockFiles.length > 0) {
    setContextFiles(mockFiles);
  }

  const handleAddFile = () => {
    if (!newPath.trim()) return;

    const file: ContextFile = {
      id: Date.now().toString(),
      path: newPath.trim(),
      addedAt: new Date().toISOString(),
    };

    addContextFile(file);
    setNewPath('');
    setIsAdding(false);
  };

  const handleRemoveFile = (id: string) => {
    removeContextFile(id);
  };

  const formatPath = (path: string) => {
    const parts = path.split('/');
    return parts[parts.length - 1] || path;
  };

  const formatTime = (timestamp: string) => {
    return new Date(timestamp).toLocaleDateString(undefined, {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  return (
    <div className="panel">
      <div className="flex items-center justify-between mb-4 pb-2 border-b border-gray-200">
        <h2 className="text-lg font-semibold text-gray-800">Context Files</h2>
        <button
          onClick={() => setIsAdding(true)}
          className="text-sm text-primary-600 hover:text-primary-700 font-medium"
        >
          + Add
        </button>
      </div>

      {/* Add file dialog */}
      {isAdding && (
        <div className="mb-4 p-3 bg-gray-50 rounded-lg">
          <input
            type="text"
            value={newPath}
            onChange={(e) => setNewPath(e.target.value)}
            placeholder="/absolute/path/to/file"
            className="w-full px-3 py-2 border border-gray-300 rounded text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
            onKeyDown={(e) => e.key === 'Enter' && handleAddFile()}
          />
          <div className="mt-2 flex justify-end space-x-2">
            <button
              onClick={() => setIsAdding(false)}
              className="px-3 py-1 text-xs text-gray-600 hover:text-gray-800"
            >
              Cancel
            </button>
            <button
              onClick={handleAddFile}
              className="px-3 py-1 text-xs bg-primary-600 text-white rounded hover:bg-primary-700"
            >
              Add
            </button>
          </div>
        </div>
      )}

      {/* File list */}
      {contextFiles.length === 0 ? (
        <div className="text-center py-8 text-gray-400">
          <p className="text-sm">No files in context</p>
          <p className="text-xs mt-1">Add files to make them available to AI</p>
        </div>
      ) : (
        <div className="space-y-2 max-h-96 overflow-y-auto">
          {contextFiles.map((file) => (
            <div
              key={file.id}
              className="group flex items-center justify-between p-3 bg-gray-50 rounded-lg hover:bg-gray-100 transition-colors"
            >
              <div className="flex-1 min-w-0">
                <div className="flex items-center space-x-2">
                  <svg
                    className="w-4 h-4 text-gray-400 flex-shrink-0"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
                    />
                  </svg>
                  <span className="text-sm font-medium text-gray-900 truncate">
                    {formatPath(file.path)}
                  </span>
                  {file.lastReadAt && (
                    <span className="w-2 h-2 bg-green-400 rounded-full flex-shrink-0" title="Read by AI" />
                  )}
                </div>
                <p className="text-xs text-gray-400 mt-1 truncate pl-6">
                  {file.path}
                </p>
                <p className="text-xs text-gray-400 pl-6">
                  Added {formatTime(file.addedAt)}
                </p>
              </div>
              <button
                onClick={() => handleRemoveFile(file.id)}
                className="ml-2 p-1 text-gray-400 hover:text-red-600 opacity-0 group-hover:opacity-100 transition-opacity"
                title="Remove from context"
              >
                <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M6 18L18 6M6 6l12 12"
                  />
                </svg>
              </button>
            </div>
          ))}
        </div>
      )}

      {/* File count */}
      {contextFiles.length > 0 && (
        <div className="mt-4 pt-3 border-t border-gray-200 text-xs text-gray-500 text-center">
          {contextFiles.length} file{contextFiles.length !== 1 ? 's' : ''} in context
        </div>
      )}
    </div>
  );
}

export default ContextPanel;
