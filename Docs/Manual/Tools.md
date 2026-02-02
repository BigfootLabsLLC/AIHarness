# AIHarness Available Tools

This document lists the tools available via the AIHarness HTTP API and MCP interface.

## Core File System Tools

### `read_file`
Reads the contents of a file.
- **Arguments:**
  - `path` (string): Absolute path to the file.

### `write_file`
Writes content to a file.
- **Arguments:**
  - `path` (string): Absolute path to the file.
  - `content` (string): Content to write.

### `list_directory`
Lists contents of a directory.
- **Arguments:**
  - `path` (string): Absolute path to the directory.

### `search_files`
Search for text in files.
- **Arguments:**
  - `path` (string): Directory to search in.
  - `query` (string): Text or regex to search for.

## Todo Management Tools
These tools manage the project-specific todo list.

### `todo_add`
Add a new todo item.
- **Arguments:**
  - `title` (string): Task title.
  - `description` (string, optional): Task details.
  - `position` (integer, optional): Order position.
  - `project_id` (string): Project ID (default: "default").

### `todo_list`
List all todo items.
- **Arguments:**
  - `project_id` (string): Project ID.

### `todo_check`
Mark a todo as completed or active.
- **Arguments:**
  - `id` (string): Todo ID.
  - `completed` (boolean): True for completed, false for active.

### `todo_remove`
Remove a todo item.
- **Arguments:**
  - `id` (string): Todo ID.

### `todo_get_next`
Get the next incomplete todo item.
- **Arguments:**
  - `project_id` (string): Project ID.

## Build System Tools
Manage and run build commands for the project.

### `build_add_command`
Add a new build command.
- **Arguments:**
  - `name` (string): Human-readable name (e.g., "Build Release").
  - `command` (string): Shell command to execute.
  - `working_dir` (string, optional): Directory to run in.

### `build_list_commands`
List available build commands.
- **Arguments:** None

### `build_run_command`
Execute a specific build command.
- **Arguments:**
  - `id` (string): Command ID.

### `build_set_default`
Set a command as the default build action.
- **Arguments:**
  - `id` (string): Command ID.

### `build_get_default`
Get the current default build command.
- **Arguments:** None

## Session Management

### `next_session_read`
Read the briefing notes for the next session.
- **Arguments:** None

### `next_session_write`
Update the briefing notes for the next session.
- **Arguments:**
  - `content` (string): The briefing content.
