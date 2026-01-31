/**
 * Core Type Definitions for AIHarness
 * 
 * This file defines all data models for the multi-agent AI orchestration system.
 */

// ============================================================================
// Base Types
// ============================================================================

export type ID = string;

export interface Timestamped {
  createdAt: Date;
  updatedAt: Date;
}

// ============================================================================
// Provider & Model Types
// ============================================================================

export type ProviderName = 'openai' | 'anthropic' | 'ollama' | 'google' | 'xai';

export interface ModelInfo {
  id: string;
  name: string;
  provider: ProviderName;
  costPer1KInput: number;   // USD
  costPer1KOutput: number;  // USD
  maxTokens: number;
  tier: 'cheap' | 'medium' | 'premium';
  capabilities: ModelCapability[];
}

export type ModelCapability = 
  | 'code' 
  | 'reasoning' 
  | 'creative' 
  | 'analysis' 
  | 'long_context';

export interface ProviderConfig {
  name: ProviderName;
  apiKey?: string;
  baseUrl?: string;         // For custom/Ollama endpoints
  enabled: boolean;
  defaultModel: string;
}

// ============================================================================
// LLM Interaction Types
// ============================================================================

export interface Message {
  role: 'system' | 'user' | 'assistant';
  content: string;
  timestamp?: Date;
}

export interface LLMRequest {
  messages: Message[];
  model?: string;
  temperature?: number;
  maxTokens?: number;
  stream?: boolean;
}

export interface LLMResponse {
  content: string;
  model: string;
  provider: ProviderName;
  tokensIn: number;
  tokensOut: number;
  costUSD: number;
  latencyMs: number;
  finishReason: 'stop' | 'length' | 'error';
}

export interface LLMStreamChunk {
  content: string;
  isComplete: boolean;
}

// ============================================================================
// Project & Document Types
// ============================================================================

export interface Project extends Timestamped {
  id: ID;
  name: string;
  description: string;
  rootPath: string;
  
  // Budget
  budgetUSD?: number;
  budgetAlertThresholds: number[];  // e.g., [0.5, 0.8, 0.95]
  
  // Configuration
  defaultAgentRoles: AgentRoleConfig[];
  routingRules: RoutingRule[];
  
  // Metadata
  tags: string[];
  isArchived: boolean;
}

export interface Document extends Timestamped {
  id: ID;
  projectId: ID;
  name: string;
  path: string;             // Relative to project root
  content: string;
  
  // Front matter metadata
  metadata: DocumentMetadata;
  
  // Versioning
  version: number;
  history: DocumentVersion[];
}

export interface DocumentMetadata {
  title?: string;
  description?: string;
  tags: string[];
  status: 'draft' | 'in_review' | 'approved' | 'deprecated';
  author?: string;
  usedBy: ID[];             // Agent/task IDs that reference this
  relatedDocs: ID[];
}

export interface DocumentVersion {
  version: number;
  content: string;
  timestamp: Date;
  changeSummary?: string;
}

// ============================================================================
// Prompt Library Types
// ============================================================================

export interface Prompt extends Timestamped {
  id: ID;
  name: string;
  description: string;
  category: string;
  
  // Template
  template: string;
  variables: PromptVariable[];
  
  // Usage
  usageCount: number;
  successRate?: number;     // User feedback
  avgCost?: number;
  
  // Versioning
  version: number;
  isArchived: boolean;
}

export interface PromptVariable {
  name: string;
  description: string;
  type: 'string' | 'number' | 'boolean' | 'document' | 'code_block';
  required: boolean;
  defaultValue?: string;
}

// ============================================================================
// Agent Types
// ============================================================================

export type AgentRole = 
  | 'architect'      // High-level design, planning
  | 'implementer'    // Code implementation
  | 'reviewer'       // Code review
  | 'tester'         // Test generation
  | 'debugger'       // Bug fixing
  | 'documenter';    // Documentation

export type AgentStatus = 
  | 'idle'
  | 'initializing'
  | 'working'
  | 'awaiting_input'
  | 'awaiting_review'
  | 'paused'
  | 'error'
  | 'terminated';

export interface Agent extends Timestamped {
  id: ID;
  projectId: ID;
  
  // Identity
  name: string;
  role: AgentRole;
  description?: string;
  
