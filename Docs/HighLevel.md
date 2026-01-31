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

## Workflow Integration
* **Architecture → Spec → Implementation pipeline** - Structured handoffs between planning and coding.
* **Approval gates** - Review and approve work at each stage before delegation continues.
* **Context preservation** - Maintain continuity as work passes between agents.
* **Quality gates** - Automated checks before accepting agent outputs.
