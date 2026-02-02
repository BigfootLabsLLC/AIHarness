# Implementation Roadmap

## Executive Summary
This roadmap outlines a phased approach to building AIHarness, a multi-agent AI orchestration platform with cost-aware routing. The key innovation is enabling a **"premium guidance + cheap execution"** model where expensive models handle architecture and planning while cheaper models execute implementation tasks.

---

## Phase 1: Foundation (Weeks 1-3)

### 1.1 Project Structure & Core Data Models
**Goal:** Establish the codebase foundation and basic data structures.

```
aiharness/
├── src/
│   ├── models/
│   │   ├── Document.ts          # Markdown with metadata
│   │   ├── Prompt.ts            # Templated prompts
│   │   ├── Project.ts           # Project container
│   │   ├── Agent.ts             # ⭐ Agent session model
│   │   └── Task.ts              # ⭐ Delegated task model
│   ├── storage/
│   │   ├── LocalStorage.ts      # File-based storage
│   │   └── Database.ts          # Future: SQLite/PostgreSQL
│   └── types/
│       └── index.ts
├── ui/
│   ├── components/
│   └── views/
└── docs/
```

**Key Deliverables:**
- [ ] Set up TypeScript/React (or similar) project structure
- [ ] Define core TypeScript interfaces for all data models
- [ ] Implement local file storage for documents
- [ ] Create project initialization and configuration system

### 1.2 Basic UI Layout
**Goal:** Three-pane workspace with document tree and editor.

**Components:**
- Left Rail: Document/project tree with drag-and-drop
- Center: Markdown editor with preview (split view)
- Right Rail: Context panel for metadata/tags

**Key Deliverables:**
- [ ] Tree view component with folder navigation
- [ ] Markdown editor (Monaco or CodeMirror)
- [ ] Live preview renderer
- [ ] Basic theming and layout persistence

---

## Phase 2: AI Provider Integration (Weeks 4-5)

### 2.1 Provider Abstraction Layer
**Goal:** Unified interface for all LLM providers with cost tracking.

```typescript
// Core abstraction
interface LLMProvider {
  readonly name: string;
  readonly costPer1KInput: number;
  readonly costPer1KOutput: number;
  
  sendPrompt(prompt: string, context: Context): Promise<LLMResponse>;
  streamPrompt(prompt: string, context: Context): AsyncIterator<LLMChunk>;
}

interface LLMResponse {
  content: string;
  tokensIn: number;
  tokensOut: number;
  cost: number;
  latency: number;
  model: string;
}
```

**Supported Providers (Phase 2):**
- OpenAI (GPT-3.5, GPT-4)
- Anthropic (Claude 3 Haiku/Sonnet/Opus)
- Local Ollama

**Key Deliverables:**
- [ ] Provider interface and base class
- [ ] OpenAI adapter with cost calculation
- [ ] Anthropic adapter with cost calculation
- [ ] Ollama adapter (local, cost = $0)
- [ ] Provider credential management (secure storage)

### 2.2 Cost Tracking Infrastructure
**Goal:** Every API call tracked with detailed cost breakdown.

```typescript
interface CostLogEntry {
  id: string;
  timestamp: Date;
  provider: string;
  model: string;
  tokensIn: number;
  tokensOut: number;
  costUSD: number;
  taskId?: string;      // Link to task
  agentId?: string;     // Link to agent
  projectId: string;
}
```

**Key Deliverables:**
- [ ] Cost logging service
- [ ] Real-time cost aggregation
- [ ] Budget checking middleware
- [ ] Cost dashboard UI (basic)

---

## Phase 3: Multi-Agent Core (Weeks 6-8)

### 3.1 Agent Session Management
**Goal:** Spawn, monitor, and manage multiple AI agents.

```typescript
interface Agent {
  id: string;
  name: string;
  role: AgentRole;           // Architect, Implementer, Reviewer, etc.
  provider: string;          // Which LLM provider
  model: string;             // Specific model
  status: AgentStatus;       // idle, working, waiting_review, error
  
  // Context management
  systemPrompt: string;
  conversationHistory: Message[];
  maxContextTokens: number;
  
  // Cost tracking
  totalCost: number;
  costBudget?: number;       // Optional budget cap
  
  // Current work
  currentTaskId?: string;
}
```

**Agent Roles (Initial Set):**
| Role | Model Tier | Typical Tasks |
|------|------------|---------------|
| Architect | Premium | High-level design, planning, reviews |
| Implementer | Cheap | Code implementation, tests |
| Reviewer | Medium | Code review, quality checks |
| Tester | Cheap | Test case generation, validation |

**Key Deliverables:**
- [ ] Agent registry and factory
- [ ] Agent session lifecycle (create, start, pause, resume, terminate)
- [ ] Context window management (trimming, summarization)
- [ ] Agent dashboard UI (list view with status)

### 3.2 Task System & Delegation
**Goal:** Break work into tasks and delegate to agents.

```typescript
interface Task {
  id: string;
  title: string;
  description: string;
  type: TaskType;            // architecture, spec, interface, implementation, review
  
  // Assignment
  assignedTo?: string;       // Agent ID
  reviewedBy?: string;       // For review tasks
  
  // Status workflow
  status: TaskStatus;        // pending, in_progress, awaiting_review, approved, rejected
  
  // Dependencies
  dependencies: string[];    // Task IDs that must complete first
  blocks: string[];          // Task IDs blocked by this one
  
  // Work product
  inputs: DocumentRef[];     // Context documents
  outputs: DocumentRef[];    // Generated documents
  
  // Cost management
  costBudget?: number;
  costSpent: number;
  
  // Quality
  reviewFeedback?: ReviewFeedback[];
}
```

**Key Deliverables:**
- [ ] Task queue and dependency resolver
- [ ] Task assignment logic
- [ ] Task status workflow engine
- [ ] Task board UI (Kanban-style)

### 3.3 Basic Delegation Workflow
**Goal:** Architecture → Spec → Implementation pipeline.

**Workflow Stages:**
1. **Architecture** (Premium model)
   - Input: High-level requirements
   - Output: Architecture document with component breakdown
   - Gate: Human approval required

2. **Specification** (Premium or Medium model)
   - Input: Architecture document
   - Output: Detailed spec with interfaces
   - Gate: Human approval optional

3. **Implementation** (Cheap model)
   - Input: Spec document
   - Output: Code implementation
   - Gate: Auto-submit for review

4. **Review** (Medium model or different cheap model)
   - Input: Implementation + Spec
   - Output: Review feedback
   - Gate: Human approval if issues found

**Key Deliverables:**
- [ ] Workflow definition engine
- [ ] Stage transition logic with gates
- [ ] Context passing between stages
- [ ] Basic workflow UI (progress tracking)

---

## Phase 4: Advanced Orchestration (Weeks 9-11)

### 4.1 Agent-to-Agent Review
**Goal:** Agents review each other's work with feedback threads.

```typescript
interface ReviewFeedback {
  id: string;
  reviewerAgentId: string;
  originalTaskId: string;
  
  // Review content
  summary: string;
  findings: Finding[];
  
  // Verdict
  verdict: 'approve' | 'approve_with_nits' | 'request_changes' | 'reject';
  confidence: number;        // 0-1, how sure is the reviewer
}

interface Finding {
  severity: 'critical' | 'major' | 'minor' | 'nit';
  location?: CodeLocation;
  description: string;
  suggestion?: string;
}
```

**Key Deliverables:**
- [ ] Review assignment system
- [ ] Review prompt templates
- [ ] Feedback collection UI
- [ ] Dispute escalation (human in the loop)

### 4.2 Cost-Aware Routing Engine
**Goal:** Automatically select the cheapest capable model.

