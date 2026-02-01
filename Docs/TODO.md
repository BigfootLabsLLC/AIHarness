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

## Version 0.1 — "The Bridge" (HTTP Tool Server) — DETAILED BREAKDOWN

### Overview
**Goal:** Built-in HTTP tool server that external AIs can connect to. No built-in chat yet.

**Success Criteria:** External AI connects to the built-in HTTP server and reads files via tool calls.

---

### Step 1: Project Scaffolding

#### 1.1 Initialize Tauri Project
- [ ] Run `npm create tauri-app@latest aiharness`
  - [ ] Choose: React + TypeScript
  - [ ] Choose: Cargo (Rust package manager)
- [ ] Verify `cargo tauri dev` launches successfully
- [ ] Verify React "Hello Vite + React" renders
- [ ] **Tests:**
  - [ ] App launches without panics
  - [ ] Dev server responds
  - [ ] Hot reload works

#### 1.2 Configure Development Environment
- [ ] Add `.gitignore` (Rust + Node standard)
- [ ] Configure `rustfmt.toml`
- [ ] Configure `.vscode/settings.json` (if using VS Code)
- [ ] Add `clippy.toml` for lints
- [ ] **Tests:**
  - [ ] `cargo fmt` runs
  - [ ] `cargo clippy` passes with no warnings
  - [ ] `cargo build` succeeds

#### 1.3 Setup Frontend Tooling
- [ ] Install Tailwind CSS
- [ ] Configure Tailwind with Vite
- [ ] Install Zustand (state management)
- [ ] Install TanStack Query (data fetching)
- [ ] Add Vitest for testing
- [ ] **Tests:**
  - [ ] Tailwind classes work
  - [ ] Vitest runs
  - [ ] Example test passes

---

### Step 2: Core Data Types (Rust)

#### 2.1 Define Core Structs
- [ ] `ToolCall` struct (name, arguments)
- [ ] `ToolResult` struct (success/error, output)
- [ ] `ContextFile` struct (path, content hash, last read)
- [ ] `ToolApiRequest` / `ToolApiResponse` types (HTTP payloads)
- [ ] **Tests (5+ each):**
  - [ ] Serialization/deserialization
  - [ ] Validation (empty fields, invalid data)
  - [ ] Edge cases (very long paths, unicode)
  - [ ] JSON roundtrip
  - [ ] Clone/copy behavior

#### 2.2 Error Types
- [ ] `ApiError` enum (IoError, InvalidRequest, ToolNotFound, etc.)
- [ ] `ToolError` enum (FileNotFound, PermissionDenied, etc.)
- [ ] Implement `std::error::Error` trait
- [ ] **Tests (5+ each):**
  - [ ] Error creation
  - [ ] Error display messages
  - [ ] Error conversion
  - [ ] Error propagation
  - [ ] JSON serialization of errors

---

### Step 3: Tool System (Rust)

#### 3.1 Tool Trait Definition
- [ ] Define `Tool` trait
  - [ ] `name()` method
  - [ ] `description()` method  
  - [ ] `execute(args: Value) -> Result<ToolResult, ToolError>` method
- [ ] **Tests:**
  - [ ] Trait object creation
  - [ ] Method dispatch
  - [ ] Error handling
  - [ ] Multiple tool implementations
  - [ ] Trait bounds

