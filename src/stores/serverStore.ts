import { create } from 'zustand';
import { ServerStatus, ToolCall, ContextFile } from '../types';

interface ServerState {
  // Server status
  status: ServerStatus;
  port: number | null;
  error: string | null;
  
  // Data
  toolCalls: ToolCall[];
  contextFiles: ContextFile[];
  
  // Actions
  setStatus: (status: ServerStatus) => void;
  setPort: (port: number | null) => void;
  setError: (error: string | null) => void;
  addToolCall: (call: ToolCall) => void;
  clearToolCalls: () => void;
  setContextFiles: (files: ContextFile[]) => void;
  addContextFile: (file: ContextFile) => void;
  removeContextFile: (id: string) => void;
}

export const useServerStore = create<ServerState>((set) => ({
  // Initial state
  status: 'stopped',
  port: null,
  error: null,
  toolCalls: [],
  contextFiles: [],
  
  // Actions
  setStatus: (status) => set({ status }),
  setPort: (port) => set({ port }),
  setError: (error) => set({ error }),
  
  addToolCall: (call) => set((state) => ({
    toolCalls: [call, ...state.toolCalls].slice(0, 100), // Keep last 100
  })),
  
  clearToolCalls: () => set({ toolCalls: [] }),
  
  setContextFiles: (files) => set({ contextFiles: files }),
  
  addContextFile: (file) => set((state) => ({
    contextFiles: [...state.contextFiles, file],
  })),
  
  removeContextFile: (id) => set((state) => ({
    contextFiles: state.contextFiles.filter((f) => f.id !== id),
  })),
}));
