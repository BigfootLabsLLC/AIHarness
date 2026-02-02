import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { ServerStatusState, ServerStatus, ToolCall, ContextFile, RawLog, TodoItem, ProjectInfo, DirectoryListing, ContextNote, BuildCommand, McpToolInfo, McpConfigResult } from '../types';

interface ServerState {
  // Server status
  status: ServerStatusState;
  port: number;
  error: string | null;
  
  // Current project (for filtering events)
  currentProjectId: string | null;
  
  // Data - keyed by project ID where applicable
  toolCalls: ToolCall[];
  contextFiles: ContextFile[];
  contextNotes: ContextNote[];
  rawLogs: RawLog[];
  todosByProject: Map<string, TodoItem[]>; // Keyed by project ID
  buildCommandsByProject: Map<string, BuildCommand[]>; // Keyed by project ID
  projects: ProjectInfo[];
  isLoading: boolean;
  
  // Getters for current project's data
  getTodos: (projectId: string) => TodoItem[];
  getBuildCommands: (projectId: string) => BuildCommand[];
  
  // Actions
  initialize: () => Promise<void>;
  setCurrentProject: (projectId: string) => void;
  loadToolHistory: (projectId: string) => Promise<void>;
  startServer: () => Promise<void>;
  stopServer: () => Promise<void>;
  refreshStatus: () => Promise<void>;
  executeTool: (name: string, args: Record<string, unknown>, projectId?: string) => Promise<string>;
  loadContextFiles: () => Promise<void>;
  loadContextFilesForProject: (projectId: string) => Promise<void>;
  loadContextNotes: (projectId: string) => Promise<void>;
  addContextNote: (projectId: string, content: string, position?: number) => Promise<void>;
  updateContextNote: (projectId: string, id: string, content: string) => Promise<void>;
  removeContextNote: (projectId: string, id: string) => Promise<void>;
  moveContextNote: (projectId: string, id: string, position: number) => Promise<void>;
  addContextFile: (projectId: string, path: string) => Promise<void>;
  removeContextFile: (projectId: string, id: string, path: string) => Promise<void>;
  listProjects: () => Promise<void>;
  createProject: (name: string, rootPath: string) => Promise<{ project: ProjectInfo | null; error?: string }>;
  listProjectDirectory: (projectId: string, subPath?: string) => Promise<DirectoryListing | null>;
  listDirectory: (path: string) => Promise<DirectoryListing | null>;
  getHomeDirectory: () => Promise<string | null>;
  loadBuildCommands: (projectId: string) => Promise<void>;
  addBuildCommand: (projectId: string, name: string, command: string, workingDir?: string) => Promise<void>;
  removeBuildCommand: (projectId: string, id: string) => Promise<void>;
  runBuildCommand: (projectId: string, id: string) => Promise<string | null>;
  setDefaultBuildCommand: (projectId: string, id: string) => Promise<void>;
  getDefaultBuildCommand: (projectId: string) => Promise<BuildCommand | null>;
  loadTodos: (projectId: string) => Promise<void>;
  addTodo: (projectId: string, title: string, description?: string) => Promise<void>;
  setTodoCompleted: (projectId: string, id: string, completed: boolean) => Promise<void>;
  removeTodo: (projectId: string, id: string) => Promise<void>;
  resetProjectData: () => void;
  clearToolCalls: () => void;
  clearRawLogs: () => void;
  addRawLog: (log: RawLog) => void;
  // MCP Config
  getMcpSupportedTools: () => Promise<McpToolInfo[]>;
  configureMcpForTool: (tool: string, projectId: string) => Promise<McpConfigResult>;
  configureMcpForAllTools: (projectId: string) => Promise<McpConfigResult[]>;
}