```typescript
interface RoutingRule {
  id: string;
  taskType: TaskType;
  complexity: 'low' | 'medium' | 'high';
  maxCost?: number;
  minQuality?: number;
  preferredProvider?: string;
  fallbackChain: string[];   // Ordered list of models to try
}

// Example rule
const implementationRule: RoutingRule = {
  taskType: 'implementation',
  complexity: 'low',
  fallbackChain: ['gpt-3.5-turbo', 'claude-3-haiku', 'claude-3-sonnet'],
};
```

**Routing Strategies:**
- **Fixed:** Always use specified model
- **Cost-Optimized:** Cheapest model in fallback chain
- **Quality-Optimized:** Best quality within budget
- **Adaptive:** Learn from past performance

**Key Deliverables:**
- [ ] Routing rule engine
- [ ] Cost estimation before API calls
- [ ] Quality scoring feedback loop
- [ ] Routing configuration UI

### 4.3 Results Aggregation
**Goal:** Combine outputs from multiple cheap agents.

**Use Cases:**
- Generate 5 implementations with cheap model, pick best
- Have 3 agents review same code, aggregate findings
- Compare outputs to build consensus

```typescript
interface AggregationResult {
  outputs: AgentOutput[];
  consensusScore: number;
  mergedOutput?: string;
  disagreements: Disagreement[];
}
```

**Key Deliverables:**
- [ ] Parallel execution controller
- [ ] Consensus scoring algorithm
- [ ] Output merging strategies
- [ ] Comparison UI

### 4.4 Expert Panel System ⭐ NEW
**Goal:** "Poll the Experts" - query multiple models simultaneously and compare responses.

**Modes:**

1. **Poll Mode** (Simple)
   - Send same question to 3-7 models
   - Side-by-side response comparison
   - Cost and latency comparison per model

2. **Debate Mode** (Advanced)
   - Round-robin: each model responds to others
   - Critique and revise positions
   - Final consensus or majority vote

3. **Synthesis Mode**
   - Generate unified answer from all responses
   - Highlight areas of agreement/disagreement
   - Attribute specific claims to models

```typescript
interface ExpertPanel {
  id: string;
  prompt: string;
  mode: 'poll' | 'debate' | 'synthesis';
  participants: PanelParticipant[];
  responses: PanelResponse[];
  consensusReport?: ConsensusReport;
}

interface PanelParticipant {
  modelId: string;
  provider: string;
  role?: string;        // e.g., "Skeptic", "Optimist"
  weight: number;       // For weighted consensus
}
```

**Key Deliverables:**
- [ ] Panel configuration UI (select models, set roles)
- [ ] Parallel query execution to multiple providers
- [ ] Side-by-side response comparison UI
- [ ] Variance analysis (semantic similarity, disagreement detection)
- [ ] Consensus scoring and visualization
- [ ] Cost comparison per model

---

## Phase 5: Polish & Integration (Weeks 12-13)

### 5.1 HTTP Tool Server Mode
**Goal:** Expose AIHarness as a callable tool for external AIs.

**Capabilities:**
- Query documents by tag/project
- Get agent status
- Submit tasks
- Retrieve cost reports

**Key Deliverables:**
- [ ] REST API server
- [ ] Authentication/authorization
- [ ] API documentation
- [ ] Client SDK

### 5.2 Harness Mode
**Goal:** Built-in AI assistant for the UI itself.

**Features:**
- Natural language task creation
- "Delegate this to an Implementer agent"
- "Review this architecture"
- Context-aware suggestions

**Key Deliverables:**
- [ ] In-app AI assistant
- [ ] Context-aware prompting
- [ ] Action execution (create task, spawn agent, etc.)

### 5.3 Final Polish
- [ ] Comprehensive testing
- [ ] Documentation
- [ ] Onboarding flow
- [ ] Error handling and recovery

---

## Phase 6: Advanced Features (Weeks 14-16)

### 6.1 Structured Debate Mode ⭐ NEW
**Goal:** Multi-round debates where models critique and revise positions.

**Flow:**
1. **Opening Statements** - Each model presents initial position
2. **Critique Round** - Each model critiques others' positions
3. **Revision Round** - Models revise based on feedback
4. **Final Position** - Models state final consensus or disagreement
5. **Synthesis** - Generate unified answer with dissent notes

**Key Deliverables:**
- [ ] Debate orchestration engine
- [ ] Round management and sequencing
- [ ] Transcript generation
- [ ] Revision diff visualization
- [ ] Dissent recording and attribution

### 6.2 Advanced Cost Optimization
- [ ] Learned routing preferences based on past results
- [ ] Cost forecasting for planned work
- [ ] Smart batching of similar requests

### 6.3 Team Collaboration Features
- [ ] Shared agent pools across team
- [ ] Workspace sharing and permissions
- [ ] Comment threads on agent outputs

---

## Phase 7: Chat System & Control Center (Weeks 17-20)

### 7.1 Conversation Management
**Goal:** Full CLI replacement with persistent, forkable conversations.

**Key Deliverables:**
- [ ] Conversation data model (tree structure, not linear)
- [ ] SQLite schema for message history
- [ ] Conversation forking (copy-on-write)
- [ ] Time travel navigation (tree browser)
- [ ] Full-text search across conversations
- [ ] Export to Markdown/HTML/JSON

### 7.2 Tool Use Integration
**Goal:** AI can execute shell commands, edit files, use Git.

**Key Deliverables:**
- [ ] Tool definition system (JSON schema)
- [ ] Shell tool (with user approval)
- [ ] File system tools (read, write, list, grep)
- [ ] Git tools (status, diff, commit, branch)
- [ ] Web tools (fetch, search)
- [ ] Tool output streaming to UI
- [ ] Approval gates for destructive operations

### 7.3 Chat UI
**Goal:** Rich conversation interface with tool visualization.

**Key Deliverables:**
- [ ] Message thread view (infinite scroll)
- [ ] Code block syntax highlighting
- [ ] Tool call cards (expandable)
- [ ] Message editing (user and AI)
- [ ] Branch/fork UI controls
- [ ] Command palette (Cmd+K)

---

## Phase 8: Real-Time Collaboration (Weeks 21-23)

### 8.1 AI Todo Visibility
**Goal:** Always-visible AI task progress.

**Key Deliverables:**
- [ ] Persistent sidebar widget
- [ ] Real-time progress streaming
- [ ] Subtask tree with checkboxes
- [ ] ETA tracking
- [ ] Blocked state visualization

### 8.2 Shared Editor Presence
**Goal:** AI and user share the editor like Google Docs.

**Key Deliverables:**
- [ ] File open sync (AI opens → tab opens)
- [ ] Cursor/scroll sync (follow mode)
- [ ] Ghost cursor rendering
- [ ] Real-time edit preview (diff view)
- [ ] Change accept/reject UI

### 8.3 Activity Feed
**Goal:** Stream of everything AI is doing.

**Key Deliverables:**
- [ ] Event collection system
- [ ] Real-time feed component
- [ ] Cost ticker
- [ ] Filtering by type/agent

---

## Phase 9: Scheduling System (Weeks 24-25)

### 9.1 Scheduler Core
**Goal:** Cron-like scheduling with AI self-scheduling.

**Key Deliverables:**
- [ ] Cron parser and execution engine
- [ ] Schedule storage (SQLite)
- [ ] Background daemon process
- [ ] Calendar view UI
- [ ] Execution history

### 9.2 AI Self-Scheduling
**Goal:** AI can schedule future work.

**Key Deliverables:**
- [ ] API for AI to create schedules
- [ ] Conditional scheduling rules
- [ ] User approval workflow
- [ ] Schedule dashboard