  // Model configuration
  provider: ProviderName;
  model: string;
  temperature: number;
  maxTokens: number;
  
  // System prompt
  systemPrompt: string;
  
  // State
  status: AgentStatus;
  statusMessage?: string;   // Human-readable status
  
  // Context
  conversationHistory: Message[];
  contextDocuments: ID[];   // Documents in context
  maxContextTokens: number;
  
  // Current work
  currentTaskId?: ID;
  
  // Cost tracking
  totalCostUSD: number;
  costBudgetUSD?: number;
  costAlertSent: boolean;
  
  // Performance
  tasksCompleted: number;
  tasksRejected: number;
  avgLatencyMs: number;
}

export interface AgentRoleConfig {
  role: AgentRole;
  name: string;
  description: string;
  defaultProvider: ProviderName;
  defaultModel: string;
  systemPromptTemplate: string;
  costBudgetUSD?: number;
  capabilities: string[];
}

// ============================================================================
// Task Types
// ============================================================================

export type TaskType = 
  | 'architecture'
  | 'specification'
  | 'interface_design'
  | 'implementation'
  | 'refactoring'
  | 'testing'
  | 'review'
  | 'documentation'
  | 'research';

export type TaskStatus = 
  | 'pending'
  | 'blocked'
  | 'in_progress'
  | 'awaiting_review'
  | 'changes_requested'
  | 'approved'
  | 'completed'
  | 'cancelled'
  | 'failed';

export type TaskPriority = 'low' | 'medium' | 'high' | 'critical';

export interface Task extends Timestamped {
  id: ID;
  projectId: ID;
  parentTaskId?: ID;        // For subtasks
  
  // Basic info
  title: string;
  description: string;
  type: TaskType;
  priority: TaskPriority;
  
  // Assignment
  assignedTo?: ID;          // Agent ID
  reviewedBy?: ID;          // Agent ID (for review tasks)
  
  // Workflow
  status: TaskStatus;
  statusHistory: StatusChange[];
  
  // Dependencies
  dependencies: ID[];       // Must complete before this
  blocks: ID[];             // Cannot start until this completes
  
  // Inputs/Outputs
  inputDocumentIds: ID[];
  outputDocumentIds: ID[];
  
  // Cost
  costBudgetUSD?: number;
  costSpentUSD: number;
  
  // Review
  reviewFeedback?: ReviewFeedback[];
  
  // Metadata
  tags: string[];
  estimatedEffort?: string; // e.g., "2 hours"
  dueDate?: Date;
}

export interface StatusChange {
  from: TaskStatus;
  to: TaskStatus;
  timestamp: Date;
  reason?: string;
  triggeredBy?: 'user' | 'agent' | 'system';
}

// ============================================================================
// Review Types
// ============================================================================

export interface ReviewFeedback {
  id: ID;
  taskId: ID;
  reviewerAgentId: ID;
  timestamp: Date;
  
  // Content
  summary: string;
  findings: Finding[];
  
  // Verdict
  verdict: ReviewVerdict;
  confidence: number;       // 0-1
  
  // Response
  response?: string;        // From implementer
  isResolved: boolean;
}

export type ReviewVerdict = 
  | 'approve'
  | 'approve_with_nits'
  | 'request_changes'
  | 'reject'
  | 'needs_discussion';

export interface Finding {
  id: ID;
  severity: FindingSeverity;
  category: FindingCategory;
  location?: CodeLocation;
  description: string;
  suggestion?: string;
  codeSnippet?: string;
}

export type FindingSeverity = 'critical' | 'major' | 'minor' | 'nit' | 'praise';

export type FindingCategory = 
  | 'correctness'
  | 'performance'
  | 'security'
  | 'maintainability'
  | 'readability'
  | 'testing'
  | 'documentation'
  | 'architecture'
  | 'style';

export interface CodeLocation {
  filePath: string;
  lineStart?: number;
  lineEnd?: number;
  columnStart?: number;
  columnEnd?: number;
}

// ============================================================================
// Cost Tracking Types
// ============================================================================

export interface CostLogEntry extends Timestamped {
  id: ID;
  projectId: ID;
  
  // What
  provider: ProviderName;
  model: string;
  operation: string;        // e.g., 'completion', 'embedding'
  
