# Session Restart Notes (AIHarness)

Date: 2026-02-01

## Context
- App is a Tauri (Rust) + React app. HTTP server auto-starts and exposes MCP at `http://127.0.0.1:8787/mcp`.
- MCP stdio proxy mode: `AIH_PORT=8787 aiharness --mcp-stdio-proxy`.
- Build output collected in `build/AIHarness.app`.
- Project system uses a global registry DB and per-project `.aiharness/project.db`.

## Current Work State
- Left project tabs are thin and flush-left.
- File system panel is now a **tree view** (expand/collapse).
- Context notes panel is wired to project DB (add/edit/remove notes).
- Build commands are project-scoped and stored in DB.
- Top toolbar includes a **Build** button that runs the default build command.
- Project creation uses a folder picker (Tauri dialog) and auto-creates missing directories.

## How to use MCP to set build command (AIHarness example)
1) Add build command:
```
curl -s http://127.0.0.1:8787/mcp \
  -H "content-type: application/json" \
  -d '{
    "jsonrpc":"2.0",
    "id":1,
    "method":"tools/call",
    "params":{
      "name":"build_add_command",
      "arguments":{
        "name":"Build App",
        "command":"npm run build:app",
        "working_dir":"/Users/danbaker/Projects/AIHarness/AIHarness"
      },
      "project_id":"default"
    }
  }'
```

2) Set default:
```
curl -s http://127.0.0.1:8787/mcp \
  -H "content-type: application/json" \
  -d '{
    "jsonrpc":"2.0",
    "id":2,
    "method":"tools/call",
    "params":{
      "name":"build_set_default",
      "arguments":{"id":"<id-from-previous-response>"},
      "project_id":"default"
    }
  }'
```

## Build Command Already Set
- Build command added and set as default via MCP:
  - id: `e513a359-5262-4fa6-8522-b9ea33e93dce`
  - name: `Build App`
  - command: `npm run build:app`
  - working_dir: `/Users/danbaker/Projects/AIHarness/AIHarness`
  - project_id: `default`

## Pending TODOs (from `todo.md`)
- Todo list auto-refresh on tool calls
- Context settings (include-in-blob vs on-demand)
- Context blob tool (notes + full-file inclusions)
- Project-aware context tools + file-on-demand tools
- Shared library DB + move/copy flows
- Project templates + import/export

## Uncommitted Changes (need commit)
- Modified: `Docs/README.md`, `TODO.md`, `package.json`, `package-lock.json`,
  `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/src/lib.rs`,
  `src-tauri/src/http_server.rs`, `src-tauri/src/projects.rs`,
  `src/App.tsx`, `src/index.css`, `src/stores/serverStore.ts`,
  `src/types/index.ts`, `src-tauri/capabilities/default.json`,
  and Tauri schema outputs under `src-tauri/gen/schemas`.
- Deleted: `src/components/ContextPanel.tsx`
- Added: `src-tauri/src/build_commands.rs`

## Notes
- New MCP build tools: `build_add_command`, `build_list_commands`, `build_remove_command`,
  `build_run_command`, `build_set_default`, `build_get_default`.
- Build commands store optional `working_dir` and `is_default`.
- `create_project` now accepts `rootPath` on the frontend; backend creates missing dirs.
- Build succeeds with `npm run build:app`.

## Known Issues / Handoff
- **Tool history and todos aren’t scoped to the active project yet.** Switching projects keeps the previous project’s tool calls, todos, and context notes in the panels. The backend already accepts a `project_id`, but the UI data loaders need to fully reset/refresh when `activeProject` changes. Look at `loadToolHistory`, `loadTodos`, and `loadContext*` calls triggered by the selector watcher.
- **File history panel is empty after switching projects.** No matter which project is active, the History box never refreshes and still reports “No tool calls yet.” Verify that `recentToolCalls` is derived from `toolCalls` filtered by `activeProject` and that the store updates when new tool call events arrive for the currently selected project. The new person should ensure `toolCalls` is reset/loaded per project so the History per-project view works.
- **Status indicator still shows dark dot when running.** The `status` chip is hard-coded but the green "running" state should light up; double-check the CSS class wiring in `src/index.css`.