---

## Phase 10: Heartbeat System (Weeks 26-28) ⭐ EXPERIMENTAL

### 10.1 Core Heartbeat
**Goal:** Living AI companion with tunable pulse.

**Key Deliverables:**
- [ ] Heartbeat timer and configuration
- [ ] Context gathering (files, git, system state)
- [ ] Smart triggering (idle detection)
- [ ] Pause/resume controls
- [ ] Cost tracking for heartbeat

### 10.2 Proactive Suggestions
**Goal:** Context-aware help without asking.

**Key Deliverables:**
- [ ] Suggestion generation logic
- [ ] Non-intrusive UI (sidebar, not modal)
- [ ] User feedback capture (thumbs up/down)
- [ ] Suggestion history

### 10.3 Ambient Features
**Goal:** Music, breaks, daily summaries.

**Key Deliverables:**
- [ ] Music player integration (Spotify, Apple Music)
- [ ] Pomodoro timer
- [ ] Break reminders
- [ ] Daily standup generation
- [ ] Startup briefing

---

## Phase 11: Tool System & Plugins (Weeks 29-31)

### 11.1 Plugin Architecture
**Goal:** Simple, extensible tool system (WASM, Python, Native).

**Key Deliverables:**
- [ ] WASM plugin runtime
- [ ] Python plugin support (embedded)
- [ ] Native Rust plugin loading
- [ ] Plugin manifest and permissions
- [ ] Hot-reload for development

### 11.2 Token-Optimized Output
**Goal:** Smart truncation and summarization for tool output.

**Key Deliverables:**
- [ ] Output truncation with preservation
- [ ] Error extraction from stack traces
- [ ] Build output summarization
- [ ] Directory listing optimization
- [ ] "Show full output" option

---

## Phase 12: Python Embedded (Weeks 32-34)

### 12.1 Python Runtime
**Goal:** Embedded Python for AI to use ML/data tools.

**Key Deliverables:**
- [ ] PyO3 integration
- [ ] Virtualenv per project
- [ ] Package management
- [ ] Security sandboxing

### 12.2 Rust-Python Bridge
**Goal:** Seamless data exchange.

**Key Deliverables:**
- [ ] Serialize Rust structs to Python
- [ ] Return Python results to Rust
- [ ] Shared memory for large data
- [ ] Jupyter-like cell execution

---

## Phase 13: Multi-Project Workspace (Weeks 35-36)

### 13.1 Project Management
**Goal:** Work on multiple projects simultaneously.

**Key Deliverables:**
- [ ] Multiple open projects
- [ ] Fast switching
- [ ] Split views
- [ ] Independent windows

### 13.2 Context Preservation
**Goal:** Per-project state isolation.

**Key Deliverables:**
- [ ] Per-project files/tabs
- [ ] Per-project AI todos
- [ ] Per-project conversations
- [ ] Global/shared resources

---

## Phase 14: Bug Tracking (Weeks 37-38)

### 14.1 Lightweight Issues
**Goal:** AI-friendly bug tracking without bloat.

**Key Deliverables:**
- [ ] Quick issue capture
- [ ] Auto-capture from tool errors
- [ ] AI triage and suggestions
- [ ] Duplicate detection

---

## Phase 15: Model Provider Support (Weeks 39-40)

### 15.1 Provider Integrations
**Goal:** Support all major providers.

**Key Deliverables:**
- [ ] OpenAI
- [ ] Anthropic
- [ ] Google (Gemini)
- [ ] Moonshot AI (Kimi)
- [ ] Local models (Ollama)
- [ ] xAI (Grok)

### 15.2 Authentication
**Goal:** Secure, flexible auth.

**Key Deliverables:**
- [ ] API key management
- [ ] OAuth flows
- [ ] Multi-account support
- [ ] Environment variable fallback

---

## Phase 16: Token Optimization (Weeks 41-42)

### 16.1 System-Wide Optimization
**Goal:** Every token counts.

**Key Deliverables:**
- [ ] Token budget enforcement
- [ ] Pre-send cost estimation
- [ ] Context compression
- [ ] Rolling summarization
- [ ] Lazy file loading

### 16.2 Output Optimization
**Goal:** Smart truncation everywhere.

**Key Deliverables:**
- [ ] Tool output summarization
- [ ] Error extraction
- [ ] Diff-based updates
- [ ] Symbol-level references

---

## Phase 17: Client/Server Mode (Weeks 43-46)

**Principle:** LAN-only. No port forwarding. No external exposure.

### 17.1 Server Architecture
**Goal:** Headless server running on powerful desktop, LAN-only.

**Key Deliverables:**
- [ ] Server binary (headless mode)
- [ ] Bind to local interfaces only (192.168.x.x, 10.x.x.x)
- [ ] WebSocket server (internal network)
- [ ] gRPC service for commands
- [ ] Session persistence
- [ ] Multi-client support (same network)

### 17.2 Client Architecture
**Goal:** Thin client that connects to server on local network.

**Key Deliverables:**
- [ ] Client mode UI
- [ ] Server discovery (mDNS on LAN)
- [ ] Connection management
- [ ] Reconnect/resume logic
- [ ] Offline queue (queue commands when disconnected)

### 17.3 Communication Protocol
**Goal:** Efficient bidirectional streaming over local network.

**Key Deliverables:**
- [ ] WebSocket event streaming
- [ ] gRPC command protocol
- [ ] File sync (laptop ↔ desktop on LAN)
- [ ] TLS encryption (even on LAN)
- [ ] Token-based authentication

### 17.4 Security (LAN-Only)
**Goal:** No external exposure, firewall friendly.

**Key Deliverables:**
- [ ] Default to local network interfaces only
- [ ] No port forwarding required
- [ ] No direct internet exposure
- [ ] Future: VPN/Tailscale integration for remote (user-controlled)
- [ ] Future: Relay server option (traffic through middle, never direct)
- [ ] Authentication/authorization

---

## Phase 18: External CLI Agent Integration (v2.0) ⭐ PLANNED

### 18.1 Overview
**Goal:** Embed external CLI-based AI agents (Kimi, Claude Code, etc.) directly into AIHarness as first-class citizens within the Control Center architecture.

**Philosophy:** Instead of building everything from scratch, leverage best-in-class CLI tools while providing a unified management interface and shared context.

### 18.2 Right Sidebar Agents Panel

**Goal:** Dedicated panel in the right sidebar for managing active CLI agents.

**UI Components:**
```
┌─────────────────────────────┐
│  Active Agents          [+] │  ← Header with "New Agent" button
├─────────────────────────────┤
│                             │
│  [●] Kimi #1                │  ← Active agent entry
│      Thinking...            │     (pulsing indicator when active)
│                             │
│  [⚠] Claude Code            │  ← Agent needing attention
│      Approval pending       │     (alert indicator)
│                             │
│  [●] Kimi #2                │  ← Another agent instance
│      Idle                   │
│                             │
└─────────────────────────────┘
```

**Agent Entry Display:**
- **Status Indicator:** Pulsing dot when active, static when idle, warning icon when needs attention
- **Agent Name/Type:** Human-readable name + CLI type (Kimi, Claude, etc.)
- **Activity Summary:** Brief status text ("Thinking...", "Approval pending", "Running tests...")
- **Thinking Details:** Optional expandable thinking/reasoning display (if CLI provides it)
- **Alert Badge:** Visual indicator when agent requires user intervention

**Key Deliverables:**
- [ ] Agents panel container in right sidebar
- [ ] Agent entry list component with status indicators
- [ ] "New Agent" button with popup selector
- [ ] Click-to-focus interaction (opens agent in main view)
- [ ] Real-time status updates via backend events

