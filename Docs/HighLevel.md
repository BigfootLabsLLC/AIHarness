# Overview
The goal of this app is to provide a convienent way to organize prompts and context when working with AI collaborators.

# Core Philosophy
This tool enables a **multi-agent orchestration paradigm** where:
- **Premium models** (Claude, GPT-4, etc.) handle architecture, planning, and guidance
- **Cheaper models** handle implementation, testing, and specific coding tasks
- **Agents review each other's work** for quality assurance
- **You maintain oversight** with clear visibility into what each agent is doing

# Features

## Context & Prompt Management
* Organize all the context you might want to share with the AI. Make it easy to edit markdown files, copy over blocks of text, etc.
* Provide a local MCP/tool interface for local AIs to interact with to pull data from so you can tell the AI to pull its own prompt.
* Maintain a prompt library
* AI Code reviews -- provide feedback on code changes being made or requested. 
* Code architecture organization
* Work and plan with the AI together.

## Multi-Agent Orchestration
* **Model multitasking** - Manage multiple agent sessions within a project from the same AI.
* **Model hierarchy** - Set up a coding team with different quality models and APIs doing different parts of the work.
* **Agent session tracking** - See what each agent is working on, their progress, and their current context.
* **Work delegation workflows** - Automatically break architecture into specs, interfaces, and implementation tasks.
* **Agent-to-agent review** - Enable agents to review each other's code and flag issues for your approval.
* **Results aggregation** - Combine outputs from multiple cheap agents and build consensus.

## Cost Optimization
* **Cost-aware routing** - Automatically route tasks to the cheapest capable model.
* **Budget tracking** - Set per-project or per-task budgets with real-time spend monitoring.
* **Cost comparison** - Compare model performance vs. cost for similar tasks.
* **Smart fallbacks** - Retry with cheaper models first, escalate only when needed.

## AI Debate & Expert Panels
* **"Poll the Experts"** - Send the same question to multiple models and compare their responses side-by-side.
* **Model variance detection** - See where models agree and disagree to understand uncertainty in AI responses.
* **Structured debates** - Have models critique each other's answers for deeper analysis.
* **Consensus building** - Aggregate multiple expert opinions into a synthesized recommendation.
* **Cost-effective wisdom** - Include cheaper models in the panel to see if they match expensive ones.

## Workflow Integration
* **Architecture → Spec → Implementation pipeline** - Structured handoffs between planning and coding.
* **Approval gates** - Review and approve work at each stage before delegation continues.
* **Context preservation** - Maintain continuity as work passes between agents.
* **Quality gates** - Automated checks before accepting agent outputs.

## AI Control Center ⭐ NEW — CLI Replacement
* **Complete CLI parity** - Everything AI CLI tools can do (Claude Code, Aider, etc.) embedded in the app.
* **Conversation forking** - Branch conversations at any point, explore different paths.
* **Time travel** - Navigate full conversation history, rewind to any message.
* **Full tool use** - Shell commands, file operations, code editing, Git integration, web search.
* **Persistent chat logs** - Searchable, exportable conversation history across all sessions.
* **Chat templates** - Save and reuse conversation starters.

## Real-Time Collaboration ⭐ NEW
* **AI todo visibility** - See what the AI is working on in real-time, with progress bars and status.
* **Shared editor** - AI opens files as tabs, navigates to locations, ghost cursor shows where AI is looking.
* **Real-time edits** - AI changes appear as you work (like Google Docs), with accept/reject per change.
* **Activity feed** - Stream of AI actions: files touched, commands run, costs accumulating.

## Scheduling System ⭐ NEW
* **Scheduled prompts** - Cron-like scheduling for recurring or one-time prompts.
* **AI self-scheduling** - AI can schedule future check-ins: "Review this tomorrow", "Retry in 10 minutes if failed".
* **Conditional scheduling** - "If tests fail, schedule retry", "After task X, schedule review".
* **Calendar view** - Visual schedule management with execution history.

## Heartbeat System ⭐ NEW ⭐ EXPERIMENTAL
* **Living AI companion** - Background thread that pulses on a tunable interval.
* **Proactive suggestions** - Context-aware ideas based on what you're doing, time of day, recent activity.
* **Ambient features** - Music control, Pomodoro reminders, stretch breaks, daily summaries.
* **Non-intrusive** - Sidebar notifications, never modal interruptions.
* **Smart triggering** - Detects idle vs. deep work, only speaks when relevant.
* **Personality** - Configurable voice: professional, casual, playful, terse.

## Tool System ⭐ NEW
* **Tool visibility** - See output, diagnostics, and status of every tool the AI uses.
* **Plugin architecture** - Simple system for adding new tools (WASM, Python, or native).
* **Built-in tools** - Shell, file system, Git, web search, code search, build tools.
* **Tool output optimization** - Smart truncation, error extraction, low-token summaries.

## Python Embedded ⭐ NEW
* **Embedded Python runtime** - Access Python ecosystem without leaving the app.
* **AI can run Python** - Execute Python scripts, use ML libraries, data analysis.
* **Seamless data exchange** - Pass data between Rust backend and Python runtime.
* **Package management** - Handle Python dependencies automatically.

## Multi-Project Workspace ⭐ NEW
* **Multiple projects** - Work on 2-3 projects simultaneously.
* **Project switching** - Fast context switching without losing state.
* **Split views** - Side-by-side project views.
* **Shared resources** - Global prompt library, shared agents across projects.
* **Mental context preservation** - Project-specific todo lists, recent files, open tabs.

## Bug Tracking ⭐ NEW
* **Lightweight bug tracker** - AI-friendly issue tracking.
* **Auto-capture** - Errors, crashes, AI-identified issues.
* **Context linking** - Link bugs to code, conversations, commits.
* **AI triage** - Auto-prioritize, suggest fixes, find related issues.
* **Minimal overhead** - No bloated workflow, just track and fix.

## Token Optimization ⭐ NEW
* **Smart output truncation** - Get the gist without overfilling context.
* **Error extraction** - Pull just the relevant error from stack traces.
* **Build output summarization** - "Build failed: 3 errors in src/parser.rs" vs full log.
* **Directory listing optimization** - Tree view vs flat list based on context.
* **Lazy loading** - Load file contents only when needed.
* **Diff optimization** - Show only changed sections, not entire files.

## Client/Server Mode ⭐ NEW
* **Server mode** - Run AIHarness on powerful desktop as headless server.
* **Thin client** - Laptop connects to server, UI streams locally.
* **Local network** - Everything stays on your LAN, no cloud needed.
* **Compute separation** - Desktop handles AI calls, agents, heavy lifting.
* **Session persistence** - Disconnect laptop, server keeps working.
* **Multi-client** - Connect from multiple devices simultaneously.
* **Seamless sync** - Switch from local mode to server mode transparently.