  // Usage
  tokensIn: number;
  tokensOut: number;
  costUSD: number;
  
  // Context
  agentId?: ID;
  taskId?: ID;
  promptId?: ID;
  
  // Metadata
  latencyMs: number;
  success: boolean;
  errorMessage?: string;
}

export interface BudgetAlert {
  id: ID;
  projectId: ID;
  timestamp: Date;
  level: 'warning' | 'critical' | 'exceeded';
  threshold: number;        // 0.5, 0.8, 0.95, 1.0
  currentSpendUSD: number;
  budgetUSD: number;
  acknowledged: boolean;
}

// ============================================================================
// Routing Types
// ============================================================================

export interface RoutingRule {
  id: ID;
  name: string;
  projectId: ID;
  
  // Match criteria
  taskType?: TaskType;
  complexity?: 'low' | 'medium' | 'high';
  minQualityScore?: number;
  
  // Constraints
  maxCostPer1KTokens?: number;
  preferredProviders?: ProviderName[];
  excludedModels?: string[];
  
  // Strategy
  strategy: RoutingStrategy;
  fallbackChain: string[];  // Ordered list of model IDs
  
  enabled: boolean;
}

export type RoutingStrategy = 
  | 'cheapest'      // Always pick cheapest
  | 'quality'       // Best quality within budget
  | 'balanced'      // Balance cost and quality
  | 'fixed'         // Use specified model
  | 'adaptive';     // Learn from past performance

// ============================================================================
// Workflow Types
// ============================================================================

export interface Workflow {
  id: ID;
  name: string;
  description: string;
  projectId: ID;
  
  stages: WorkflowStage[];
  enabled: boolean;
}

export interface WorkflowStage {
  id: ID;
  name: string;
  description: string;
  order: number;
  
  // Assignment
  agentRole: AgentRole;
  
  // Gate
  gateType: GateType;
  gateConfig?: GateConfig;
  
  // Inputs/Outputs
  inputDocTypes: string[];
  outputDocTypes: string[];
}

export type GateType = 'auto' | 'approval' | 'review' | 'condition';

export interface GateConfig {
  approver?: 'user' | AgentRole;
  reviewAgentRole?: AgentRole;
  condition?: string;       // e.g., "cost < 0.50"
  timeoutMinutes?: number;
}

// ============================================================================
// UI State Types
// ============================================================================

export interface WorkspaceState {
  activeProjectId?: ID;
  openDocuments: ID[];
  activeDocumentId?: ID;
  
  // Panels
  leftPanelOpen: boolean;
  rightPanelOpen: boolean;
  bottomPanelOpen: boolean;
  
  // View state
  selectedView: 'documents' | 'agents' | 'tasks' | 'costs' | 'prompts';
  agentFilter?: AgentStatus;
  taskFilter?: TaskStatus;
}

// ============================================================================
// Expert Panel Types
// ============================================================================

export type PanelMode = 'poll' | 'debate' | 'synthesis';

export type PanelStatus = 
  | 'configuring'
  | 'queuing'
  | 'running'
  | 'awaiting_responses'
  | 'analyzing'
  | 'completed'
  | 'cancelled';

export interface ExpertPanel extends Timestamped {
  id: ID;
  projectId: ID;
  
  // Configuration
  name: string;
  description?: string;
  mode: PanelMode;
  
  // Participants
  participants: PanelParticipant[];
  
  // The question/prompt
  prompt: string;
  contextDocumentIds?: ID[];
  
  // Settings
  maxRounds?: number;       // For debate mode
  anonymousResponses: boolean;
  showCostComparison: boolean;
  
  // State
  status: PanelStatus;
  currentRound: number;
  
  // Results
  responses: PanelResponse[];
  consensusReport?: ConsensusReport;
  debateTranscript?: DebateRound[];
  
  // Cost
  totalCostUSD: number;
  budgetUSD?: number;
}

export interface PanelParticipant {
  id: ID;
  modelId: string;          // e.g., "claude-3-5-sonnet"
  provider: ProviderName;
  role?: string;            // e.g., "Skeptic", "Optimist", "Expert"
  weight: number;           // For weighted consensus (default: 1)
  systemPrompt?: string;    // Override for this panel
  isActive: boolean;
}

