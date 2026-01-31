# AIHarness Development TODO

> **Last Updated:** 2026-01-31  
> **Status:** Planning Phase — Architecture & Design  
> **Stack:** Tauri (Rust + React/TypeScript)  
> **Vision:** Complete AI Control Center — Your CLI Replacement + Proactive AI Companion

---

## Legend

- [ ] Not started
- [~] In progress
- [x] Complete
- [?] Needs decision/clarification
- [!] Blocked

---

## Phase 0: Foundation & Planning

### Architecture Decisions
- [x] Choose tech stack: **Tauri + React + Rust**
- [ ] Finalize backend architecture (modules, crates structure)
- [ ] Define frontend component hierarchy
- [ ] Design database schema (SQLite tables)
- [ ] Define IPC interface (Tauri commands/events)
- [ ] Setup CI/CD (GitHub Actions)
- [ ] Write architecture ADRs (Architecture Decision Records)

### Project Setup
- [ ] Initialize Tauri project with React template
- [ ] Setup Rust workspace structure (multi-crate if needed)
- [ ] Configure ESLint, Prettier, TypeScript strict mode
- [ ] Setup Rust clippy, rustfmt
- [ ] Add pre-commit hooks
- [ ] Create development documentation (BUILD.md)

---

## Phase 1: Core Infrastructure

### Data Layer (Rust)
- [ ] Define core data structures (serde Serialize/Deserialize)
  - [ ] `Project` struct
  - [ ] `Document` struct with versioning
  - [ ] `Agent` struct with session management
  - [ ] `Task` struct with workflow state
  - [ ] `CostLog` entry
  - [ ] `ExpertPanel` and related structs
- [ ] Setup SQLite database (rusqlite)
  - [ ] Migration system (refinery or similar)
  - [ ] Connection pooling
- [ ] File system abstraction
  - [ ] Project directory structure
  - [ ] Document versioning (Git-like or simple)
  - [ ] Backup/recovery

### Configuration
- [ ] App configuration (settings.toml)
- [ ] Provider API keys (secure storage via keychain)
- [ ] Project configuration (aiharness.yaml)
- [ ] Default templates and prompts

---

## Phase 2: AI Provider Integration

### Provider Abstraction
- [ ] Define `LLMProvider` trait
- [ ] Provider implementations:
  - [ ] OpenAI (GPT-4, GPT-4o-mini, etc.)
  - [ ] Anthropic (Claude 3 Haiku/Sonnet/Opus)
  - [ ] Ollama (local models)
  - [ ] Google Gemini (optional)
  - [ ] xAI Grok (optional)
- [ ] Streaming support (Server-Sent Events)
- [ ] Token counting (tiktoken-rs or provider APIs)
- [ ] Rate limiting and retry logic

### Cost Tracking
- [ ] Per-call cost calculation
- [ ] Cost aggregation (per agent, per task, per project)
- [ ] Budget enforcement (pre-call checks)
- [ ] Cost alert system
- [ ] Export to CSV/JSON

---

## Phase 3: Document & Context Management

### Backend (Rust)
- [ ] Document CRUD operations
- [ ] Markdown with YAML frontmatter parsing
- [ ] Document tagging and search
- [ ] Document tree hierarchy
- [ ] Version history (diff storage)
- [ ] Block-level references (for quoting context)

### Frontend (React)
- [ ] Document tree view (react-arborist)
- [ ] Markdown editor (Monaco with markdown language server)
- [ ] Split view: editor + preview
- [ ] Tag management UI
- [ ] Document search (full-text)
- [ ] Drag-and-drop reordering

---

## Phase 4: Agent System

### Agent Session Management
- [ ] Agent lifecycle (spawn, pause, resume, terminate)
- [ ] Context window management
  - [ ] Token counting per conversation
  - [ ] Automatic summarization when approaching limit
  - [ ] Conversation history pruning
- [ ] Agent state persistence
- [ ] Agent-to-agent messaging (if needed)