#### 3.2 File Tools
- [ ] `ReadFile` tool
  - [ ] Read file content
  - [ ] Handle errors (not found, permission denied)
  - [ ] Size limits (don't read 1GB files)
  - [ ] **Tests (5+):**
    - [ ] Read existing file
    - [ ] File not found error
    - [ ] Permission denied error
    - [ ] Empty file
    - [ ] Large file truncation
    - [ ] Unicode content
    - [ ] Binary file detection
- [ ] `WriteFile` tool
  - [ ] Write content to file
  - [ ] Create directories if needed
  - [ ] Atomic writes (write temp, then rename)
  - [ ] **Tests (5+):**
    - [ ] Write new file
    - [ ] Overwrite existing file
    - [ ] Create nested directories
    - [ ] Permission denied
    - [ ] Invalid path characters
    - [ ] Atomic write verification
    - [ ] Concurrent write safety
- [ ] `ListDirectory` tool
  - [ ] List files in directory
  - [ ] Recursive option
  - [ ] Filter by extension
  - [ ] **Tests (5+):**
    - [ ] List flat directory
    - [ ] List recursively
    - [ ] Empty directory
    - [ ] Directory not found
    - [ ] Filter by extension
    - [ ] Hidden files option
    - [ ] Large directory pagination
- [ ] `SearchFiles` tool
  - [ ] Grep-like search
  - [ ] Regex support
  - [ ] Result limit
  - [ ] **Tests (5+):**
    - [ ] Simple string search
    - [ ] Regex search
    - [ ] No matches
    - [ ] Invalid regex
    - [ ] Result limit respected
    - [ ] Binary file skipping
    - [ ] Case insensitive option

#### 3.3 Tool Registry
- [ ] `ToolRegistry` struct
  - [ ] Register tools by name
  - [ ] Lookup tools
  - [ ] List available tools
- [ ] **Tests (5+):**
  - [ ] Register tool
  - [ ] Lookup existing tool
  - [ ] Lookup missing tool
  - [ ] List all tools
  - [ ] Duplicate registration handling
  - [ ] Thread-safe registration

---

### Step 4: HTTP Tool API

#### 4.1 HTTP Server Core
- [ ] HTTP server module
  - [ ] Initialize with tool registry and context store
  - [ ] Start/stop server
  - [ ] Shared app state
- [ ] Request parsing (JSON body)
- [ ] Response formatting
- [ ] **Tests (5+):**
  - [ ] Server initialization
  - [ ] Valid request handling
  - [ ] Invalid JSON handling
  - [ ] Unknown tool handling
  - [ ] Response format correctness
  - [ ] Concurrent request handling

#### 4.2 HTTP Endpoints
- [ ] `GET /tools` (return available tools)
- [ ] `POST /call` (execute a tool)
- [ ] `GET /events` (return tool call history)
- [ ] `GET /events/stream` (optional SSE)
- [ ] **Tests (5+ per endpoint):**
  - [ ] Valid request
  - [ ] Missing parameters
  - [ ] Invalid parameters
  - [ ] Tool execution success
  - [ ] Tool execution failure
  - [ ] Resource not found

---

### Step 5: Context Management

#### 5.1 Context Store
- [ ] `ContextStore` struct
  - [ ] Add file to context
  - [ ] Remove file from context
  - [ ] List context files
  - [ ] Persist to SQLite
- [ ] **Tests (5+):**
  - [ ] Add file
  - [ ] Remove file
  - [ ] List files
  - [ ] Persistence across restarts
  - [ ] Duplicate handling
  - [ ] Non-existent file handling

#### 5.2 SQLite Schema
- [ ] `context_files` table
  - [ ] id, path, content_hash, added_at
- [ ] `tool_calls` table (logging)
  - [ ] id, tool_name, arguments, result, timestamp
- [ ] Migration system
- [ ] **Tests (5+):**
  - [ ] Table creation
  - [ ] Insert/select
  - [ ] Migration up/down
  - [ ] Concurrent access
  - [ ] Error handling

---

### Step 6: Tauri Integration

#### 6.1 Tauri Commands
- [ ] `start_server` command
- [ ] `stop_server` command
- [ ] `get_server_status` command
- [ ] `get_event_history` command
- [ ] **Tests (5+ per command):**
  - [ ] Command success
  - [ ] Command failure
  - [ ] State transitions
  - [ ] Event emission
  - [ ] Error propagation to frontend

#### 6.2 State Management
- [ ] Global server state (running/stopped)
- [ ] Tool call history (in-memory cache + DB)
- [ ] Context file list
- [ ] **Tests (5+):**
  - [ ] State transitions
  - [ ] Concurrent access
  - [ ] Persistence
  - [ ] Event notifications
  - [ ] Resource cleanup

#### 6.3 Events (Backend → Frontend)
- [ ] `tool_call_executed` event
- [ ] `server_status_changed` event
- [ ] `context_updated` event
- [ ] **Tests (5+):**
  - [ ] Event emission
  - [ ] Event payload
  - [ ] Multiple listeners
  - [ ] Event ordering
  - [ ] Event filtering

---

### Step 7: React UI

#### 7.1 Layout Components
- [ ] `App` component (main layout)
- [ ] `Sidebar` component (navigation)
- [ ] `MainPanel` component (content area)
- [ ] **Tests (5+ each):**
  - [ ] Renders without errors
  - [ ] Props handled correctly
  - [ ] State changes
  - [ ] Event handlers
  - [ ] Accessibility (aria labels, etc.)

#### 7.2 Server Status UI
- [ ] `ServerStatus` component
  - [ ] Show running/stopped state
  - [ ] Start/Stop buttons
  - [ ] Port display
- [ ] **Tests (5+):**
  - [ ] Displays correct status
  - [ ] Button click handlers
  - [ ] Loading states
  - [ ] Error display
  - [ ] State updates from backend

#### 7.3 Tool Call Log
- [ ] `ToolCallLog` component
  - [ ] List recent tool calls
  - [ ] Show tool name, arguments, result
  - [ ] Expand for full details
- [ ] **Tests (5+):**
  - [ ] Renders list
  - [ ] Empty state
  - [ ] Expand/collapse
  - [ ] Real-time updates
  - [ ] Large list performance

#### 7.4 Context Management UI
- [ ] `ContextPanel` component
  - [ ] List context files
  - [ ] Add file button
  - [ ] Remove file button
- [ ] `AddFileDialog` component
  - [ ] File picker
  - [ ] Path validation
- [ ] **Tests (5+ each):**
  - [ ] Renders file list
  - [ ] Add file flow
  - [ ] Remove file flow
  - [ ] Error handling
  - [ ] Real-time sync

#### 7.5 State Management (Zustand)
- [ ] `useServerStore` (server status, port, etc.)
- [ ] `useToolCallStore` (tool call history)
- [ ] `useContextStore` (context files)
- [ ] **Tests (5+ per store):**
  - [ ] Initial state
  - [ ] State updates
  - [ ] Selectors
  - [ ] Actions
  - [ ] Persistence

---

### Step 8: Testing & Integration

#### 8.1 End-to-End Tests
- [ ] "Server starts and stops"
- [ ] "Tool call flows through system"
- [ ] "Context file appears in UI"
- [ ] **Tests (5+):**
  - [ ] Full user flow
  - [ ] Error recovery
  - [ ] Concurrent operations
  - [ ] Restart behavior
  - [ ] Resource cleanup

#### 8.2 HTTP Integration Test
- [ ] Start the built-in HTTP server
- [ ] Send HTTP requests to endpoints
- [ ] Verify responses
- [ ] **Tests (5+):**
  - [ ] List tools
  - [ ] Call read_file
  - [ ] Error handling
  - [ ] Concurrent requests
  - [ ] Event history

#### 8.3 Performance Tests
- [ ] Large file read (10MB)
- [ ] Many concurrent tool calls (100)
- [ ] Large directory listing (10k files)
- [ ] **Tests (5+):**
  - [ ] Response time < 100ms for small files
  - [ ] Memory usage stays bounded
  - [ ] No memory leaks
  - [ ] Graceful degradation
  - [ ] Recovery from overload

---

### Step 9: Documentation & Packaging

#### 9.1 Documentation
- [ ] README.md (setup, build, run)
- [ ] HTTP API usage example
- [ ] API documentation (HTTP endpoints)
- [ ] Architecture diagram

#### 9.2 Build & Package
- [ ] `cargo build --release` works
- [ ] Tauri app bundles correctly
- [ ] **Tests:**
  - [ ] Release build succeeds
  - [ ] Binary runs on target platform
  - [ ] No debug artifacts in release

---

### V0.1 Definition of Done Checklist

- [ ] `cargo tauri dev` launches app
- [ ] React UI shows server status
- [ ] Click "Start Server" → server starts on localhost
- [ ] External AI can call HTTP endpoint to read file
- [ ] Tool call appears in UI log
- [ ] Add file to context via UI → external AI can access it
- [ ] All tests pass (100% of written tests)
- [ ] Documentation complete

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

### HTTP Tool Server Mode
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

**Principle:** LAN-only, no port forwarding, no external exposure.

### Server Architecture
- [ ] Headless server binary
- [ ] Bind to local network interfaces only (192.168.x.x, 10.x.x.x)
- [ ] WebSocket server (internal network only)
- [ ] gRPC service layer
- [ ] Session persistence
- [ ] Multi-client session sharing

### Client Architecture
- [ ] Thin client mode
- [ ] Server discovery (mDNS on local network)
- [ ] Connection management UI
- [ ] Reconnect/resume logic (same network)
- [ ] Offline command queue

### Communication
- [ ] WebSocket bidirectional streaming
- [ ] gRPC command protocol
- [ ] File synchronization
- [ ] TLS encryption (even on LAN)
- [ ] Authentication (tokens, not passwords)

### Security & Networking
- [ ] **LAN-only by default** - no port forwarding required
- [ ] **No external exposure** - firewall friendly
- [ ] **Future: Remote via relay/VPN** - Tailscale, WireGuard, or user-controlled relay
- [ ] **No direct internet exposure** - never open ports to public internet

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
