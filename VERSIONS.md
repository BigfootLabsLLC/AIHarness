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

## Version 0.1 — "The Bridge" (MCP Server)

**Goal:** Local MCP server that external AI tools can connect to. AIHarness is the "tool provider" for your existing AI assistants.

**Why this first:** Claude and others have limited direct tool access. Instead, we build a local MCP server that **you** control, and any AI can call it.

**Definition of Done:**
- [ ] Tauri app launches (system tray or window)
- [ ] **Local MCP Server** running on localhost
  - [ ] MCP protocol over stdio or HTTP (localhost only)
  - [ ] Exposes tools: `read_file`, `write_file`, `list_directory`, `search_files`
  - [ ] Exposes resources: project context, todo list, file contents
  - [ ] Simple token auth
- [ ] **React UI for monitoring**
  - [ ] Server status (running/stopped)
  - [ ] Tool call log (what was called, when, result)
  - [ ] Connected clients list
  - [ ] Basic context management (add/remove files from context)
- [ ] **Persistence**
  - [ ] SQLite for tool call history
  - [ ] Context configuration persists

**Explicitly NOT in V0.1:**
- Built-in AI/chat (external AIs provide this)
- Direct API integration
- Cost tracking
- Agents, workflows, panels, etc.

**V0.1 Success Criteria:**
> "I can configure Claude Desktop (or Cursor, etc.) to use AIHarness's MCP server, and it can read my project files and todo list through the bridge."

**Usage Example:**
```bash
# Claude Desktop config
{
  "mcpServers": {
    "aiharness": {
      "command": "aiharness-mcp-server",
      "env": { "AIH_TOKEN": "..." }
    }
  }
}

# In Claude chat
User: "What's on my todo list?"
Claude: [calls aiharness/get_todos]
Claude: "You have 3 items: ..."
```

---

## Version 0.2 — "Direct Connect" (Built-in AI)

**Goal:** AIHarness gets its own AI integration. No external MCP client needed — chat directly in the app.

**Definition of Done:**
- [ ] **Direct AI API integration**
  - [ ] Connect to one provider (OpenAI, Anthropic, or local Ollama)
  - [ ] Built-in chat UI (message thread, input box)
  - [ ] Streaming responses
  - [ ] Tool use (same tools from V0.1, now called by built-in AI)
- [ ] **Enhanced UI**
  - [ ] Monaco editor for file viewing
  - [ ] File tree sidebar
  - [ ] Chat panel (persistent, not external)
- [ ] **Tool expansion**
  - [ ] Shell execution (with approval UI)
  - [ ] Git tools (status, diff, log)
- [ ] **Basic cost tracking**
  - [ ] Per-call cost logging
  - [ ] Simple cost display in UI

**Success Criteria:**
> "I can open AIHarness, chat with AI directly in the app, and it can use tools to help me code."

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