export interface PanelResponse {
  id: ID;
  panelId: ID;
  participantId: ID;
  
  // Content
  content: string;
  rawResponse: string;
  
  // Metadata
  round: number;
  timestamp: Date;
  latencyMs: number;
  
  // Cost
  costUSD: number;
  tokensIn: number;
  tokensOut: number;
  
  // Analysis
  sentiment?: 'positive' | 'neutral' | 'negative';
  keyPoints?: string[];
}

export interface ConsensusReport {
  generatedAt: Date;
  
  // Agreement metrics
  agreementScore: number;   // 0-1, how much models agree
  disagreementAreas: DisagreementArea[];
  
  // Synthesis
  synthesizedAnswer?: string;
  keyConsensusPoints: string[];
  contestedPoints: string[];
  
  // Model contributions
  contributionBreakdown: ModelContribution[];
}

export interface DisagreementArea {
  topic: string;
  severity: 'minor' | 'moderate' | 'significant';
  positions: ModelPosition[];
}

export interface ModelPosition {
  participantId: ID;
  modelId: string;
  position: string;
  confidence: number;
}

export interface ModelContribution {
  participantId: ID;
  modelId: string;
  uniqueInsights: string[];
  consensusSupport: string[];
}

export interface DebateRound {
  roundNumber: number;
  startedAt: Date;
  completedAt?: Date;
  
  // For this round
  promptContext?: string;   // e.g., "Respond to the criticisms..."
  responses: PanelResponse[];
  
  // Analysis
  keyDisputes: string[];
  emergingConsensus: string[];
}

export interface PanelTemplate {
  id: ID;
  name: string;
  description: string;
  
  // Pre-configured settings
  mode: PanelMode;
  defaultParticipants: PanelParticipant[];
  systemPromptTemplate?: string;
  
  // Metadata
  category: string;
  tags: string[];
  usageCount: number;
}

// ============================================================================
// Chat & Conversation Types ⭐ NEW
// ============================================================================

export type ConversationMode = 'chat' | 'agent' | 'panel';

export interface Conversation extends Timestamped {
  id: ID;
  projectId: ID;
  
  // Identity
  title: string;
  mode: ConversationMode;
  
  // Tree structure (not linear!)
  rootMessageId?: ID;
  currentBranchId: ID;      // Where user is currently viewing
  
  // Branches (forks)
  branches: ConversationBranch[];
  
  // Metadata
  tags: string[];
  isArchived: boolean;
  
  // Stats
  messageCount: number;
  totalCostUSD: number;
}

export interface ConversationBranch {
  id: ID;
  name: string;             // User-defined or auto-generated
  parentBranchId?: ID;      // Null for root
  forkedFromMessageId?: ID; // Where this branch diverged
  headMessageId: ID;        // Latest message in this branch
  createdAt: Date;
}

export interface Message extends Timestamped {
  id: ID;
  conversationId: ID;
  branchId: ID;
  
  // Tree structure
  parentId?: ID;            // Null for root
  childrenIds: ID[];
  
  // Content
  role: 'system' | 'user' | 'assistant' | 'tool';
  content: string;
  
  // Tool use (for assistant messages)
  toolCalls?: ToolCall[];
  
  // Tool response (for tool messages)
  toolCallId?: string;
  toolResult?: ToolResult;
  
  // Metadata
  model?: string;           // Which model generated this
  costUSD?: number;
  tokensIn?: number;
  tokensOut?: number;
  latencyMs?: number;
  
  // Editing
  editedAt?: Date;
  originalContent?: string; // For tracking edits
  
  // User feedback
  feedback?: 'helpful' | 'not_helpful' | null;
}

export interface ToolCall {
  id: string;
  type: 'function';
  function: {
    name: string;
    arguments: string;      // JSON string
  };
}

export interface ToolResult {
  content: string;
  isError: boolean;
  exitCode?: number;        // For shell commands
}

export type ToolName = 
  | 'shell'                 // Execute shell command
  | 'read_file'             // Read file contents
  | 'write_file'            // Write/edit file
  | 'list_directory'        // List files
  | 'search_files'          // Grep/search
  | 'git_status'            // Git status
  | 'git_diff'              // Git diff
  | 'git_commit'            // Git commit
  | 'web_search'            // Web search
  | 'web_fetch';            // Fetch URL

