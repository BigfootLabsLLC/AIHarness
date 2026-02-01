import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { ServerStatusState, ServerStatus, ToolCall, ContextFile, RawLog, TodoItem, ProjectInfo } from '../types';

interface ServerState {
  // Server status
  status: ServerStatusState;
  port: number;
  error: string | null;
  
  // Data
  toolCalls: ToolCall[];
  contextFiles: ContextFile[];
  rawLogs: RawLog[];
  todos: TodoItem[];
  projects: ProjectInfo[];
  isLoading: boolean;
  
  // Actions
  initialize: () => Promise<void>;
  loadToolHistory: (projectId: string) => Promise<void>;
  startServer: () => Promise<void>;
  stopServer: () => Promise<void>;
  refreshStatus: () => Promise<void>;
  executeTool: (name: string, args: Record<string, unknown>) => Promise<string>;
  loadContextFiles: () => Promise<void>;
  loadContextFilesForProject: (projectId: string) => Promise<void>;
  addContextFile: (path: string) => Promise<void>;
  removeContextFile: (id: string, path: string) => Promise<void>;
  listProjects: () => Promise<void>;
  createProject: (name: string, rootPath: string) => Promise<ProjectInfo | null>;
  loadTodos: (projectId: string) => Promise<void>;
  addTodo: (projectId: string, title: string, description?: string) => Promise<void>;
  setTodoCompleted: (projectId: string, id: string, completed: boolean) => Promise<void>;
  removeTodo: (projectId: string, id: string) => Promise<void>;
  clearToolCalls: () => void;
  clearRawLogs: () => void;
  addRawLog: (log: RawLog) => void;
}

export const useServerStore = create<ServerState>((set, get) => ({
  // Initial state
  status: 'stopped',
  port: 8787,
  error: null,
  toolCalls: [],
  contextFiles: [],
  rawLogs: [],
  todos: [],
  projects: [],
  isLoading: false,
  
  // Initialize - check server status and subscribe to events
  initialize: async () => {
    try {
      // Get current status
      const status = await invoke<ServerStatus>('get_server_status');
      set({ 
        status: status.running ? 'running' : 'stopped',
        port: status.port 
      });
      
      // Load existing events
      const history = await invoke<ToolCall[]>('get_event_history');
      set({ toolCalls: history });
      
      // Load context files
      await get().loadContextFiles();
      await get().listProjects();
      
      // Subscribe to tool-call events from backend
      listen<ToolCall>('tool-call', (event) => {
        console.log('[ToolCall] Received:', event.payload);
        set((state) => ({
          toolCalls: [event.payload, ...state.toolCalls].slice(0, 100)
        }));
      }).catch(console.error);
      
      // Subscribe to raw-log events from backend
      listen<RawLog>('raw-log', (event) => {
        console.log('[RawLog] Received:', event.payload);
        set((state) => ({
          rawLogs: [event.payload, ...state.rawLogs].slice(0, 500)
        }));
      }).catch(console.error);

      // Auto-start happens in the backend; refresh status after a short delay.
      const refreshLater = (delayMs: number) => {
        setTimeout(() => {
          get().refreshStatus().catch(console.error);
        }, delayMs);
      };
      refreshLater(500);
      refreshLater(2000);
      
    } catch (error) {
      console.error('Failed to initialize:', error);
      set({ error: String(error) });
    }
  },
  
  // Start the HTTP server
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
  
  // Stop the HTTP server
  stopServer: async () => {
    try {
      await invoke<ServerStatus>('stop_server');
      set({ 
        status: 'stopped',
        port: 8787
      });
    } catch (error) {
      console.error('Failed to stop server:', error);
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
  
  // Execute a tool directly
  executeTool: async (name: string, args: Record<string, unknown>) => {
    const result = await invoke<string>('execute_tool', {
      tool_name: name,
      arguments: args
    });
    return result;
  },

  loadToolHistory: async (projectId: string) => {
    try {
      const history = await invoke<ToolCall[]>('get_event_history', { project_id: projectId });
      set({ toolCalls: history });
    } catch (error) {
      console.error('Failed to load tool history:', error);
    }
  },
  
  // Load context files from backend
  loadContextFiles: async () => {
    try {
      const files = await invoke<Array<{id: string, path: string, name: string, added_at: string, last_read_at?: string}>>('list_context_files');
      
      const contextFiles: ContextFile[] = files.map(f => ({
        id: f.id,
        path: f.path,
        addedAt: f.added_at,
        lastReadAt: f.last_read_at,
      }));
      
      set({ contextFiles });
    } catch (error) {
      console.error('Failed to load context files:', error);
    }
  },

  loadContextFilesForProject: async (projectId: string) => {
    try {
      const files = await invoke<Array<{id: string, path: string, name: string, added_at: string, last_read_at?: string}>>(
        'list_context_files',
        { project_id: projectId },
      );

      const contextFiles: ContextFile[] = files.map(f => ({
        id: f.id,
        path: f.path,
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
      const file = await invoke<{id: string, path: string, name: string, added_at: string, last_read_at?: string}>(
        'add_context_file',
        { path },
      );
      
      const newFile: ContextFile = {
        id: file.id,
        path: file.path,
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
  
  clearToolCalls: () => set({ toolCalls: [] }),
  clearRawLogs: () => set({ rawLogs: [] }),
  addRawLog: (log: RawLog) => set((state) => ({
    rawLogs: [log, ...state.rawLogs].slice(0, 500)
  })),

  listProjects: async () => {
    try {
      const projects = await invoke<ProjectInfo[]>('list_projects');
      set({ projects });
    } catch (error) {
      console.error('Failed to list projects:', error);
    }
  },

  createProject: async (name: string, rootPath: string) => {
    try {
      const project = await invoke<ProjectInfo>('create_project', { name, root_path: rootPath });
      await get().listProjects();
      return project;
    } catch (error) {
      console.error('Failed to create project:', error);
      return null;
    }
  },

  loadTodos: async (projectId: string) => {
    try {
      const todos = await invoke<TodoItem[]>('list_todos', { project_id: projectId });
      set({ todos });
    } catch (error) {
      console.error('Failed to load todos:', error);
    }
  },

  addTodo: async (projectId: string, title: string, description?: string) => {
    try {
      await invoke<TodoItem>('add_todo', { project_id: projectId, title, description });
      await get().loadTodos(projectId);
    } catch (error) {
      console.error('Failed to add todo:', error);
    }
  },

  setTodoCompleted: async (projectId: string, id: string, completed: boolean) => {
    try {
      await invoke('set_todo_completed', { project_id: projectId, id, completed });
      await get().loadTodos(projectId);
    } catch (error) {
      console.error('Failed to update todo:', error);
    }
  },

  removeTodo: async (projectId: string, id: string) => {
    try {
      await invoke('remove_todo', { project_id: projectId, id });
      await get().loadTodos(projectId);
    } catch (error) {
      console.error('Failed to remove todo:', error);
    }
  },
}));