### 18.3 Agent Type System & Custom Containers

**Goal:** Pluggable architecture supporting multiple CLI agent types, each with custom configuration and behavior.

**Core Abstraction:**
```rust
pub trait CliAgentType: Send + Sync {
    /// Unique identifier for this agent type (e.g., "kimi", "claude-code")
    fn id(&self) -> &'static str;
    
    /// Display name for UI (e.g., "Kimi Code CLI")
    fn display_name(&self) -> &'static str;
    
    /// Icon/path for UI representation
    fn icon(&self) -> Option<&'static str>;
    
    /// Create a new agent instance
    fn create_container(&self, config: AgentConfig) -> Box<dyn CliAgentContainer>;
    
    /// Available configuration options for new agent modal
    fn configuration_schema(&self) -> Vec<ConfigOption>;
    
    /// Check if this agent type is available (binary installed, etc.)
    fn is_available(&self) -> bool;
}

pub trait CliAgentContainer: Send + Sync {
    /// Start the agent process
    async fn start(&mut self) -> Result<(), AgentError>;
    
    /// Send input to the agent
    async fn send_input(&mut self, input: &str) -> Result<(), AgentError>;
    
    /// Get current status
    fn status(&self) -> AgentStatus;
    
    /// Get thinking/reasoning data if available
    fn thinking_state(&self) -> Option<ThinkingState>;
    
    /// Subscribe to output/events stream
    fn subscribe_events(&self) -> broadcast::Receiver<AgentEvent>;
    
    /// Terminate the agent
    async fn terminate(&mut self) -> Result<(), AgentError>;
}
```

**Agent Configuration Options:**
```rust
pub struct AgentConfig {
    /// Working directory for this agent
    pub working_dir: PathBuf,
    
    /// Environment variables
    pub env_vars: HashMap<String, String>,
    
    /// Type-specific configuration
    pub type_config: HashMap<String, ConfigValue>,
    
    /// Session persistence ID (optional)
    pub session_id: Option<String>,
    
    /// Cost tracking enabled
    pub track_costs: bool,
}

pub enum ConfigValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Select { options: Vec<String>, default: String },
}
```

**Key Deliverables:**
- [ ] `CliAgentType` trait definition
- [ ] `CliAgentContainer` trait definition
- [ ] Agent type registry (dynamic registration)
- [ ] Configuration schema system
- [ ] Availability detection for each agent type

### 18.4 Kimi CLI Integration (Reference Implementation)

**Goal:** Full Kimi CLI support via Wire mode as the first implemented agent type.

**KimiAgentContainer Implementation:**
```rust
pub struct KimiAgentContainer {
    process: Option<Child>,
    stdin: Option<ChildStdin>,
    event_sender: broadcast::Sender<AgentEvent>,
    status: Arc<RwLock<AgentStatus>>,
    thinking_state: Arc<RwLock<Option<ThinkingState>>>,
    wire_protocol: WireProtocolHandler,
}

impl CliAgentContainer for KimiAgentContainer {
    async fn start(&mut self) -> Result<(), AgentError> {
        // Spawn: kimi --wire --work-dir <dir> --session <id>
        // Initialize wire protocol handshake
        // Start event streaming tasks
    }
    
    async fn send_input(&mut self, input: &str) -> Result<(), AgentError> {
        // Send JSON-RPC prompt request via stdin
        // Handle streaming response events
    }
    
    fn thinking_state(&self) -> Option<ThinkingState> {
        // Extract from wire protocol events
        // Kimi provides thinking mode via --thinking flag
    }
}
```

**Kimi-Specific Configuration:**
- `--model`: Model selection (e.g., "kimi-k2")
- `--thinking`: Enable thinking mode
- `--session`: Session ID for persistence
- `--work-dir`: Working directory
- `--mcp-config-file`: MCP server configuration
- `--yolo`: Auto-approve mode (use with caution)

**Wire Protocol Event Mapping:**
```rust
pub enum AgentEvent {
    /// Agent started successfully
    Started,
    
    /// Agent terminated
    Stopped { reason: StopReason },
    
    /// Output chunk from agent
    Output { content: String, is_stderr: bool },
    
    /// Agent is thinking/reasoning
    Thinking { content: String },
    
    /// Tool call executed
    ToolCall { name: String, arguments: Value, result: Value },
    
    /// Approval request from agent
    ApprovalRequest { 
        request_id: String, 
        tool_name: String, 
        arguments: Value,
        description: String,
    },
    
    /// Status changed
    StatusChanged { old: AgentStatus, new: AgentStatus },
    
    /// Error occurred
    Error { message: String, fatal: bool },
}
```

**Key Deliverables:**
- [ ] Kimi agent type implementation
- [ ] Wire protocol JSON-RPC handler
- [ ] Event streaming to frontend
- [ ] Approval request routing to UI
- [ ] Session persistence support

### 18.5 New Agent Creation Flow

**Goal:** Simple modal/dialog for creating new agent instances.

**UI Flow:**
1. User clicks `[+]` in Agents panel
2. Modal appears showing available agent types:
   ```
   ┌─────────────────────────────────────┐
   │  Start New Agent              [×]   │
   ├─────────────────────────────────────┤
   │                                     │
   │  [🌙] Kimi Code CLI            →   │
   │      Available (v0.69.0)            │
   │                                     │
   │  [◐] Claude Code               →   │
   │      Not installed                  │
   │                                     │
   │  [●] Aider                       →   │
   │      Available                      │
   │                                     │
   └─────────────────────────────────────┘
   ```
3. User selects agent type
4. Configuration form appears with type-specific options:
   ```
   ┌─────────────────────────────────────┐
   │  Configure Kimi Agent         [×]   │
   ├─────────────────────────────────────┤
   │                                     │
   │  Name: [Kimi Helper          ]      │
   │                                     │
   │  Working Directory:                 │
   │  [/path/to/project           ] […]  │
   │                                     │
   │  Model: [kimi-k2 ▼]                 │
   │                                     │
   │  [✓] Enable thinking mode           │
   │                                     │
   │  Session: [New session ▼]           │
   │                                     │
   │        [Cancel]  [Start Agent]      │
   │                                     │
   └─────────────────────────────────────┘
   ```
5. Agent starts, appears in sidebar, auto-opens in main view

**Key Deliverables:**
- [ ] Agent type selector modal
- [ ] Dynamic configuration form generation
- [ ] Working directory picker
- [ ] Session selection (new/existing)
- [ ] Form validation

### 18.6 Main Content Agent View

**Goal:** Rich, interactive view for interacting with an active agent.

