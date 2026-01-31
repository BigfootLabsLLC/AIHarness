# AIContextOrganizer Planning

## 1. Context
- **Purpose:** Build an app for organizing prompts, context, and collaborating with local AI tools per *HighLevel.md*.
- **Scope:** No coding today—focus on planning the feature set, implementation steps, and next-actions documentation.
- **Constraints:** Local MCP/tools integration, prompt library, AI-assisted code reviews, architecture planning features.
- **Key Paradigm:** Multi-agent orchestration with cost-aware routing - premium models for architecture/guidance, cheap models for implementation.

## 2. Detailed Feature List

### 2.1 Context Library Management
- Markdown-based context repository with tagging, versioning, and quick editing controls.
- Block selection and copy-to-clipboard controls for sharing with AIs.
- Folder tree view with drag-and-drop rearrangement capabilities.

### 2.2 Projects & Global Library
- Projects organize collections of prompts, docs, AI plans, and architecture notes with focused metadata.
- Shared global library for curated items that can be promoted from any project and reused elsewhere.
- Quick move/promotion UI so items flow from project to shared library with provenance tracking.

### 2.3 Graphical Workspace Layout
- Tree view rail on the side showing prompts, markdown files, projects, and AI tools.
- Center presentation canvas for editing, previewing, or running prompt/playbook simulations.
- Optional right rail for AI interactions, todo tracking, and architecture references.
- Shared UI modules (document tree, prompt editor, AI rails) to keep the experience consistent and extensible.

### 2.4 Local MCP/Tool Interface
- Interface to register local tools (MCPs) and expose structured documents via APIs/IPC.
- Panel for assigning documents/blocks to tools with metadata (source, format, freshness).
- Access controls so AI assistants request data by name/ID rather than copying manually.
- Support for major LLM providers (Codex, Grok, Claude, Gemini) via their native authentication/credential flows.
- Integration with local/remote Ollama instances, managing startup/shutdown and exposing connectors for hosted models.

### 2.5 Prompt Library
- Prompt catalog with categories, descriptions, sample outcomes.
- Templates that merge context snippets, e.g., standard code-review prompt + selected diff.
- Prompt version history and reuse analytics (most used, success feedback).

### 2.6 AI Code Review & Planning Workspace
- Workspace for listing code changes, intended outcomes, and discussion threads with AI.
- Integrations with Git diffs (local repo) or manual snippets.
- Feedback capture: comments, tasks, suggested changes.

### 2.7 Code Architecture Organization
- Map or outline view for modules, responsibilities, and dependencies.
- Ability to pin architectural notes, decisions, and rationale for AI and human collaborators.
- Planning templates (e.g., feature request, refactor) that link context blocks and prompts.