export interface ToolDefinition {
  name: ToolName;
  description: string;
  parameters: object;       // JSON Schema
  requireApproval: boolean;
}

// ============================================================================
// Scheduling Types ⭐ NEW
// ============================================================================

export type ScheduleType = 'once' | 'recurring';
export type ScheduleStatus = 'active' | 'paused' | 'completed' | 'error';

export interface ScheduledPrompt extends Timestamped {
  id: ID;
  projectId: ID;
  
  // What to run
  name: string;
  description?: string;
  prompt: string;
  contextDocumentIds?: ID[];
  
  // When to run
  type: ScheduleType;
  cronExpression?: string;  // For recurring
  runAt?: Date;             // For one-time
  
  // State
  status: ScheduleStatus;
  nextRunAt?: Date;
  lastRunAt?: Date;
  runCount: number;
  
  // Execution
  targetAgentId?: ID;       // Which agent to run as
  createTask: boolean;      // Create a task for tracking?
  
  // History
  executionHistory: ScheduleExecution[];
  
  // Created by
  createdBy: 'user' | ID;   // User or Agent ID
}

export interface ScheduleExecution {
  id: ID;
  scheduledPromptId: ID;
  runAt: Date;
  completedAt?: Date;
  status: 'running' | 'success' | 'error' | 'cancelled';
  result?: string;
  errorMessage?: string;
  costUSD?: number;
}

// ============================================================================
// Heartbeat Types ⭐ NEW
// ============================================================================

export type HeartbeatStatus = 'running' | 'paused' | 'disabled';
export type HeartbeatPersonality = 'professional' | 'casual' | 'playful' | 'terse';

export interface HeartbeatConfig {
  id: ID;
  projectId: ID;
  
  // Basic settings
  enabled: boolean;
  intervalSeconds: number;  // 30, 60, 300, etc.
  status: HeartbeatStatus;
  
  // Triggers
  requireIdleMs: number;    // Minimum idle time before triggering
  maxSuggestionsPerHour: number;
  
  // Personality
  personality: HeartbeatPersonality;
  customSystemPrompt?: string;
  
  // Features
  enableProactiveSuggestions: boolean;
  enableMusicIntegration: boolean;
  enableBreakReminders: boolean;
  enableDailySummaries: boolean;
  
  // Context gathering
  contextSources: HeartbeatContextSource[];
}

export type HeartbeatContextSource = 
  | 'current_file'
  | 'recent_edits'
  | 'git_status'
  | 'error_logs'
  | 'time_of_day'
  | 'music_playing'
  | 'task_queue';

export interface HeartbeatPulse extends Timestamped {
  id: ID;
  configId: ID;
  
  // Context snapshot
  contextSnapshot: HeartbeatContext;
  
  // What happened
  suggestion?: HeartbeatSuggestion;
  actionTaken?: string;
  
  // Cost
  costUSD: number;
  latencyMs: number;
}

export interface HeartbeatContext {
  timestamp: Date;
  currentFile?: string;
  currentLine?: number;
  recentFiles: string[];
  gitBranch?: string;
  gitStatus?: string;
  activeTaskId?: ID;
  musicPlaying?: string;
  timeOfDay: string;
  errorsLastHour: number;
}

export interface HeartbeatSuggestion {
  id: ID;
  type: HeartbeatSuggestionType;
  content: string;
  confidence: number;       // 0-1
  
  // User feedback
  shownAt: Date;
  dismissedAt?: Date;
  feedback?: 'accepted' | 'dismissed' | 'helpful' | 'not_helpful';
}

export type HeartbeatSuggestionType =
  | 'code_suggestion'
  | 'reminder'
  | 'related_file'
  | 'documentation_gap'
  | 'test_suggestion'
  | 'refactor_opportunity'
  | 'break_reminder'
  | 'music_suggestion'
  | 'daily_summary';

// ============================================================================
// Real-Time Activity Types ⭐ NEW
// ============================================================================

export type ActivityType = 
  | 'file_opened'
  | 'file_edited'
  | 'file_closed'
  | 'command_executed'
  | 'tool_called'
  | 'message_sent'
  | 'task_started'
  | 'task_completed'
  | 'agent_spawned'
  | 'cost_incurred';

