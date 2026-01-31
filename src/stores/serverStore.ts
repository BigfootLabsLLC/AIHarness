import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { ServerStatusState, ServerStatus, ToolCall, ContextFile } from '../types';

interface ServerState {
  // Server status
  status: ServerStatusState;
  port: number | null;
  error: string | null;
  
  // Data
  toolCalls: ToolCall[];
  contextFiles: ContextFile[];
  isLoading: boolean;
  
  // Actions
  initialize: () => Promise<void>;
  startServer: () => Promise<void>;
  stopServer: () => Promise<void>;
  refreshStatus: () => Promise<void>;
  loadContextFiles: () => Promise<void>;
  addContextFile: (path: string) => Promise<void>;
  removeContextFile: (id: string, path: string) => Promise<void>;
  addToolCall: (call: ToolCall) => void;
  clearToolCalls: () => void;
}

export const useServerStore = create<ServerState>((set, get) => ({
  // Initial state
  status: 'stopped',
  port: null,
  error: null,
  toolCalls: [],
  contextFiles: [],
  isLoading: false,
  
  // Initialize - check server status on load
  initialize: async () => {
    try {
      const status = await invoke<ServerStatus>('get_server_status');
      set({ 
        status: status.running ? 'running' : 'stopped',
        port: status.port 
      });
      
      // Load context files
      await get().loadContextFiles();
      
      // Listen for tool call events from backend
      listen<ToolCall>('tool-call', (event) => {
        get().addToolCall(event.payload);
      }).catch(console.error);
      
    } catch (error) {
      console.error('Failed to initialize:', error);
      set({ error: String(error) });
    }
  },
  
  // Start the MCP server
  startServer: async () => {
    set({ status: 'starting', error: null });
    
    try {
      const status = await invoke<ServerStatus>('start_server');
      set({ 
        status: status.running ? 'running' : 'error',
        port: status.port,
        error: null
      });
    } catch (error) {
      console.error('Failed to start server:', error);
      set({ 
        status: 'error',
        error: String(error)
      });
    }
  },
  
  // Stop the MCP server
  stopServer: async () => {
    try {
      const status = await invoke<ServerStatus>('stop_server');
      set({ 
        status: status.running ? 'running' : 'stopped',
        port: null
      });
    } catch (error) {
      console.error('Failed to stop server:', error);
      set({ error: String(error) });
    }
  },
  
  // Refresh server status
  refreshStatus: async () => {
    try {
      const status = await invoke<ServerStatus>('get_server_status');
      set({ 
        status: status.running ? 'running' : 'stopped',
        port: status.port
      });
    } catch (error) {
      console.error('Failed to get status:', error);
    }
  },
  
  // Load context files from backend
  loadContextFiles: async () => {
    try {
      const files = await invoke<Array<{id: string, path: string, name: string, added_at: string, last_read_at?: string}>>('list_context_files');
      
      const contextFiles: ContextFile[] = files.map(f => ({
        id: f.id,
        path: f.path,
        contentHash: undefined,
        addedAt: f.added_at,
        lastReadAt: f.last_read_at,
      }));
      
      set({ contextFiles });
    } catch (error) {
      console.error('Failed to load context files:', error);
    }
  },
  
  // Add a context file
  addContextFile: async (path: string) => {
    try {
      const file = await invoke<{id: string, path: string, name: string, added_at: string, last_read_at?: string}>('add_context_file', { path });
      
      const newFile: ContextFile = {
        id: file.id,
        path: file.path,
        contentHash: undefined,
        addedAt: file.added_at,
        lastReadAt: file.last_read_at,
      };
      
      set((state) => ({
        contextFiles: [...state.contextFiles, newFile]
      }));
    } catch (error) {
      console.error('Failed to add context file:', error);
      alert(`Failed to add file: ${error}`);
    }
  },
  
  // Remove a context file
  removeContextFile: async (id: string, path: string) => {
    try {
      await invoke('remove_context_file', { path });
      
      set((state) => ({
        contextFiles: state.contextFiles.filter((f) => f.id !== id)
      }));
    } catch (error) {
      console.error('Failed to remove context file:', error);
      alert(`Failed to remove file: ${error}`);
    }
  },
  
  addToolCall: (call: ToolCall) => {
    set((state) => ({
      toolCalls: [call, ...state.toolCalls].slice(0, 100)
    }));
  },
  
  clearToolCalls: () => set({ toolCalls: [] }),
}));
