/// Types for AIHarness HTTP Tool Server UI

/** Server status states */
export type ServerStatusState = 'stopped' | 'starting' | 'running' | 'error';

/** Server status from backend */
export interface ServerStatus {
  running: boolean;
  port?: number;
}

/** Tool call record */
export interface ToolCall {
  id: string;
  timestamp: string;
  tool_name: string;
  project_id: string;
  arguments: Record<string, unknown>;
  success: boolean;
  content: string;
  duration_ms: number;
}

/** Project metadata */
export interface ProjectInfo {
  id: string;
  name: string;
  root_path: string;
  db_path: string;
  created_at: string;
  updated_at: string;
}

/** Todo item */
export interface TodoItem {
  id: string;
  title: string;
  description?: string;
  completed: boolean;
  position: number;
  created_at: string;
  updated_at: string;
}

/** Context file */
export interface ContextFile {
  id: string;
  path: string;
  contentHash?: string;
  addedAt: string;
  lastReadAt?: string;
}

/** Context note */
export interface ContextNote {
  id: string;
  content: string;
  position: number;
  created_at: string;
  updated_at: string;
}

/** Build command */
export interface BuildCommand {
  id: string;
  name: string;
  command: string;
  working_dir?: string | null;
  is_default: boolean;
  created_at: string;
}

/** Directory entry */
export interface DirectoryEntry {
  name: string;
  path: string;
  is_dir: boolean;
}

/** Directory listing */
export interface DirectoryListing {
  path: string;
  parent_path?: string | null;
  entries: DirectoryEntry[];
}

/** Tool definition */
export interface ToolDefinition {
  name: string;
  description: string;
  inputSchema: Record<string, unknown>;
}

/** Raw log entry for debugging */
export interface RawLog {
  timestamp: string;
  source: string;
  message: string;
}