export interface ActivityEvent {
  id: ID;
  timestamp: Date;
  projectId: ID;
  
  type: ActivityType;
  agentId?: ID;
  taskId?: ID;
  
  // Description
  title: string;
  description?: string;
  
  // Context
  filePath?: string;
  lineNumber?: number;
  command?: string;
  costUSD?: number;
  
  // Links
  conversationId?: ID;
  messageId?: ID;
}

// ============================================================================
// Shared Editor Types ⭐ NEW
// ============================================================================

export interface EditorPresence {
  agentId: ID;
  filePath: string;
  line: number;
  column: number;
  selectionStart?: { line: number; column: number };
  selectionEnd?: { line: number; column: number };
  lastSeenAt: Date;
}

export interface PendingEdit {
  id: ID;
  agentId: ID;
  filePath: string;
  
  // The proposed change
  originalContent: string;
  proposedContent: string;
  lineStart: number;
  lineEnd: number;
  
  // Metadata
  reason?: string;
  timestamp: Date;
  
  // Status
  status: 'pending' | 'accepted' | 'rejected' | 'stale';
}

// ============================================================================
// Tool System Types ⭐ NEW
// ============================================================================

export type ToolExecutionStatus = 'pending' | 'running' | 'completed' | 'error' | 'cancelled';

export interface ToolExecution extends Timestamped {
  id: ID;
  toolName: ToolName;
  
  // Input
  arguments: Record<string, unknown>;
  workingDirectory?: string;
  
  // Execution
  status: ToolExecutionStatus;
  startedAt?: Date;
  completedAt?: Date;
  
  // Output
  stdout?: string;
  stderr?: string;
  exitCode?: number;
  
  // Optimization
  outputTruncated: boolean;
  fullOutputPath?: string;  // If truncated, where's the full output?
  tokenCount: number;
  
  // Metadata
  costUSD?: number;         // If LLM was used to process output
  durationMs?: number;
  
  // For display
  summary?: string;         // AI-generated summary
  errorExtract?: string;    // Extracted error message
}

export interface ToolPlugin {
  id: ID;
  name: string;
  version: string;
  
  // Type
  type: 'wasm' | 'python' | 'native';
  entryPoint: string;       // File path or wasm module
  
  // Schema
  manifest: ToolPluginManifest;
  
  // State
  enabled: boolean;
  lastLoadedAt?: Date;
  
  // Errors
  loadError?: string;
}

export interface ToolPluginManifest {
  name: string;
  description: string;
  version: string;
  author?: string;
  
  // Permissions
  permissions: ToolPermission[];
  
  // Tools provided
  tools: ToolDefinition[];
}

export type ToolPermission = 
  | 'filesystem:read'
  | 'filesystem:write'
  | 'network:fetch'
  | 'shell:execute'
  | 'python:execute'
  | 'env:read';

// ============================================================================
// Python Integration Types ⭐ NEW
// ============================================================================

export interface PythonExecution extends Timestamped {
  id: ID;
  code: string;
  
  // Execution
  status: 'pending' | 'running' | 'completed' | 'error';
  startedAt?: Date;
  completedAt?: Date;
  
  // Output
  stdout?: string;
  stderr?: string;
  result?: unknown;         // Serialized Python result
  resultType?: string;      // Python type name
  
  // Environment
  virtualenvPath?: string;
  packagesInstalled: string[];
  
  // Resources
  memoryUsedMb?: number;
  durationMs?: number;
}

export interface PythonEnvironment {
  id: ID;
  projectId: ID;
  
  // Config
  pythonVersion: string;
  requirementsPath?: string;
  
  // State
  packages: PythonPackage[];
  lastUsedAt?: Date;
}

export interface PythonPackage {
  name: string;
  version: string;
  isDevDependency: boolean;
}

// ============================================================================
// Multi-Project Types ⭐ NEW
// ============================================================================

export interface Workspace {
  id: ID;
  name: string;
  
  // Projects
  projectIds: ID[];
  activeProjectId: ID;
  
  // Layout
  layout: WorkspaceLayout;
  
  // Global
  globalPromptLibraryId: ID;
  globalAgentIds: ID[];
}