**Layout:**
```
┌─────────────────────────────────────────────────────────────┐
│  [◀] Kimi Helper                    [⚙] [⏸] [⏹] [×]       │  ← Header with controls
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─ Conversation ────────────────────────────────────────┐ │
│  │                                                       │ │
│  │  User: Refactor this code                            │ │
│  │                                                       │ │
│  │  [Thinking... ▼]                                     │ │  ← Collapsible thinking
│  │  ├─ I'll analyze the code structure...               │ │
│  │  └─ Looking for repeated patterns...                 │ │
│  │                                                       │ │
│  │  Kimi: I'll help refactor this code.                  │ │
│  │                                                       │ │
│  │  ┌─ Tool: ReadFile ─────────────────────────────────┐│ │
│  │  │ Path: src/utils.ts                               ││ │
│  │  │ [Show output ▼]                                 ││ │
│  │  └──────────────────────────────────────────────────┘│ │
│  │                                                       │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
│  ┌─ Context & Files ─────────────────────────────────────┐ │  ← Right panel or tabs
│  │ [+ Add File]  [+ Add Folder]  [+ Paste Text]         │ │
│  │ • README.md                                          │ │
│  │ • src/main.ts                                        │ │
│  │ • docs/api.md                                        │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
│  $ █                                                        │  ← Input line (sticky)
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**Features:**
- **Header:** Back to agents list, settings, pause/resume, stop, close
- **Conversation View:** Scrollable message history with:
  - User messages (editable)
  - AI responses with markdown rendering
  - Thinking/reasoning blocks (collapsible)
  - Tool call cards (expandable with arguments and results)
  - Approval request banners (with approve/reject buttons)
- **Context Panel:** Add/remove files, folders, or text snippets to agent context
- **Input Line:** Command input with history, autocomplete, @mentions for files

**Input Features:**
- `/` commands (passthrough to CLI)
- `@` mentions for file references
- Up/down arrow for history
- Multi-line input (Shift+Enter)
- Drag-and-drop files into context

**Key Deliverables:**
- [ ] Agent view container component
- [ ] Conversation message list with virtual scrolling
- [ ] Message rendering (markdown, code blocks, tool calls)
- [ ] Thinking/reasoning display component
- [ ] Tool call visualization cards
- [ ] Approval request UI
- [ ] Context panel with file management
- [ ] Input line with history and autocomplete
- [ ] Header with agent controls

### 18.7 Thinking State Display

**Goal:** Visualize agent reasoning/thinking when CLI provides it.

**Kimi-Specific:**
- Kimi supports `--thinking` flag
- Wire protocol may emit thinking events
- Display as collapsible blocks between user input and AI response

**UI Pattern:**
```
User: How do I optimize this database query?

[🧠 Thinking... (click to expand)]
  ├─ The user is asking about database optimization
  ├─ I should check if there's an existing schema
  └─ Possible approaches: indexing, query restructuring, caching

Kimi: Here are several ways to optimize your query...
```

**Other Agents:**
- Some agents may provide thinking via stderr
- Some via custom protocol extensions
- Container should normalize to common `ThinkingState` format

**Key Deliverables:**
- [ ] Thinking state data model
- [ ] Collapsible thinking block UI
- [ ] Real-time thinking updates
- [ ] Per-agent-type thinking extraction

### 18.8 Alert/Notification System

**Goal:** Notify user when agent needs attention (approval, error, completion).

**Alert Types:**
- **Approval Required:** Agent wants to execute a tool/command
- **Error:** Agent encountered an error and stopped
- **Completed:** Agent finished its task
- **Budget Warning:** Approaching cost limit
- **Input Required:** Agent needs clarification

**UI Indicators:**
- Sidebar: Badge count on agent entry
- Main view: Banner at top
- System: Optional native notification

**Approval Flow:**
```
┌─────────────────────────────────────────────────────────────┐
│  ⚠️ Approval Required                                       │
│  Kimi wants to execute: Shell("rm -rf node_modules")        │
│                                                             │
│  [View Details] [Approve] [Reject] [Approve Always]         │
└─────────────────────────────────────────────────────────────┘
```

**Key Deliverables:**
- [ ] Alert state management
- [ ] Sidebar badge system
- [ ] In-view alert banners
- [ ] Approval dialog/detail view
- [ ] "Always approve" options (per tool, per session)
- [ ] Native notification integration (optional)

### 18.9 Future Extensibility Points

**Planned Future Features:**
- **Agent Templates:** Save/load common agent configurations
- **Agent Teams:** Link multiple agents for coordinated work
- **Agent Scripting:** Automate agent interactions
- **Cost Sharing:** Allocate budget across agents
- **Context Sharing:** Shared context between agents
- **Agent Marketplace:** Download configurations for specific tasks
- **Custom Agents:** Plugin system for non-CLI agents
- **Agent Comparison:** Run same task on multiple agents, compare results

**Extensibility Hooks:**
```rust
// Plugin extension point
pub trait AgentExtension {
    fn on_event(&self, event: &AgentEvent) -> Option<AgentEvent>;
    fn modify_ui(&self, ui: &mut AgentViewBuilder);
}

// Custom command handlers
pub trait AgentCommandHandler {
    fn handles(&self, command: &str) -> bool;
    fn execute(&self, command: &str, agent: &mut dyn CliAgentContainer) -> Result<String, Error>;
}
```

### 18.10 Backend Architecture

**Rust Implementation Sketch:**
```rust
// src-tauri/src/agents/
pub mod types;        // Core traits and types
pub mod manager;      // AgentManager for lifecycle
pub mod kimi;         // Kimi CLI integration
pub mod registry;     // Agent type registry

// AgentManager
pub struct AgentManager {
    registry: Arc<AgentTypeRegistry>,
    active_agents: RwLock<HashMap<AgentId, Box<dyn CliAgentContainer>>>,
    event_bus: broadcast::Sender<AgentManagerEvent>,
}

impl AgentManager {
    pub async fn create_agent(
        &self, 
        agent_type: &str, 
        config: AgentConfig
    ) -> Result<AgentId, AgentError>;
    
    pub async fn send_to_agent(
        &self, 
        agent_id: AgentId, 
        input: &str
    ) -> Result<(), AgentError>;
    
    pub fn get_agent_status(&self, agent_id: AgentId) -> Option<AgentStatus>;
    
    pub async fn terminate_agent(&self, agent_id: AgentId) -> Result<(), AgentError>;
}
```

**Key Deliverables:**
- [ ] AgentManager service
- [ ] Agent type registry
- [ ] Process lifecycle management
- [ ] Event streaming to frontend
- [ ] Session persistence
- [ ] Cost tracking per agent

---

## Phase 19: Agent Conversation Control Design (v2.0) ⭐ TECHNICAL SPEC

### 19.1 Design Considerations Todo List

**Data Model:**
- [ ] Define TypeScript interfaces for all message types
- [ ] Design message metadata schema (cost, latency, model info)
- [ ] Handle @mention resolution and caching
- [ ] Design attachment system (files, folders, text snippets)
- [ ] Design conversation branching/forking data model (future)
- [ ] Message versioning for edits

**Component Hierarchy:**
- [ ] Define component tree structure
- [ ] Design prop interfaces between components
- [ ] Plan for component reusability (tool cards, etc.)
- [ ] Lazy loading strategy for long conversations
- [ ] Split pane layout for context panel

**State Management:**
- [ ] Zustand store design for conversation state
- [ ] Separate UI state from message data
- [ ] Optimistic updates vs server confirmation
- [ ] State persistence strategy (localStorage, session storage)
- [ ] Undo/redo considerations

**Streaming Flow:**
- [ ] Design streaming buffer strategy (chunk accumulation)
- [ ] Flush timing (punctuation vs interval-based)
- [ ] Handle reconnection during streaming
- [ ] Cancel/pause streaming mid-response
- [ ] Render partial markdown gracefully

**Input System:**
- [ ] @mention autocomplete (file path resolution)
- [ ] /command autocomplete
- [ ] Input history navigation (up/down arrows)
- [ ] Multi-line input handling (Shift+Enter)
- [ ] IME composition support (CJK input)
- [ ] Drag-and-drop file attachment
- [ ] Paste image handling
- [ ] Input validation and sanitization

**Tool Call Visualization:**
- [ ] Design tool call card structure
- [ ] Status indicators (pending/running/completed/error)
- [ ] Expandable arguments/result sections
- [ ] Special renderers for common tools (read_file, shell)
- [ ] Terminal output styling for shell commands
- [ ] Syntax highlighting for code results
- [ ] Error state visualization

**Thinking/Reasoning Display:**
- [ ] Collapsible thinking block design
- [ ] Real-time thinking updates
- [ ] Tree structure for multi-step reasoning
- [ ] Link thinking to final response
- [ ] Persist user expand/collapse preference

**Approval System:**
- [ ] Approval request banner design
- [ ] Risk level indicators (color coding)
- [ ] Detail view for complex requests
- [ ] "Always approve" options (per-tool, per-session)
- [ ] Approval timeout handling
- [ ] Batch approvals (multiple pending)

**Scroll & Virtualization:**
- [ ] Choose virtualization library (react-virtuoso)
- [ ] Dynamic item height handling
- [ ] Auto-scroll behavior (sticky bottom)
- [ ] "New messages" indicator when scrolled up
- [ ] Scroll to specific message
- [ ] Handle streaming at bottom smoothly
- [ ] Search result scroll-to behavior

**Context/Attachments:**
- [ ] Context panel design
- [ ] File tree vs flat list
- [ ] Drag-and-drop zones
- [ ] Context size limits/warnings
- [ ] Quick-add from recent files
- [ ] Context persistence with conversation

**Error & Recovery:**
- [ ] Network error handling
- [ ] Agent process crash recovery
- [ ] Retry mechanisms for failed messages
- [ ] Error message design
- [ ] Partial state recovery

**Performance:**
- [ ] Message list virtualization
- [ ] Image lazy loading
- [ ] Large conversation handling (>1000 messages)
- [ ] Memory management for long sessions
- [ ] Debounced search/filtering

**Accessibility:**
- [ ] Keyboard navigation
- [ ] ARIA labels for dynamic content
- [ ] Screen reader announcements for streaming
- [ ] Focus management
- [ ] High contrast mode support

**Theming:**
- [ ] CSS variable integration
- [ ] Dark/light mode support
- [ ] Custom accent color handling
- [ ] Code block theme consistency

### 19.2 Data Model Specification

```typescript
// ==================== Core Types ====================