export const useServerStore = create<ServerState>((set, get) => ({
  // Initial state
  status: 'stopped',
  port: 8787,
  error: null,
  currentProjectId: null,
  toolCalls: [],
  contextFiles: [],
  contextNotes: [],
  rawLogs: [],
  todosByProject: new Map(),
  buildCommandsByProject: new Map(),
  projects: [],
  isLoading: false,
  
  // Getters
  getTodos: (projectId: string) => get().todosByProject.get(projectId) ?? [],
  getBuildCommands: (projectId: string) => get().buildCommandsByProject.get(projectId) ?? [],
  
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
        set((state) => {
          // Only add if it matches the current project
          if (state.currentProjectId && event.payload.project_id !== state.currentProjectId) {
            return state; // Skip - different project
          }
          return {
            toolCalls: [event.payload, ...state.toolCalls].slice(0, 100)
          };
        });
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
  executeTool: async (name: string, args: Record<string, unknown>, projectId?: string) => {
    const result = await invoke<string>('execute_tool', {
      tool_name: name,
      arguments: args,
      project_id: projectId ?? null,
    });
    return result;
  },

  setCurrentProject: (projectId: string) => {
    set({ currentProjectId: projectId });
  },

  loadToolHistory: async (projectId: string) => {
    try {
      const history = await invoke<ToolCall[]>('get_event_history', { project_id: projectId });
      set({ toolCalls: history, currentProjectId: projectId });
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

  loadContextNotes: async (projectId: string) => {
    try {
      const notes = await invoke<ContextNote[]>('list_context_notes', { project_id: projectId });
      const sorted = [...notes].sort((a, b) => a.position - b.position);
      set({ contextNotes: sorted });
    } catch (error) {
      console.error('Failed to load context notes:', error);
    }
  },

  loadBuildCommands: async (projectId: string) => {
    try {
      const commands = await invoke<BuildCommand[]>('list_build_commands', { project_id: projectId });
      set((state) => {
        const newMap = new Map(state.buildCommandsByProject);
        newMap.set(projectId, commands);
        return { buildCommandsByProject: newMap };
      });
    } catch (error) {
      console.error('Failed to load build commands:', error);
      set((state) => {
        const newMap = new Map(state.buildCommandsByProject);
        newMap.set(projectId, []);
        return { buildCommandsByProject: newMap };
      });
    }
  },

  addBuildCommand: async (projectId: string, name: string, command: string, workingDir?: string) => {
    try {
      await invoke<BuildCommand>('add_build_command', {
        project_id: projectId,
        name,
        command,
        working_dir: workingDir ?? null,
      });
      await get().loadBuildCommands(projectId);
    } catch (error) {
      console.error('Failed to add build command:', error);
    }
  },

  removeBuildCommand: async (projectId: string, id: string) => {
    try {
      await invoke('remove_build_command', { project_id: projectId, id });
      await get().loadBuildCommands(projectId);
    } catch (error) {
      console.error('Failed to remove build command:', error);
    }
  },

  runBuildCommand: async (projectId: string, id: string) => {
    try {
      const output = await invoke<string>('run_build_command', { project_id: projectId, id });
      return output;
    } catch (error) {
      console.error('Failed to run build command:', error);
      return null;
    }
  },

  setDefaultBuildCommand: async (projectId: string, id: string) => {
    try {
      await invoke('set_default_build_command', { project_id: projectId, id });
      await get().loadBuildCommands(projectId);
    } catch (error) {
      console.error('Failed to set default build command:', error);
    }
  },

  getDefaultBuildCommand: async (projectId: string) => {
    try {
      const command = await invoke<BuildCommand | null>('get_default_build_command', {
        project_id: projectId,
      });
      return command;
    } catch (error) {
      console.error('Failed to get default build command:', error);
      return null;
    }
  },

  addContextNote: async (projectId: string, content: string, position?: number) => {
    try {
      await invoke<ContextNote>('add_context_note', {
        project_id: projectId,
        content,
        position: position ?? null,
      });
      await get().loadContextNotes(projectId);
    } catch (error) {
      console.error('Failed to add context note:', error);
    }
  },

  updateContextNote: async (projectId: string, id: string, content: string) => {
    try {
      await invoke('update_context_note', { project_id: projectId, id, content });
      await get().loadContextNotes(projectId);
    } catch (error) {
      console.error('Failed to update context note:', error);
    }
  },

  removeContextNote: async (projectId: string, id: string) => {
    try {
      await invoke('remove_context_note', { project_id: projectId, id });
      await get().loadContextNotes(projectId);
    } catch (error) {
      console.error('Failed to remove context note:', error);
    }
  },

  moveContextNote: async (projectId: string, id: string, position: number) => {
    try {
      await invoke('move_context_note', { project_id: projectId, id, position });
      await get().loadContextNotes(projectId);
    } catch (error) {
      console.error('Failed to move context note:', error);
    }
  },
  
  // Add a context file
  addContextFile: async (projectId: string, path: string) => {
    try {
      const file = await invoke<{id: string, path: string, name: string, added_at: string, last_read_at?: string}>(
        'add_context_file',
        { project_id: projectId, path },
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
  removeContextFile: async (projectId: string, id: string, path: string) => {
    try {
      await invoke('remove_context_file', { project_id: projectId, path });
      
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
      const project = await invoke<ProjectInfo>('create_project', { name, rootPath });
      await get().listProjects();
      return { project };
    } catch (error) {
      console.error('Failed to create project:', error);
      return { project: null, error: String(error) };
    }
  },

  listProjectDirectory: async (projectId: string, subPath?: string) => {
    try {
      const listing = await invoke<DirectoryListing>('list_project_directory', {
        project_id: projectId,
        sub_path: subPath ?? null,
      });
      return listing;
    } catch (error) {
      console.error('Failed to list project directory:', error);
      return null;
    }
  },

  listDirectory: async (path: string) => {
    try {
      const listing = await invoke<DirectoryListing>('list_directory', { path });
      return listing;
    } catch (error) {
      console.error('Failed to list directory:', error);
      return null;
    }
  },

  getHomeDirectory: async () => {
    try {
      const home = await invoke<string>('get_home_directory');
      return home;
    } catch (error) {
      console.error('Failed to get home directory:', error);
      return null;
    }
  },

  loadTodos: async (projectId: string) => {
    try {
      console.log('[Store] Loading todos for project:', projectId);
      invoke('debug_log_cmd', { msg: `[Store] loadTodos START projectId=${projectId}` });
      const todos = await invoke<TodoItem[]>('list_todos', { project_id: projectId });
      invoke('debug_log_cmd', { msg: `[Store] loadTodos DONE projectId=${projectId} count=${todos.length}` });
      console.log('[Store] Loaded', todos.length, 'todos for project:', projectId);
      console.log('[Store] Loaded', todos.length, 'todos for project:', projectId);
      set((state) => {
        const newMap = new Map(state.todosByProject);
        newMap.set(projectId, todos);
        return { todosByProject: newMap };
      });
    } catch (error) {
      console.error('Failed to load todos:', error);
      set((state) => {
        const newMap = new Map(state.todosByProject);
        newMap.set(projectId, []);
        return { todosByProject: newMap };
      });
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

  resetProjectData: () => set({
    toolCalls: [],
    contextFiles: [],
    contextNotes: [],
    // Don't clear todosByProject or buildCommandsByProject - 
    // they contain data for all projects
  }),

  // MCP Config
  getMcpSupportedTools: async () => {
    try {
      const tools = await invoke<McpToolInfo[]>('get_mcp_supported_tools');
      return tools;
    } catch (error) {
      console.error('Failed to get MCP supported tools:', error);
      return [];
    }
  },

  configureMcpForTool: async (tool: string, projectId: string) => {
    try {
      const port = get().port;
      const result = await invoke<McpConfigResult>('configure_mcp_for_tool', {
        tool,
        project_id: projectId,
        port,
      });
      return result;
    } catch (error) {
      console.error('Failed to configure MCP:', error);
      return {
        success: false,
        message: String(error),
        config_path: null,
      };
    }
  },

  configureMcpForAllTools: async (projectId: string) => {
    try {
      const port = get().port;
      const results = await invoke<McpConfigResult[]>('configure_mcp_for_all_tools', {
        project_id: projectId,
        port,
      });
      return results;
    } catch (error) {
      console.error('Failed to configure MCP for all tools:', error);
      return [{
        success: false,
        message: String(error),
        config_path: null,
      }];
    }
  },
}));
