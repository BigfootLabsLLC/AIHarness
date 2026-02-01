# AIHarness Work TODO

## In Progress
- [ ] Fix panel overflow: wrap long text and add scrollbars in side panels
- [ ] Todo list auto-refresh on tool calls
- [ ] Context settings: choose which files are in the context blob vs on-demand
- [ ] Context blob tool: returns manual text + full-file inclusions
- [ ] Project-aware context tools + file-on-demand tools
- [ ] Shared library DB + move/copy flows
- [ ] Project templates + import/export
- [ ] Project selector should scope tool history/todos/context to the active project

## Done
- [x] Build commands: store per project + MCP tools + UI buttons
- [x] File System panel: project tree + filesystem browser tabs
- [x] Context editor: manual text lines stored in project DB
- [x] Project add UI (name + folder) wired to registry
- [x] Reduce padding/rounding for a tighter, utilitarian UI
- [x] Project system: global registry DB + per-project DB in `.aiharness/project.db`
- [x] Project-aware tool calls + context storage + todo storage
- [x] Ordered todo list UI wired to storage
- [x] Todo tools: add, remove, check, read, get_next, insert, move
- [x] Project selector UI wired to registry
- [x] MCP-over-HTTP endpoint
- [x] MCP stdio proxy mode
- [x] Auto-start HTTP server
- [x] Build output in build/ and tests-on-build