type MessageId = string;
type AgentId = string;
type ToolCallId = string;

interface Message {
  id: MessageId;
  type: MessageType;
  timestamp: Date;
  // Common metadata
  metadata?: {
    cost?: number;           // Token cost for this message
    latencyMs?: number;      // Response time
    model?: string;          // Which model generated this
  };
}

type MessageType = 
  | UserMessage 
  | AssistantMessage 
  | ToolCallMessage 
  | ToolResultMessage 
  | SystemMessage
  | ThinkingMessage;

// -------------------- User Input --------------------

interface UserMessage extends Message {
  type: 'user';
  content: string;           // Raw input (may contain @mentions)
  parsedContent?: ParsedContent;  // Processed mentions
  attachments?: Attachment[];     // Files/context added
}

interface ParsedContent {
  text: string;
  mentions: Mention[];       // @file references
  commands: Command[];       // /slash commands
}

interface Mention {
  type: 'file' | 'folder' | 'symbol';
  raw: string;               // "@src/main.ts"
  resolvedPath?: string;     // Absolute path
  content?: string;          // Fetched content at send time
}

interface Attachment {
  id: string;
  type: 'file' | 'folder' | 'text' | 'image';
  name: string;
  content: string;
  size?: number;
}

// -------------------- AI Response --------------------

interface AssistantMessage extends Message {
  type: 'assistant';
  content: string;           // Markdown content
  contentParts: ContentPart[];  // Structured for streaming
  thinkingBlockId?: string;  // Link to associated thinking
  isStreaming: boolean;      // Currently receiving?
  isComplete: boolean;       // Finished?
}

type ContentPart = 
  | { type: 'text'; text: string }
  | { type: 'code'; language: string; code: string }
  | { type: 'tool_reference'; toolCallId: string };

// -------------------- Thinking/Reasoning --------------------

interface ThinkingMessage extends Message {
  type: 'thinking';
  content: string;           // Raw thinking text
  isVisible: boolean;        // User expanded?
  isComplete: boolean;       // Finished thinking?
  parentMessageId: MessageId; // Links to assistant message
}

// -------------------- Tool Execution --------------------

interface ToolCallMessage extends Message {
  type: 'tool_call';
  toolCallId: ToolCallId;
  toolName: string;
  arguments: Record<string, unknown>;
  status: 'pending' | 'running' | 'completed' | 'error' | 'cancelled';
  executionTimeMs?: number;
}

interface ToolResultMessage extends Message {
  type: 'tool_result';
  toolCallId: ToolCallId;    // Links to the call
  result: unknown;           // Structured result
  output: string;            // Human-readable output
  isError: boolean;
  exitCode?: number;         // For shell commands
}

// -------------------- System --------------------

interface SystemMessage extends Message {
  type: 'system';
  content: string;
  level: 'info' | 'warning' | 'error';
  action?: SystemAction;     // Optional actionable
}

interface SystemAction {
  label: string;
  action: () => void;
}

// ==================== Conversation State ====================

interface ConversationState {
  messages: Message[];
  sessionId: string;
  agentId: AgentId;
  
  // Input state
  inputValue: string;
  inputHistory: string[];
  historyIndex: number;
  isComposing: boolean;      // IME composition
  
  // Streaming state
  streamingMessageId?: MessageId;
  streamingThinkingId?: MessageId;
  
  // UI state
  scrollPosition: 'bottom' | 'sticky' | number;
  selectedMessageId?: MessageId;
  expandedToolCalls: Set<ToolCallId>;
  expandedThinking: Set<MessageId>;
  
  // Pending interactions
  pendingApproval?: ApprovalRequest;
  isLoading: boolean;
}

interface ApprovalRequest {
  id: string;
  toolCallId: ToolCallId;
  toolName: string;
  arguments: Record<string, unknown>;
  description: string;
  riskLevel: 'low' | 'medium' | 'high';
}
```

### 19.3 Component Hierarchy

```
AgentConversationView (container)
├── AgentHeader
│   ├── Breadcrumb (Project / Agent Name)
│   ├── AgentStatus (idle/thinking/running/error)
│   ├── CostIndicator (session cost)
│   └── Actions (settings, pause, stop, close)
├── ConversationPanel (main scrollable area)
│   └── Virtuoso (virtualized list)
│       └── MessageRenderer (switch on message.type)
│           ├── UserMessageItem
│           │   ├── MessageBubble
│           │   ├── AttachmentList
│           │   └── EditButton (if last message)
│           ├── AssistantMessageItem
│           │   ├── Avatar
│           │   ├── MessageContent (markdown)
│           │   ├── StreamingIndicator (if active)
│           │   └── Actions (copy, regenerate)
│           ├── ThinkingBlock
│           │   ├── CollapsibleHeader
│           │   └── ThinkingContent
│           ├── ToolCallCard
│           │   ├── ToolHeader (name + status icon)
│           │   ├── Arguments (collapsible JSON)
│           │   └── ResultPanel (when complete)
│           └── SystemMessageBanner
├── PendingApprovalBanner (sticky if present)
│   ├── ApprovalDetails
│   └── ActionButtons (approve/reject/always)
├── ContextPanel (collapsible side drawer)
│   ├── AttachedFilesList
│   ├── AddContextButton
│   └── SuggestedContext
└── InputArea (fixed bottom)
    ├── ContextPreview (mini attachments)
    ├── InputToolbar
    │   ├── AttachButton
    │   ├── ClearButton
    │   └── ModelSelector (if applicable)
    ├── AgentInput (main textarea)
    │   ├── MentionAutocomplete (popup)
    │   └── CommandAutocomplete (popup)
    └── SendButton
```

### 19.4 State Management & Data Flow

```typescript
// ==================== Store Design (Zustand) ====================

interface AgentConversationStore {
  // State
  conversation: ConversationState;
  