### 2.8 Multi-Agent Orchestration ⭐ NEW
- **Agent Session Manager**
  - Dashboard showing all active agent sessions across projects
  - Real-time status: idle, working, waiting for review, error state
  - Context window tracking per agent (what they know, what's in their history)
  - Pause/resume/terminate controls for each agent
  
- **Model Hierarchy & Team Setup**
  - Define "agent roles" (e.g., Architect, Implementer, Tester, Reviewer)
  - Assign specific models/APIs to each role (Claude for Arch, GPT-3.5 for Impl)
  - Role-based permission sets (who can edit what, who needs approval)
  
- **Work Delegation Pipeline**
  - Architecture → Specification → Interface Definition → Implementation workflow
  - Automatic task breakdown: AI generates subtasks, you approve, they get assigned
  - Dependency tracking between delegated tasks
  - Handoff protocols: structured context passing between agents
  
- **Agent-to-Agent Review**
  - Assign code review between agents (Agent A implements, Agent B reviews)
  - Review feedback threads with severity levels
  - Dispute escalation: when agents disagree, flag for human decision
  - Review quality scoring: track which agent reviewers catch real issues
  
- **Results Aggregation**
  - Collect outputs from multiple cheap agents on same task
  - Consensus building: show where agents agree/disagree
  - Confidence scoring based on model agreement
  - Merge tools for combining partial implementations

### 2.9 Cost Optimization ⭐ NEW
- **Cost-Aware Routing Engine**
  - Cost estimation before sending any prompt
  - Automatic model selection based on task complexity vs. budget
  - Smart routing rules (e.g., "use cheapest model under 90% accuracy threshold")
  - Override controls: force specific model for critical tasks
  
- **Budget Management**
  - Per-project budget caps with alerts at 50%, 80%, 95%
  - Per-task budgets with automatic escalation approval
  - Daily/weekly/monthly spending limits across all agents
  - Cost forecasting based on planned work
  
- **Cost Analytics**
  - Real-time spend dashboard
  - Cost per feature/completed task
  - Model comparison reports (cost vs. quality for different task types)
  - Export for expense tracking

### 2.10 AI Debate & Expert Panels ⭐ NEW
- **Poll the Experts Mode**
  - Submit a question to a panel of 3-7 models simultaneously
  - Side-by-side response comparison with highlighting
  - Cost breakdown per model (useful for comparing cheap vs. expensive)
  - Response time tracking
  
- **Model Variance Analysis**
  - Automatic detection of agreement/disagreement points
  - Semantic similarity scoring between responses
  - Controversy heatmap showing where models diverge most
  - Confidence indicators based on consensus level
  
- **Structured Debate Mode**
  - Round-robin critique: each model responds to others' answers
  - Synthesis round: models revise their positions based on critiques
  - Final vote/position on the question
  - Debate transcript with threaded discussion view
  
- **Consensus Aggregation**
  - Merge multiple responses into single synthesized answer
  - Attribute specific claims to originating models
  - Highlight areas of strong consensus vs. uncertainty
  - Export panel results as annotated document
  
- **Expert Panel Templates**
  - Pre-defined panels: "Code Review Panel", "Architecture Review", "Research Panel"
  - Custom panel creation with model selection and roles
  - Save favorite panel configurations
  - Weighted consensus (trust some models more than others)

### 2.11 AI Control Center (CLI Replacement) ⭐ NEW
- **Conversation Management**
  - Persistent chat sessions with full history
  - Searchable conversation archive across all projects
  - Export conversations (Markdown, JSON, HTML)
  - Chat templates for common conversation starters
  
- **Conversation Forking & Time Travel**
  - Fork conversation at any message (creates new branch)
  - Navigate full conversation tree
  - Compare branches side-by-side
  - "Rewind" to any point and retry with different approach
  - Visual timeline showing all branches
  
- **Complete Tool Use**
  - Shell command execution (with approval gates)
  - File operations (read, write, list, search)
  - Code viewing and editing
  - Git integration (status, diff, commit, branch)
  - Web search and fetch
  - Build/test command execution
  
- **CLI Parity Features**
  - All capabilities of Claude Code, Aider, Cody, etc.
  - Command palette (Cmd+K) for quick actions
  - Keyboard shortcuts for power users
  - Terminal integration (embedded or external)

### 2.12 Real-Time Collaboration UI ⭐ NEW
- **AI Todo Visibility**
  - Persistent sidebar showing AI's current task
  - Real-time progress updates (streaming)
  - Subtask breakdown with completion checkboxes
  - Time estimates vs. actual tracking
  - Blocked task indicators with hover explanations
  - "What am I working on?" quick summary
  
- **Shared Editor Presence**
  - AI opens files → automatic tab creation
  - AI navigates to line → scroll and highlight
  - Ghost cursor showing AI's current focus
  - Real-time edit preview (before application)
  - Follow mode (your view syncs to AI)
  - Change approval/reject per modification
  - Conflict resolution when both editing same file
  
- **Activity Feed**
  - Real-time stream of AI actions
  - File touched, command executed, tool called
  - Cost accumulation ticker
  - Filter by agent, task, or action type
  - Collapsible/detachable panel

### 2.13 Scheduling System ⭐ NEW
- **Scheduled Prompts**
  - Cron-like syntax support
  - One-time scheduled prompts
  - Recurring: daily, weekly, custom intervals
  - Prompt templates with time/date variables
  - Visual calendar view of scheduled items
  - Notification on execution (optional)
  - Execution history and logs
  
- **AI Self-Scheduling**
  - AI can queue future prompts for itself
  - Natural language scheduling: "remind me tomorrow"
  - Conditional scheduling based on task outcomes
  - Dependency-based scheduling ("after X, do Y")
  - User approval gates for AI-created schedules
  - Scheduled task dashboard (review/cancel)

### 2.14 Heartbeat System ⭐ NEW ⭐ EXPERIMENTAL
- **Core Heartbeat**
  - Configurable pulse interval (10s to 1hr)
  - Background context gathering (files, git, time, errors)
  - Context-aware prompt generation
  - Smart triggering (idle detection, no interruption during typing)
  - Pause/resume controls (toolbar button, keyboard shortcut)
  
- **Proactive Suggestions**
  - Code pattern suggestions based on recent work
  - "You might want to..." contextual hints
  - Reminder system ("You said you'd review this")
  - Related file recommendations
  - Documentation gap detection
  - Test coverage suggestions
  - Refactoring opportunities
  
- **Ambient Features**
  - Music integration (Spotify, Apple Music)
    - Focus mode playlists
    - Task-appropriate music suggestions
    - Auto-pause when AI needs attention
  - Pomodoro timer integration
  - Gentle break reminders (posture, stretch)
  - Daily standup summary generation
  - End-of-day commit suggestions
  - Morning briefing (what's on your plate)
  
- **Personality & Safety**
  - Configurable personality: professional, casual, playful, terse
  - Notification sounds (optional, customizable)
  - Greeting on startup with context
  - Do Not Disturb mode
  - Cost tracking (heartbeats are not free!)
  - Decision transparency (log every suggestion reason)
  - User feedback on suggestions (thumbs up/down for learning)

### 2.15 Tool System ⭐ NEW
- **Tool Visibility & Control**
  - Real-time tool output streaming
  - Tool execution status (pending, running, completed, error)
  - Exit codes and diagnostics display
  - Tool performance metrics (duration, cost)
  - Tool execution history and replay
  
- **Plugin Architecture** (Simple)
  - WASM-based plugins (sandboxed, fast)
  - Python script plugins (embedded runtime)
  - Native Rust plugins (dynamic loading)
  - Plugin manifest (name, version, permissions, schema)
  - Hot-reload for development
  
- **Built-in Tools**
  - Shell execution (bash, zsh, PowerShell)
  - File operations (read, write, list, search)
  - Git tools (status, diff, log, commit, branch)
  - Web tools (fetch, search)
  - Code tools (grep, find references, lint)
  - Build tools (compile, test, package managers)
  
- **Token-Optimized Output** ⭐ KEY
  - Smart truncation with "... (truncated, full output in panel)"
  - Error extraction from stack traces
  - Build output summarization
  - Directory tree vs. flat list selection
  - Line number references without full content
  - Semantic compression (keep meaning, reduce tokens)

### 2.16 Python Embedded ⭐ NEW
- **Python Runtime**
  - Embedded Python interpreter (PyO3 or rust-cpython)
  - AI can execute arbitrary Python code
  - Access to Python ML ecosystem (numpy, pandas, etc.)
  - Jupyter-like cell execution
  
- **Data Exchange**
  - Serialize Rust structs to Python dicts/objects
  - Return Python results to Rust
  - Shared memory for large data (zero-copy)
  - JSON/Arrow as interchange format
  
- **Package Management**
  - Automatic virtualenv per project
  - requirements.txt / pyproject.toml support
  - Automatic dependency installation
  - Isolated environments
  
- **Security**
  - Sandboxed execution (resource limits)
  - No network access by default
  - File system access controls
  - Timeout enforcement

### 2.17 Multi-Project Workspace ⭐ NEW
- **Project Management**
  - Multiple open projects simultaneously
  - Project tabs or sidebar switcher
  - Fast context switching (< 100ms)
  - Per-project settings and configuration
  
- **Split Views**
  - Side-by-side project comparison
  - Drag files between projects
  - Shared clipboard across projects
  - Independent window option (pop out project)
  
- **Context Preservation**
  - Per-project open files/tabs
  - Per-project recent files
  - Per-project AI todo lists
  - Per-project conversation history
  - Project-specific agent assignments
  
- **Global Resources**
  - Shared prompt library across projects
  - Global agents (available everywhere)
  - Cross-project search
  - Shared cost budget tracking

### 2.18 Bug Tracking ⭐ NEW
- **Lightweight Issues**
  - Minimal fields: title, description, status, priority
  - No bloated workflow (open → in progress → closed)
  - Quick capture from anywhere (keybinding)
  
- **AI Integration**
  - Auto-capture errors from tool output
  - AI suggests fixes based on error context
  - Link to conversation where bug was found
  - Link to code location
  - Link to commit that introduced/fixed
  
- **Triage**
  - AI auto-prioritization based on severity
  - Duplicate detection
  - Related issue suggestions
  - "Similar to issue #42"
  
- **Minimal Overhead**
  - Create issue in < 5 seconds
  - No mandatory fields
  - No meetings, no process
  - Just track and fix

### 2.19 Model Provider Support ⭐ NEW
- **Supported Providers** (Initial)
  - OpenAI (GPT-4, GPT-4o, GPT-4o-mini, etc.)
  - Anthropic (Claude 3 Opus, Sonnet, Haiku)
  - Google (Gemini Pro, Flash)
  - Moonshot AI (Kimi)
  - Local models (Ollama, LM Studio)
  - xAI (Grok) - optional
  - Mistral AI - optional
  - Cohere - optional
  
- **Authentication Methods**
  - API keys (stored in system keychain)
  - OAuth flows where supported
  - Environment variable fallback
  - Multiple accounts per provider
  
- **Provider Features**
  - Model discovery (list available models)
  - Token counting per provider
  - Rate limit tracking
  - Cost per model
  - Streaming support
  - Vision/multimodal where available

### 2.20 Token Optimization ⭐ NEW
*Comprehensive token efficiency throughout the system.*

- **Tool Output**
  - Smart truncation with context preservation
  - Error-focused extraction
  - Build result summarization
  - Directory listing optimization
  
- **Code Context**
  - Symbol-level references vs. full files
  - Diff-based updates (send changes, not whole file)
  - Lazy loading of file contents
  - Import/dependency tracking
  
- **Conversation**
  - Rolling summarization for long contexts
  - Message pruning (keep important, drop old)
  - Context compression techniques
  
- **System-Wide**
  - Token budget enforcement
  - Cost estimation before sending
  - "This will use ~2K tokens" warnings
  - Optimization suggestions

### 2.21 AI Interaction Modes
- **Harness Mode**: Built-in collaborative AI can edit prompts, generate todos, and modify shared docs within the UI, using tooling like guided suggestions and contextual helpers.
- **MCP/Tool Server Mode**: Expose the app as a callable MCP server or Python tool interface so external AIs can query documents, log todos, or request architecture guidance with authentication and audit trails.

### 2.12 Commercial Product Readiness
- Onboarding flow, usage analytics, and feature differentiation for paying customers.
- Security controls around local storage, exports, and shared MCP endpoints.
- Shared libraries (UI/data modules) to avoid reinventing components and support future surfaces.

### 2.12 Sandboxed Content Access
- Allowing the app to register folder/take snapshots of user-specified directories across the machine.
- Provide tools/APIs for prompts and AI runners to query registered content safely, with permissions controls and read-only options.
- Support remote/local server connections so the sandboxed content can live across devices (e.g., remote file share or another workstation).
- Authentication adapters for Codex, Grok, Claude, and Gemini so each layer can run through the provider's native auth.

## 3. Code Implementation Planning

### 3.1 Architecture Overview
- **Data Model**
  - Documents: markdown files with metadata (tags, source, used_by).
  - Prompts: templated text with placeholders for context (context_refs list).
  - Tools: registered MCP endpoints with capabilities list.
  - Reviews: user notes, AI feedback, status, linked diffs.
  - **NEW - Agents**: agent sessions with model, role, status, context window, cost spent.
  - **NEW - Tasks**: delegated tasks with owner, reviewer, dependencies, cost budget.
  - **NEW - CostLog**: per-call cost tracking with model, tokens, timestamp.
  - **NEW - ExpertPanels**: panel configurations, debate sessions, model responses.
  - **NEW - PanelResponses**: individual model responses with metadata.
  
- **UI Layers**
  - Side rail for navigation (contexts, tools, prompts, reviews).
  - Main canvas for workspace (editing, planning, deployment instructions).
  - Modal/system overlay for adding metadata or connecting to local AI.
  - **NEW - Agent Orchestration Panel**: dashboard for managing multiple agents.
  - **NEW - Cost Monitor**: real-time budget display and spend alerts.
  - **NEW - Expert Panel Interface**: side-by-side response comparison, debate view.

### 3.2 Implementation Phases

#### Phase 1: Foundation
1. **Storage & Document Model**
   - Define markdown schema (front matter with tags/status).
   - Build document manager supporting create, edit, preview, history.

2. **Navigation & Layout**
   - Tree view of contexts, prompt library, and tools.
   - Persistent workspace layout configuration (panels, sections).

#### Phase 2: AI Integration
3. **MCP Tool Connectors**
   - Abstraction for registering tools and exposing context via local APIs.
   - UI for linking documents/blocks to tools (drag/drop).

4. **Prompt Library & Templates**
   - Prompt editor with merge fields referencing context (JSON path, doc URL).
   - Template execution preview feeding context data.

#### Phase 3: Multi-Agent Core ⭐ NEW PRIORITY
5. **Agent Session Management**
   - Agent registry with model/API configuration
   - Session lifecycle management (spawn, monitor, terminate)
   - Context window tracking and management
   - Basic agent status dashboard

6. **Cost Tracking Infrastructure**
   - Per-call cost calculation and logging
   - Budget checking before API calls
   - Basic spend dashboard

#### Phase 4: Advanced Orchestration
7. **Delegation Workflows**
   - Task creation and breakdown UI
   - Approval gates between workflow stages
   - Automated handoff protocols
   - Dependency tracking

8. **Agent-to-Agent Review**
   - Review assignment system
   - Feedback collection and threading
   - Dispute escalation to human
   - Review quality metrics

9. **Expert Panel System** ⭐ NEW
   - Panel configuration (select models, set roles)
   - Parallel query execution to multiple providers
   - Side-by-side response comparison UI
   - Variance analysis and consensus scoring

10. **Review & Planning Workspace**
    - Diff capture (integration or manual).
    - Threaded notes/AI feedback view with status toggles.
    - Task generation from review findings.

#### Phase 5: AI Interaction Layers
11. **Harness Mode Implementation**
    - Built-in AI collaboration UI
    - Action logging and permission controls
    - Guided helpers for common tasks

12. **MCP/Tool Server Mode**
    - Expose Python/REST interface for external AIs
    - Telemetry and security hooks
    - Authentication adapters for all supported providers

#### Phase 6: Advanced Features
12. **Cost Optimization Engine**
    - Automatic model selection based on cost/quality
    - Smart routing rules engine
    - Cost forecasting and optimization suggestions

13. **Results Aggregation**
    - Multi-agent consensus building
    - Output merging and conflict resolution
    - Confidence scoring

14. **Structured Debate Mode** ⭐ NEW
    - Round-robin critique workflow
    - Response revision tracking
    - Debate transcript generation
    - Final consensus position

15. **Sandboxed Content Access**
    - Directory registration and snapshots
    - Safe query APIs for prompts and AI runners
    - Remote server connections

## 4. Exhaustive TODOs

### Planning Required
- [ ] Align on primary personas (AI collaborator, developer, reviewer).
- [ ] Define success metrics (reduced prep time, review quality).
- [ ] Gather example prompts and context flows from current workflow.
- [ ] Decide on storage backend (local files, db, both).
- [ ] Outline MCP/local tool API contract (REST, socket, etc.).
- [ ] Clarify commercialization approach: pricing tiers, onboarding, analytics surface.
- [ ] Determine authentication/credential requirements for each supported model/provider (Codex, Grok, Claude, Gemini).
- [ ] Define sandbox permission model for accessing arbitrary directories and remote/local content sources.
- **NEW:**
- [ ] Define agent role taxonomy (Architect, Implementer, Tester, Reviewer, etc.)
- [ ] Establish cost/quality benchmarks for model routing decisions
- [ ] Design approval workflow UX (when to interrupt vs. auto-approve)
- [ ] Plan agent communication protocol (how agents share context)
- [ ] Define review quality metrics and feedback loops
- [ ] Design expert panel configurations (which models, what roles)
- [ ] Define consensus scoring algorithms for multi-model responses
- [ ] Plan debate moderation rules and round structures

### Documentation & Visuals
- [ ] Create UX sketches for navigation/workspace layout.
- [ ] Document data schema for context, prompts, tools.
- [ ] Draft onboarding copy for new users (how to structure prompts).
- **NEW:**
- [ ] Design agent orchestration dashboard mockups
- [ ] Create cost monitoring UI wireframes
- [ ] Document delegation workflow diagrams
- [ ] Design agent-to-agent review interface
- [ ] Design expert panel comparison UI (side-by-side view)
- [ ] Create debate mode interaction wireframes
- [ ] Design consensus visualization components

### Implementation Prep
- [ ] Inventory existing markdown/context that users want preserved.
- [ ] Research local MCP frameworks/tooling for integration.
- [ ] Investigate Ollama API for startup/shutdown control plus sandboxed local model hosting.
- [ ] Catalog diffs or repos targeted for AI code review flows.
- [ ] Plan prompt template DSL or placeholders (e.g., {{context:project}}).
- **NEW:**
- [ ] Research cost APIs for each provider (token pricing, rate limits)
- [ ] Evaluate task queue systems for multi-agent coordination
- [ ] Research consensus algorithms for aggregating agent outputs
- [ ] Plan agent context serialization format
- [ ] Research semantic similarity algorithms for response comparison
- [ ] Investigate parallel query execution patterns
- [ ] Evaluate text diff algorithms for finding model disagreements

### Process & Collaboration
- [ ] Set review cadence (e.g., weekly planning updates).
- [ ] Identify checkpoints for testing each workspace component.
- [ ] Determine required logs/metrics for prompt/library usage.
- **NEW:**
- [ ] Establish agent performance review process
- [ ] Define cost budget approval workflows
- [ ] Plan agent-to-agent review quality standards

### Future Considerations
- [ ] Localization strategy for prompt text and interface.
- [ ] Sync options for multi-machine or team usage.
- [ ] Extensions for web-based AI services or remote agents.
- **NEW:**
- [ ] Team collaboration features (shared agent pools)
- [ ] Marketplace for agent roles and workflows
- [ ] Advanced routing with learned preferences
