# AIHarness Version Roadmap

> **Development Philosophy:** Working software over comprehensive specs. Each version should be *usable*, not just a checklist of features.

---

## Core Principles

1. **Version N must be runnable and useful** — not just scaffolding
2. **Dogfood early** — we use it to build the next version
3. **One integration at a time** — get Claude working *well* before adding Gemini
4. **CLI parity first, bells later** — if Claude Code can do it, we should too
5. **Vertical slices over horizontal** — full feature (UI → Backend → Integration) over partial features

---

## Version 0.1 — "Hello AI"

**Goal:** Basic scaffolding + one working AI integration. Prove the stack works.

**Definition of Done:**
- [ ] Tauri app launches without errors
- [ ] React UI renders (basic layout: sidebar, chat panel)
- [ ] Rust backend initializes (SQLite connects, no migrations yet)
- [ ] **One AI provider integrated** (Claude via Anthropic)
  - [ ] Can send a message
  - [ ] Can receive a response
  - [ ] Basic streaming (optional for V0.1, can be polling)
- [ ] **Basic tool use** (read_file, list_directory)
  - [ ] AI can request tool use
  - [ ] Tool output shown in UI
- [ ] Conversation persists to SQLite
- [ ] Can start a new conversation

**Explicitly NOT in V0.1:**
- Multiple providers
- Cost tracking
- Agent system
- Workflows
- Expert panels
- Heartbeat
- Client/server mode
- Python embedding

**V0.1 Success Criteria:**
> "I can open the app, start a conversation with Claude, ask it to read a file, and see the response."

---

## Version 0.2 — "Dogfood"

**Goal:** Use V0.1 to build V0.2. Fix pain points. Add essentials.

**Definition of Done:**
- [ ] Used V0.1 for 1 week, documented friction
- [ ] Monaco editor integrated (file viewing)
- [ ] All basic file tools work (read, write, list, search)
- [ ] Shell tool (with approval)
- [ ] Git tools (status, diff)
- [ ] Conversation branching/forking
- [ ] Cost tracking (per call, basic display)
- [ ] Basic agent (single agent, simple task execution)

**Success Criteria:**
> "I can use this instead of Claude Code for basic coding tasks."

---

## Version 0.3 — "Workflows"

**Goal:** Delegation works. Multi-step tasks.

**Definition of Done:**
- [ ] Task system (create, assign, track)
- [ ] Agent can break work into subtasks
- [ ] Human approval gates
- [ ] File watching (AI notices changes)
- [ ] Second provider (OpenAI) for comparison
- [ ] Basic cost optimization (choose cheaper model)

**Success Criteria:**
> "I can say 'implement this feature' and the AI breaks it down, asks approval, and does it."

---

## Version 0.4 — "Multi-Agent"

**Goal:** Multiple agents, expert panels, real collaboration.

**Definition of Done:**
- [ ] Multiple concurrent agents
- [ ] Agent-to-agent handoff
- [ ] Expert panel system (poll multiple models)
- [ ] Agent todo visibility in UI
- [ ] Real-time activity feed

**Success Criteria:**
> "I have an Architect agent planning and an Implementer agent coding, and I can see both working."

---

## Version 0.5 — "Control Center"

**Goal:** Complete CLI replacement + unique features.

**Definition of Done:**
- [ ] All tools from Claude Code work
- [ ] Scheduling (basic)
- [ ] Heartbeat (experimental, off by default)
- [ ] Multi-project workspace
- [ ] Bug tracking
- [ ] Python embedded

**Success Criteria:**
> "I live in this app. I don't open Claude Code anymore."

---

## Version 0.6 — "Scale"

**Goal:** Client/server mode, polish, performance.

**Definition of Done:**
- [ ] Client/server mode (LAN)
- [ ] Token optimization everywhere
- [ ] Tool plugin system
- [ ] Performance: handles 100K line files
- [ ] Packaging for all platforms

**Success Criteria:**
> "I run the server on my desktop, work from my laptop, and it's faster than local Claude."

---

## Version 1.0 — "Production"

**Goal:** Others can use it. Stable, documented, installable.

**Definition of Done:**
- [ ] Stable release
- [ ] Documentation complete
- [ ] Auto-updater
- [ ] Error reporting
- [ ] Commercial features (if any)

---

## Development Approach

### Vertical Slices

**Bad (horizontal):**
- Week 1: All UI components
- Week 2: All Rust structs
- Week 3: All integrations

**Good (vertical):**
- Week 1: Chat UI → Message types → Claude integration → SQLite persistence
- Week 2: File tool UI → File operations → Tool system → Display results

### Decision Framework

When prioritizing, ask:

1. **Does this unblock dogfooding?** → Do it
2. **Is this required for V0.N definition?** → Do it
3. **Does this save tokens/cost?** → Maybe later
4. **Is this a "nice to have"?** → V0.5+
5. **Is this experimental?** → Feature flag, default off

### Tech Debt Rules

- V0.1-V0.3: Tech debt OK if it ships
- V0.4: Pay down debt, refactor
- V0.5+: Clean architecture only

---

## Current Focus

**We are here:** Planning V0.1

**Next immediate task:** Scaffold the Tauri project

**Success metric:** `cargo tauri dev` launches, React renders "Hello AIHarness"
