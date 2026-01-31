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

### 5.1 MCP/Tool Server Mode
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

## Technical Stack Recommendations

### Frontend
- **Framework:** React + TypeScript
- **State Management:** Zustand or Redux Toolkit
- **UI Components:** Radix UI or Chakra UI
- **Editor:** Monaco Editor (VS Code's editor)
- **Charts:** Recharts for cost analytics

### Backend (if needed for MCP server)
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