export interface WorkspaceLayout {
  // Which projects visible
  visibleProjectIds: ID[];
  
  // Split configuration
  splitDirection: 'horizontal' | 'vertical';
  splitRatios: number[];    // e.g., [0.5, 0.5] for 50/50
  
  // Per-project state
  projectLayouts: Record<ID, ProjectLayoutState>;
}

export interface ProjectLayoutState {
  openFileIds: ID[];
  activeFileId?: ID;
  sidebarOpen: boolean;
  sidebarWidth: number;
}

// ============================================================================
// Bug Tracking Types ⭐ NEW
// ============================================================================

export type BugStatus = 'open' | 'in_progress' | 'resolved' | 'closed';
export type BugPriority = 'critical' | 'high' | 'medium' | 'low';
export type BugSource = 'manual' | 'tool_error' | 'ai_detected' | 'crash';

export interface Bug extends Timestamped {
  id: ID;
  projectId: ID;
  
  // Basic info
  title: string;
  description?: string;
  status: BugStatus;
  priority: BugPriority;
  source: BugSource;
  
  // Links
  conversationId?: ID;
  messageId?: ID;
  filePath?: string;
  lineNumber?: number;
  commitHash?: string;
  
  // AI suggestions
  suggestedFix?: string;
  relatedBugIds: ID[];
  
  // Assignment
  assignedTo?: ID;          // Agent or user
  
  // Resolution
  resolution?: string;
  resolvedAt?: Date;
}

// ============================================================================
// Client/Server Types ⭐ NEW
// ============================================================================

export type AppMode = 'standalone' | 'server' | 'client';
export type ServerStatus = 'disconnected' | 'connecting' | 'connected' | 'error';
export type ConnectionQuality = 'excellent' | 'good' | 'fair' | 'poor';

export interface ServerConfig {
  id: ID;
  name: string;
  host: string;
  port: number;
  
  // Security
  useTls: boolean;
  authToken?: string;
  
  // Auto-discovered
  discoveredAt?: Date;
  lastConnectedAt?: Date;
}

export interface ClientSession {
  id: ID;
  serverId: ID;
  
  // Connection
  status: ServerStatus;
  connectedAt?: Date;
  disconnectedAt?: Date;
  
  // Quality
  latencyMs?: number;
  connectionQuality: ConnectionQuality;
  
  // State
  isReconnecting: boolean;
  offlineQueueLength: number;
}

export interface ServerInstance {
  id: ID;
  name: string;
  
  // Network
  host: string;
  port: number;
  
  // State
  status: 'running' | 'stopped' | 'error';
  startedAt?: Date;
  
  // Sessions
  connectedClients: ID[];
  activeSessions: ID[];
  
  // Resources
  totalCostUSD: number;
  activeAgents: number;
  activeTasks: number;
}

export interface RemoteEvent {
  id: ID;
  timestamp: Date;
  sessionId: ID;
  
  type: RemoteEventType;
  payload: unknown;
}

export type RemoteEventType =
  | 'agent_update'
  | 'task_update'
  | 'cost_update'
  | 'message_received'
  | 'file_changed'
  | 'tool_output'
  | 'heartbeat';

export interface RemoteCommand {
  id: ID;
  sessionId: ID;
  
  type: RemoteCommandType;
  payload: unknown;
  
  // Response tracking
  sentAt: Date;
  responseReceivedAt?: Date;
  status: 'pending' | 'sent' | 'acknowledged' | 'completed' | 'error';
}

export type RemoteCommandType =
  | 'spawn_agent'
  | 'send_message'
  | 'execute_tool'
  | 'open_file'
  | 'create_task'
  | 'query_costs'
  | 'heartbeat';

// ============================================================================
// API Types (for MCP Server)
// ============================================================================

export interface APIKey {
  id: ID;
  name: string;
  keyHash: string;
  permissions: APIPermission[];
  createdAt: Date;
  lastUsedAt?: Date;
  expiresAt?: Date;
  enabled: boolean;
}

export type APIPermission = 
  | 'read:documents'
  | 'write:documents'
  | 'read:agents'
  | 'manage:agents'
  | 'read:tasks'
  | 'manage:tasks'
  | 'read:costs'
  | 'admin';