  // Actions
  // --- Input ---
  setInput: (value: string) => void;
  sendMessage: () => Promise<void>;
  navigateHistory: (direction: 'up' | 'down') => void;
  addAttachment: (file: File) => void;
  removeAttachment: (id: string) => void;
  
  // --- Streaming ---
  startStreaming: () => MessageId;
  appendStreamingContent: (messageId: MessageId, chunk: string) => void;
  appendThinkingContent: (thinkingId: MessageId, chunk: string) => void;
  completeStreaming: (messageId: MessageId) => void;
  
  // --- Tool Calls ---
  addToolCall: (toolCall: ToolCallMessage) => void;
  updateToolStatus: (toolCallId: ToolCallId, status: ToolStatus) => void;
  setToolResult: (toolCallId: ToolCallId, result: ToolResultMessage) => void;
  toggleToolExpanded: (toolCallId: ToolCallId) => void;
  
  // --- Approvals ---
  requestApproval: (request: ApprovalRequest) => void;
  approve: (requestId: string, rememberChoice?: boolean) => void;
  reject: (requestId: string) => void;
  
  // --- UI ---
  scrollToBottom: () => void;
  toggleThinking: (messageId: MessageId) => void;
  selectMessage: (messageId: MessageId | undefined) => void;
}

// ==================== Event Flow ====================

/*
┌─────────────────────────────────────────────────────────────┐
│  USER ACTION: Type message and press Enter                  │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────┐
│  INPUT PROCESSING                                           │
│  • Parse @mentions → resolve file paths                     │
│  • Parse /commands → extract instructions                   │
│  • Create UserMessage with attachments                      │
│  • Add to message list                                      │
│  • Clear input, save to history                             │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────┐
│  SEND TO BACKEND (Rust via Tauri)                           │
│  invoke('send_to_agent', {                                  │
│    agentId,                                                 │
│    message: parsedContent,                                  │
│    context: attachments                                     │
│  })                                                         │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────┐
│  RUST: AgentManager sends to child process (kimi --wire)    │
│  • Send JSON-RPC prompt request                             │
│  • Subscribe to stdout events                               │
└───────────────────────┬─────────────────────────────────────┘
                        │
            ┌───────────┴───────────┐
            │                       │
            ▼                       ▼
┌───────────────────┐   ┌─────────────────────────────────────┐
│  EVENT: content   │   │  EVENT: thinking                    │
│  • Append to      │   │  • Create/update ThinkingMessage    │
│    streaming msg  │   │  • Emit to frontend                 │
│  • Emit delta     │   └─────────────────────────────────────┘
└───────────────────┘
            │
            ▼
┌───────────────────┐   ┌─────────────────────────────────────┐
│  EVENT: tool_call │   │  EVENT: approval_request            │
│  • Add ToolCall   │   │  • Set pendingApproval              │
│    message        │   │  • Show banner                      │
└───────────────────┘   └─────────────────────────────────────┘
            │
            ▼
┌─────────────────────────────────────────────────────────────┐
│  EVENT: tool_result                                         │
│  • Update ToolCall status                                   │
│  • Add ToolResult message                                   │
│  • Agent continues processing                               │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────┐
│  EVENT: turn_complete                                       │
│  • Mark streaming complete                                  │
│  • Update final metadata (cost, tokens)                     │
│  • Enable input                                             │
└─────────────────────────────────────────────────────────────┘
*/
```

### 19.5 Streaming Behavior Specification

```typescript
// ==================== Streaming Implementation ====================

interface StreamingState {
  messageId: MessageId;
  buffer: string;
  lastFlushTime: number;
  flushInterval: number;  // ms
}

// Buffering strategy for smooth UI
class StreamingBuffer {
  private buffer = '';
  private flushTimer?: number;
  private readonly FLUSH_INTERVAL = 50; // ms
  
  append(chunk: string, onFlush: (text: string) => void): void {
    this.buffer += chunk;
    
    // Clear existing timer
    if (this.flushTimer) {
      clearTimeout(this.flushTimer);
    }
    
    // Flush immediately on punctuation or newlines
    if (/[.!?\n]$/.test(chunk)) {
      onFlush(this.buffer);
      this.buffer = '';
      return;
    }
    
    // Otherwise flush on interval
    this.flushTimer = window.setTimeout(() => {
      onFlush(this.buffer);
      this.buffer = '';
    }, this.FLUSH_INTERVAL);
  }
  
  flush(onFlush: (text: string) => void): void {
    if (this.buffer) {
      onFlush(this.buffer);
      this.buffer = '';
    }
    if (this.flushTimer) {
      clearTimeout(this.flushTimer);
    }
  }
}

// React component with streaming
function StreamingMessage({ messageId }: { messageId: MessageId }) {
  const [displayContent, setDisplayContent] = useState('');
  const streamingBuffer = useRef(new StreamingBuffer());
  
  useEffect(() => {
    // Subscribe to backend events via Tauri
    const unlisten = listen('agent:content_chunk', (event) => {
      if (event.payload.messageId === messageId) {
        streamingBuffer.current.append(
          event.payload.chunk,
          (text) => setDisplayContent(prev => prev + text)
        );
      }
    });
    
    return () => {
      streamingBuffer.current.flush((text) => {
        setDisplayContent(prev => prev + text);
      });
      unlisten.then(f => f());
    };
  }, [messageId]);
  
  return (
    <MarkdownContent content={displayContent} />
  );
}
```

### 19.6 Input System Deep Dive

```typescript
// ==================== Smart Input Component ====================

interface AgentInputProps {
  value: string;
  onChange: (value: string) => void;
  onSubmit: () => void;
  history: string[];
  onAttachFile: () => void;
  disabled?: boolean;
  placeholder?: string;
}

// Features:
// 1. @mention autocomplete for files
// 2. /command autocomplete
// 3. History navigation (up/down)
// 4. Multi-line (Shift+Enter)
// 5. Auto-resize height

function AgentInput(props: AgentInputProps) {
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const [suggestions, setSuggestions] = useState<Suggestion[]>([]);
  const [suggestionIndex, setSuggestionIndex] = useState(0);
  const [cursorPosition, setCursorPosition] = useState(0);
  
  // Auto-resize
  useEffect(() => {
    const el = textareaRef.current;
    if (el) {
      el.style.height = 'auto';
      el.style.height = `${Math.min(el.scrollHeight, 200)}px`;
    }
  }, [props.value]);
  
  // Handle input changes for autocomplete
  const handleInput = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const value = e.target.value;
    const pos = e.target.selectionStart;
    props.onChange(value);
    setCursorPosition(pos);
    
    // Check for @mention trigger
    const beforeCursor = value.slice(0, pos);
    const mentionMatch = beforeCursor.match(/@([^\s]*)$/);
    if (mentionMatch) {
      fetchFileSuggestions(mentionMatch[1]).then(setSuggestions);
    }
    // Check for /command trigger
    else if (beforeCursor.match(/^\/)) {
      setSuggestions(getCommandSuggestions(beforeCursor));
    }
    else {
      setSuggestions([]);
    }
  };
  
  // Handle special keys
  const handleKeyDown = (e: React.KeyboardEvent) => {
    // Submit on Enter (not Shift+Enter)
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      if (suggestions.length > 0) {
        acceptSuggestion(suggestions[suggestionIndex]);
      } else {
        props.onSubmit();
      }
      return;
    }
    
    // History navigation
    if (e.key === 'ArrowUp' && cursorPosition === 0) {
      e.preventDefault();
      navigateHistory('up');
    }
    if (e.key === 'ArrowDown' && isAtEnd()) {
      e.preventDefault();
      navigateHistory('down');
    }
    
    // Suggestion navigation
    if (suggestions.length > 0) {
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        setSuggestionIndex(i => (i + 1) % suggestions.length);
      }
      if (e.key === 'ArrowUp') {
        e.preventDefault();
        setSuggestionIndex(i => (i - 1 + suggestions.length) % suggestions.length);
      }
      if (e.key === 'Escape') {
        setSuggestions([]);
      }
    }
  };
  
  return (
    <div className="relative">
      <textarea
        ref={textareaRef}
        value={props.value}
        onChange={handleInput}
        onKeyDown={handleKeyDown}
        disabled={props.disabled}
        placeholder={props.placeholder}
        className="w-full resize-none bg-transparent border-0 focus:ring-0"
        rows={1}
      />
      
      {/* Autocomplete popup */}
      {suggestions.length > 0 && (
        <AutocompletePopup
          suggestions={suggestions}
          selectedIndex={suggestionIndex}
          onSelect={acceptSuggestion}
        />
      )}
    </div>
  );
}
```

### 19.7 Tool Call Visualization

```typescript
// ==================== Tool Call Card ====================