### Agent Roles
- [ ] Role definition system
  - [ ] Architect (high-level design)
  - [ ] Implementer (code writing)
  - [ ] Reviewer (code review)
  - [ ] Tester (test generation)
- [ ] System prompt templates
- [ ] Role-specific tool access

### Agent Dashboard UI
- [ ] Agent list with status indicators
- [ ] Real-time agent activity log
- [ ] Context viewer (what does agent know?)
- [ ] Cost per agent display
- [ ] Kill/pause/resume controls

---

## Phase 5: Task System & Workflows

### Task Management
- [ ] Task CRUD operations
- [ ] Dependency tracking
- [ ] Task queue with priority
- [ ] Task assignment to agents
- [ ] Task status workflow engine

### Delegation Workflows
- [ ] Workflow definition system
- [ ] Built-in workflows:
  - [ ] Architecture → Spec → Implementation → Review
  - [ ] Code Review (agent reviews agent)
  - [ ] Bug Fix (find → fix → verify)
- [ ] Approval gates (human-in-the-loop)
- [ ] Automatic handoff between stages
- [ ] Context passing between agents

### Task Board UI
- [ ] Kanban-style board
- [ ] Task detail view
- [ ] Dependency visualization
- [ ] Quick actions (assign, approve, reject)

---

## Phase 6: Expert Panel System ⭐ KEY FEATURE

### Backend
- [ ] Panel configuration management
- [ ] Parallel query execution (tokio::join!)
- [ ] Response collection and storage
- [ ] Semantic similarity analysis
  - [ ] Text embedding (local model or API)
  - [ ] Clustering/agreement detection
- [ ] Consensus scoring algorithm
- [ ] Cost aggregation per panel

### Panel Modes
- [ ] **Poll Mode:** Simple parallel query
- [ ] **Debate Mode:** Round-robin critique
  - [ ] Opening statements
  - [ ] Critique round
  - [ ] Revision round
  - [ ] Final position
- [ ] **Synthesis Mode:** Merge responses into unified answer

### Frontend
- [ ] Panel configuration UI
  - [ ] Model selection (checkboxes)
  - [ ] Role assignment per model
  - [ ] Budget setting
- [ ] Side-by-side response comparison
  - [ ] Synchronized scrolling
  - [ ] Diff highlighting
  - [ ] Vote/agreement indicators
- [ ] Consensus visualization
  - [ ] Agreement heatmap
  - [ ] Disagreement topics
- [ ] Cost breakdown per model
- [ ] Export results (markdown, PDF)

---

## Phase 7: Code Architecture & Diagramming

### Diagram Editor (React Flow)
- [ ] Canvas with zoom/pan
- [ ] Node types:
  - [ ] Service/Module
  - [ ] API Endpoint
  - [ ] Database
  - [ ] Event/Queue
  - [ ] External System
- [ ] Edge types:
  - [ ] Calls (sync)
  - [ ] Events (async)
  - [ ] Depends on
- [ ] Node property editor (sidebar)
- [ ] Auto-layout algorithms
- [ ] Save/load diagrams

### AI-Powered Diagramming
- [ ] Generate diagram from code analysis
- [ ] Generate diagram from description
- [ ] Update diagram based on code changes
- [ ] Export to various formats (PNG, SVG, PlantUML)

---

## Phase 8: Prompt Library

### Backend
- [ ] Prompt CRUD
- [ ] Variable substitution system
- [ ] Prompt versioning
- [ ] Usage analytics

### Frontend
- [ ] Prompt browser
- [ ] Prompt editor with variable preview
- [ ] Quick-insert from editor
- [ ] Prompt templates gallery

---

## Phase 9: Cost Optimization & Analytics

### Backend
- [ ] Cost-aware routing engine
- [ ] Routing rules system
- [ ] Model performance tracking
- [ ] Adaptive routing (learn from quality feedback)

### Frontend
- [ ] Real-time cost dashboard
- [ ] Budget alerts
- [ ] Cost per feature analysis
- [ ] Model comparison reports
- [ ] Cost forecasting

