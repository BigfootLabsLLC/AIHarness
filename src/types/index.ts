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
