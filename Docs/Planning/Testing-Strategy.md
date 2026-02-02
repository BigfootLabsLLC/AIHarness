# AIHarness Testing Strategy

This document outlines the strategy for expanding the test suite of AIHarness, moving beyond unit tests to comprehensive integration and end-to-end verification.

## Goals
1. **Verify Full User Flows**: Ensure features work from the perspective of an external AI or the UI, not just as isolated functions.
2. **Prevent Regression**: Catch bugs in the HTTP API contract and tool logic before they reach users.
3. **Safe Playground**: Provide a way to run tests without affecting the user's real data.

## 1. Existing Test Coverage
Currently, we have Rust unit tests covering:
- **Core Logic**: `TodoStore`, `ProjectRegistry`.
- **Tools**: `ReadFile`, `WriteFile`, `ListDirectory`, `SearchFiles`.

## 2. Integration Testing (New)
We need to test the "glue" – the HTTP server, the JSON serialization, and the state management.

### Python Test Script (`scripts/integration-test.py`)
A Python script acting as an external AI client.

**Workflow:**
1.  **Setup**:
    -   Connect to the running AIHarness server (default port 8787).
    -   Create a temporary test project via the API (requires exposing `create_project` via HTTP or using a pre-setup step). *Note: Currently `create_project` is a Tauri command, not an explicit HTTP tool. We might need to expose it or manually create the DB entry for testing.*
    -   *Alternative*: Use the "default" project but work in a temp subdirectory.
2.  **Execution**:
    -   **File Ops**: Write a file, read it back, list directory, search for content.
    -   **Todos**: Add a todo, list it, check it off, verify it's completed, remove it.
    -   **Build**: Add a dummy build command (e.g., `echo "build"`), run it, verify output.
    -   **Session**: Write a session note, read it back.
3.  **Teardown**:
    -   Clean up created files.
    -   Remove the test project (if possible).

## 3. "Test Mode" / UI Verification
Capabilities within the application to verify its own health.

### Internal "Self-Test" Tool
A Rust function exposed to the UI that runs a series of checks:
-   Check DB connection health.
-   Verify read/write access to the project root.
-   Check HTTP server status.
-   Return a report (Pass/Fail) for each subsystem.

### UI Test Tab (Developer Mode)
A hidden or debug-only tab in the UI:
-   Button to "Run Diagnostics".
-   Displays the log of the self-test.
-   Visualizes the current state of the Redux/Zustand store.

## 4. Implementation Plan

### Phase 1: Python Integration Script
-   Create `scripts/test_client.py`.
-   Implement the "Happy Path" for all tools.
-   Run this script manually or via CI.

### Phase 2: Rust Integration Tests
-   Add `tests/api_tests.rs`.
-   Spin up the `http_server` in a separate thread.
-   Use `reqwest` to hit the endpoints.
-   Use a temp directory for the entire test run.

### Phase 3: Project Creation via API
-   Expose `project_create` as a tool (or a special administrative API endpoint) so the test script can be fully autonomous.

## 5. Specification for `scripts/test_client.py`

**Dependencies**: `requests`, `argparse`.

**Usage**:
```bash
./scripts/run-tests.sh [port]
```

**Test Sequence**:
1.  **Health Check**: GET `/`. Assert 200 OK.
2.  **Tool Discovery**: GET `/tools`. Assert list contains `read_file`, `todo_add`, etc.
3.  **Project Setup**:
    -   (Constraint: If we can't create projects via API yet, use "default").
    -   Define a test working directory.
4.  **File Tests**:
    -   `write_file`("test.txt", "hello").
    -   `read_file`("test.txt") -> assert "hello".
    -   `list_directory`(".") -> assert contains "test.txt".
5.  **Todo Tests**:
    -   `todo_add`("Run test").
    -   `todo_list` -> find the ID.
    -   `todo_check`(id, true).
    -   `todo_list` -> assert completed=true.
    -   `todo_remove`(id).
6.  **Cleanup**:
    -   `run_shell_command`("rm test.txt").

## 6. Future Considerations
-   **Mocking LLMs**: When we add direct AI chat, we'll need to mock the OpenAI/Anthropic API responses to test the chat flow without spending money.
-   **Performance Testing**: Script to spawn 100 concurrent requests to ensure the server doesn't deadlock.