---

## Phase 10: Advanced Features

### Agent-to-Agent Review
- [ ] Review assignment system
- [ ] Review feedback structure
- [ ] Dispute escalation to human
- [ ] Review quality metrics

### Results Aggregation
- [ ] Multi-agent consensus building
- [ ] Output merging strategies
- [ ] Confidence scoring

### MCP/Tool Server Mode
- [ ] REST API server (optional, embedded)
- [ ] Authentication
- [ ] External AI query interface

### Sandboxed Content Access
- [ ] Directory registration
- [ ] Snapshot system
- [ ] Safe query APIs

---

## Phase 11: Chat System & CLI Parity ⭐ NEW

### Vision
Full replacement for AI CLI tools (Claude Code, Aider, etc.) — fork conversations, time travel, complete tool use.

### Chat Session Management
- [ ] Conversation persistence (SQLite)
- [ ] Chat branching/forking
  - [ ] Fork at any message
  - [ ] Multiple conversation branches
  - [ ] Branch comparison (diff view)
- [ ] Time travel navigation
  - [ ] Browse full conversation history
  - [ ] Jump to any point in conversation
  - [ ] "What if" scenarios (rewind and retry)
- [ ] Chat search (full-text across all conversations)
- [ ] Chat export (Markdown, JSON, shareable links)
- [ ] Chat templates (save conversation starters)

### Tool Use Integration
- [ ] Shell command execution
  - [ ] User approval for destructive commands
  - [ ] Command output capture and display
  - [ ] Working directory context
- [ ] File system tools
  - [ ] Read/write files
  - [ ] List directories
  - [ ] Search files (grep)
- [ ] Code tools
  - [ ] View code with line numbers
  - [ ] Search/replace in files
  - [ ] Apply patches/diffs
- [ ] Git integration
  - [ ] Status, diff, commit
  - [ ] Branch operations
  - [ ] View commit history
- [ ] Web tools
  - [ ] Fetch URLs
  - [ ] Search (DuckDuckGo, etc.)

### Chat UI
- [ ] Threaded message view
- [ ] Code block syntax highlighting
- [ ] Tool call visualization (expandable cards)
- [ ] Message editing (by user and AI)
- [ ] Typing indicators
- [ ] Message reactions/feedback

---

## Phase 12: Real-Time Collaboration UI ⭐ NEW

### AI Todo List Visibility
- [ ] Real-time task progress display
  - [ ] Current task being worked on
  - [ ] Subtask breakdown
  - [ ] Progress bars/percentages
  - [ ] Time estimates vs. actual
- [ ] Todo list in sidebar (always visible)
- [ ] Task status updates (streaming)
- [ ] Blocked task indicators with reason
- [ ] Completed task celebration 🎉

### Editor Integration (Shared Presence)
- [ ] AI opens file → opens as tab automatically
- [ ] AI navigates to location → scrolls to that position
- [ ] Visual indicator of AI's "cursor" (ghost cursor)
- [ ] AI edits appear in real-time (like Google Docs)
- [ ] User can "follow" AI or work independently
- [ ] Conflict resolution when both editing
- [ ] AI change preview (accept/reject per change)

### Activity Feed
- [ ] Stream of AI actions
- [ ] What files touched, what commands run
- [ ] Cost accumulation in real-time
- [ ] Filterable by agent/task/type

---

## Phase 13: Scheduling System ⭐ NEW

### Scheduled Prompts
- [ ] Cron-like scheduling interface
- [ ] One-time scheduled prompts
- [ ] Recurring prompts (daily standup, weekly review)
- [ ] Prompt templates with variables (date, project state)
- [ ] Schedule management UI (calendar view)
- [ ] Notification system when scheduled prompt runs
- [ ] Execution history

### AI Self-Scheduling
- [ ] AI can schedule future prompts for itself
  - [ ] "Check back on this tomorrow"
  - [ ] "Review progress in 2 hours"
- [ ] Scheduled task dependencies
  - [ ] "After task X completes, schedule review"
