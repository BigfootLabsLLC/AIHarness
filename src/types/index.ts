/// Types for AIHarness MCP Server UI

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
  arguments: Record<string, unknown>;
  success: boolean;
  content: string;
  duration_ms: number;
}

/** Context file */
export interface ContextFile {
  id: string;
  path: string;
  contentHash?: string;
  addedAt: string;
  lastReadAt?: string;
}

/** Tool definition */
export interface ToolDefinition {
  name: string;
  description: string;
  inputSchema: Record<string, unknown>;
}
