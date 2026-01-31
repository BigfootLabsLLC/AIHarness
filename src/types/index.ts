/// Types for AIHarness MCP Server UI

/** Server status */
export type ServerStatus = 'stopped' | 'starting' | 'running' | 'error';

/** Tool call record */
export interface ToolCall {
  id: string;
  timestamp: string;
  toolName: string;
  arguments: Record<string, unknown>;
  success: boolean;
  content: string;
  durationMs?: number;
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