- [ ] Conditional scheduling
  - [ ] "If tests fail, retry in 10 minutes"
- [ ] User approval for AI-scheduled items (optional gate)

### Background Execution
- [ ] Scheduler daemon (always running)
- [ ] Queue management
- [ ] Retry logic for failed scheduled runs
- [ ] Resource management (don't run if already busy)

---

## Phase 14: Heartbeat System ⭐ NEW ⭐ EXPERIMENTAL

### Vision
A "living" AI companion that runs on a tunable interval, proactively suggesting, monitoring, and enhancing your workflow.

### Core Heartbeat
- [ ] Configurable heartbeat interval (30s, 1m, 5m, etc.)
- [ ] Background context gathering
  - [ ] Current file state
  - [ ] Recent activity
  - [ ] Time of day, day of week
  - [ ] Music playing (system integration)
  - [ ] Git state
  - [ ] Error logs (recent failures)
- [ ] Heartbeat prompt template
  - [ ] Context-aware suggestions
  - [ ] Non-intrusive (sidebar notification, not modal)
- [ ] Smart triggering (only when relevant)
  - [ ] Don't interrupt deep work
  - [ ] Detect idle time vs. active coding

### Proactive Suggestions
- [ ] Code suggestions based on patterns
- [ ] "You might want to..." based on context
- [ ] Reminders ("You said you'd review this")
- [ ] Related file suggestions
- [ ] Documentation gaps detected
- [ ] Test coverage suggestions

### Ambient Features (Experimental)
- [ ] Music integration (control Spotify/Apple Music)
  - [ ] "Focus mode" playlist
  - [ ] Music suggestions based on task type
  - [ ] Pause when AI needs attention
- [ ] Pomodoro-style work/break reminders
- [ ] Posture/stretch reminders (gentle)
- [ ] Daily standup summary generation
- [ ] End-of-day commit message suggestions

### Personality & Voice
- [ ] Configurable assistant personality
  - [ ] Professional, casual, playful, terse
- [ ] Voice/notification sounds (optional)
- [ ] Greeting on startup (contextual)

### Safety & Controls
- [ ] Easy pause/resume heartbeat
- [ ] Do Not Disturb mode
- [ ] Cost tracking (heartbeats add up!)
- [ ] Transparency: log every heartbeat decision

---

## Phase 15: Tool System & Plugins ⭐ NEW

### Tool Visibility & Control
- [ ] Tool output streaming to UI
- [ ] Tool execution status tracking
- [ ] Exit code and diagnostics display
- [ ] Tool performance metrics
- [ ] Tool execution history

### Plugin Architecture
- [ ] WASM plugin system
- [ ] Python script plugins
- [ ] Native Rust plugin loading
- [ ] Plugin manifest format
- [ ] Permission system for plugins
- [ ] Hot-reload for development

### Token-Optimized Output
- [ ] Smart truncation with preservation
- [ ] Error extraction from output
- [ ] Build result summarization
- [ ] Directory listing optimization
- [ ] "Show more" for full output

---

## Phase 16: Python Embedded ⭐ NEW

### Python Runtime
- [ ] Embed Python interpreter (PyO3)
- [ ] Jupyter-like cell execution
- [ ] Package management (pip, virtualenv)
- [ ] Security sandboxing

### Integration
- [ ] Rust ↔ Python data exchange
- [ ] AI can execute Python code
- [ ] Python output in chat
- [ ] Shared memory for large data

---

## Phase 17: Multi-Project Workspace ⭐ NEW

### Project Management
- [ ] Multiple open projects
- [ ] Fast project switching
- [ ] Project tabs/sidebar
- [ ] Independent window option

### Context Preservation
- [ ] Per-project open files
- [ ] Per-project AI todos
- [ ] Per-project conversations
- [ ] Project-specific agents

### Global Resources
- [ ] Shared prompt library
- [ ] Global agents
- [ ] Cross-project search

---

## Phase 18: Bug Tracking ⭐ NEW

### Lightweight Issues
- [ ] Quick capture (keybinding)
  - [ ] Auto-capture errors from tools
  - [ ] Link to conversation/code
  - [ ] AI triage and suggestions
- [ ] Minimal workflow (open → closed)
- [ ] Duplicate detection

---

## Phase 19: Model Provider Support ⭐ NEW

### Providers
- [ ] OpenAI (GPT-4, GPT-4o, etc.)
- [ ] Anthropic (Claude 3 family)
- [ ] Google (Gemini)
- [ ] Moonshot AI (Kimi)
- [ ] Local models (Ollama)
- [ ] xAI (Grok)

### Authentication
- [ ] API key storage (keychain)
- [ ] OAuth flows
- [ ] Multi-account per provider
- [ ] Environment variable fallback

### Features
- [ ] Model discovery
- [ ] Per-model token counting
- [ ] Rate limit tracking
- [ ] Cost per model

---

## Phase 20: Token Optimization ⭐ NEW

### System-Wide Optimization
- [ ] Token budget enforcement
- [ ] Cost estimation before send
- [ ] Context compression
- [ ] Rolling conversation summarization
- [ ] Lazy file loading

### Tool Output
- [ ] Smart truncation
- [ ] Error extraction
- [ ] Build summary

### Code Context
- [ ] Symbol-level references
- [ ] Diff-based updates
- [ ] Import tracking

---

## Phase 21: Client/Server Mode ⭐ NEW

### Server Architecture
- [ ] Headless server binary
- [ ] WebSocket server implementation
- [ ] gRPC service layer
- [ ] Session persistence
- [ ] Multi-client session sharing

### Client Architecture
- [ ] Thin client mode
- [ ] Server discovery (mDNS)
- [ ] Connection management UI
- [ ] Reconnect/resume logic
- [ ] Offline command queue

### Communication
- [ ] WebSocket bidirectional streaming
- [ ] gRPC command protocol
- [ ] File synchronization
- [ ] TLS encryption
- [ ] Authentication

---

## Polish & Release

### Testing
- [ ] Unit tests (Rust + Vitest for React)
- [ ] Integration tests
- [ ] End-to-end tests (Playwright)
- [ ] Performance benchmarks

### Documentation
- [ ] User guide
- [ ] API documentation
- [ ] Video tutorials
- [ ] Example projects

### Distribution
- [ ] macOS app bundle
- [ ] Windows installer
- [ ] Linux AppImage/deb
- [ ] Auto-updater

---

## Current Priorities (Next 2 Weeks)

1. [ ] Finalize architecture & scaffold project
2. [ ] Implement basic document management
3. [ ] Integrate one LLM provider (OpenAI or Anthropic)
4. [ ] Build simple agent execution (single agent, single task)
5. [ ] Basic React Flow diagram canvas

---

## Ideas / Future

- [ ] Team collaboration (multi-user, sync)
- [ ] Plugin system (WASM extensions)
- [ ] Mobile companion app
- [ ] Cloud sync option
- [ ] Marketplace for agent roles
- [ ] Voice input for prompts
- [ ] AI-generated documentation from code

---

## Questions / Decisions Needed

- [?] Use SQLite or sled (pure Rust)?
- [?] Use tauri-specta for type-safe IPC?
- [?] Monaco or CodeMirror for editor?
- [?] React Query or Zustand for state management?
- [?] How to handle large context documents (chunking)?
- [?] Should we support real-time collaboration (CRDTs)?

---

## How to Update This File

When completing a task:
1. Change `[ ]` to `[x]`
2. Add date in comment if significant
3. Move to "Complete" section if needed

When adding tasks:
1. Add to appropriate section
2. Use priority markers: 🔴 High, 🟡 Medium, 🟢 Low

---

## Completed

- [x] 2026-01-31: Initial project vision and planning documents
- [x] 2026-01-31: Expert Panel feature specification
- [x] 2026-01-31: Tech stack decision: Tauri + React + Rust