interface ToolCallCardProps {
  toolCall: ToolCallMessage;
  result?: ToolResultMessage;
  isExpanded: boolean;
  onToggleExpand: () => void;
}

function ToolCallCard(props: ToolCallCardProps) {
  const { toolCall, result } = props;
  const isRunning = toolCall.status === 'running';
  const isError = result?.isError ?? false;
  
  return (
    <div className={cn(
      "rounded-lg border my-2 overflow-hidden",
      isError ? "border-red-300 bg-red-50" : "border-[var(--border-soft)]"
    )}>
      {/* Header - Always visible */}
      <button 
        onClick={props.onToggleExpand}
        className="w-full flex items-center justify-between p-3 hover:bg-[var(--bone-100)]"
      >
        <div className="flex items-center gap-2">
          <ToolStatusIcon status={toolCall.status} />
          <span className="font-medium">{toolCall.toolName}</span>
          {isRunning && <Spinner size="sm" />}
        </div>
        <ChevronIcon expanded={props.isExpanded} />
      </button>
      
      {/* Expanded content */}
      {props.isExpanded && (
        <div className="border-t border-[var(--border-soft)] p-3 space-y-3">
          {/* Arguments */}
          <div>
            <h4 className="text-xs uppercase text-[var(--ink-500)] mb-1">Arguments</h4>
            <pre className="bg-[var(--bone-50)] p-2 rounded text-sm overflow-x-auto">
              <code>{JSON.stringify(toolCall.arguments, null, 2)}</code>
            </pre>
          </div>
          
          {/* Result (when complete) */}
          {result && (
            <div>
              <h4 className="text-xs uppercase text-[var(--ink-500)] mb-1">Result</h4>
              <ToolResultDisplay result={result} />
            </div>
          )}
        </div>
      )}
      
      {/* Compact preview (when collapsed and complete) */}
      {!props.isExpanded && result && !isError && (
        <div className="px-3 pb-2 text-sm text-[var(--ink-500)] truncate">
          {truncateResult(result.output, 60)}
        </div>
      )}
    </div>
  );
}

// Special renderers for common tools
function ToolResultDisplay({ result }: { result: ToolResultMessage }) {
  // File content - show with syntax highlighting
  if (isFileContent(result)) {
    return <CodeBlock code={result.output} language={detectLanguage(result)} />;
  }
  
  // Shell output - terminal style
  if (isShellOutput(result)) {
    return <TerminalOutput output={result.output} exitCode={result.exitCode} />;
  }
  
  // Error - styled error message
  if (result.isError) {
    return <ErrorDisplay message={result.output} />;
  }
  
  // Default - plain text
  return <pre className="text-sm whitespace-pre-wrap">{result.output}</pre>;
}
```

### 19.8 Scroll & Virtualization Behavior

```typescript
// ==================== Smart Scrolling ====================

interface ScrollBehavior {
  // "bottom" = auto-scroll to bottom on new content
  // "sticky" = stay at bottom if already near bottom, else free scroll
  // number = specific scroll position
  mode: 'bottom' | 'sticky' | number;
  threshold: number;  // pixels from bottom to consider "at bottom"
}

function ConversationPanel() {
  const virtuosoRef = useRef<VirtuosoHandle>(null);
  const [atBottom, setAtBottom] = useState(true);
  const [newMessagesCount, setNewMessagesCount] = useState(0);
  
  // Handle new messages
  useEffect(() => {
    if (atBottom) {
      // Auto-scroll if user is at bottom
      virtuosoRef.current?.scrollToIndex({ index: 'LAST' });
    } else {
      // Show "new messages" indicator
      setNewMessagesCount(c => c + 1);
    }
  }, [messages.length]);
  
  return (
    <div className="relative h-full">
      <Virtuoso
        ref={virtuosoRef}
        data={messages}
        followOutput={atBottom ? 'auto' : false}
        atBottomStateChange={setAtBottom}
        itemContent={(index, message) => (
          <MessageRenderer message={message} />
        )}
      />
      
      {/* New messages indicator */}
      {!atBottom && newMessagesCount > 0 && (
        <button
          onClick={() => {
            virtuosoRef.current?.scrollToIndex({ index: 'LAST' });
            setNewMessagesCount(0);
          }}
          className="absolute bottom-4 left-1/2 -translate-x-1/2 
                     bg-[var(--accent-500)] text-white px-4 py-2 rounded-full
                     shadow-lg hover:bg-[var(--accent-600)]"
        >
          {newMessagesCount} new message{newMessagesCount !== 1 ? 's' : ''}
        </button>
      )}
    </div>
  );
}
```

### 19.9 Library Recommendations

| Purpose | Library | Reason |
|---------|---------|--------|
| **Virtualization** | `react-virtuoso` | Dynamic heights, smooth scroll, follow output |
| **Markdown** | `react-markdown` + `remark-gfm` | Extensible, tree-sitter support |
| **Syntax Highlight** | `shiki` | VS Code themes, accurate, lightweight |
| **Styling** | Tailwind + CSS Variables | Matches existing design system |
| **Collapsible** | `@radix-ui/react-collapsible` | Accessible, unstyled |
| **Auto-resize** | Custom hook | Simple textarea measurement |

---

## Technical Stack Recommendations

### Frontend
- **Framework:** React + TypeScript
- **State Management:** Zustand or Redux Toolkit
- **UI Components:** Radix UI or Chakra UI
- **Editor:** Monaco Editor (VS Code's editor)
- **Charts:** Recharts for cost analytics

### Backend (if needed for HTTP tool server)
- **Runtime:** Node.js with Express or Fastify
- **Database:** SQLite (local) or PostgreSQL (team)
- **ORM:** Prisma

### AI Integration
- **HTTP Clients:** Native fetch or axios
- **Streaming:** Native ReadableStream
- **Token Counting:** tiktoken (OpenAI) or provider APIs

### Storage
- **Documents:** Local filesystem with JSON metadata
- **Config:** Electron-store or similar
- **Cost Logs:** SQLite for queryability

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| API rate limits | Implement queuing with backoff |
| Context window overflow | Automatic summarization and trimming |
| Cost overruns | Hard budget caps with pre-call checks |
| Agent loops/deadlocks | Timeout and max iteration limits |
| Poor quality from cheap models | Review gates and feedback loops |
| Provider outages | Multi-provider fallback chain |

---

## Success Metrics

- **Cost Efficiency:** 60%+ cost reduction vs. using premium models for everything
- **Quality Maintenance:** <10% regression in output quality
- **Developer Velocity:** 2x faster from idea to implementation
- **Agent Utilization:** >80% of tasks delegated without human intervention
- **Review Accuracy:** >90% of agent reviews match human judgment
