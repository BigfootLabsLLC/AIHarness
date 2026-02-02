# Testing Guide

This document describes the testing strategy for AIHarness, with a focus on preventing frontend-backend serialization bugs.

## Test Layers

### 1. Contract Tests (`src-tauri/src/tests/contract_tests.rs`)

**Purpose:** Verify that parameters sent from the frontend are correctly deserialized by the backend.

**Why:** Tauri v2 has subtle serialization issues where `Option<String>` parameters can be lost between frontend and backend. These tests catch that specific bug class.

**Run:**
```bash
cd AIHarness/src-tauri
cargo test contract_tests
```

**When to add:** Any time you add or modify a Tauri command that takes parameters, add a contract test for its argument struct.

**Example:**
```rust
#[test]
fn test_my_command_args_deserialization() {
    let json = json!({
        "args": {
            "project_id": "test-id",
            "other_param": "value"
        }
    });
    
    let args: MyCommandArgs = serde_json::from_value(
        json.get("args").unwrap().clone()
    ).expect("Should deserialize");
    
    assert_eq!(args.project_id, Some("test-id".to_string()));
}
```

### 2. Integration Tests (`src-tauri/src/tests/integration_tests.rs`)

**Purpose:** Test the actual database operations with temporary databases.

**Why:** Ensures project isolation and data integrity work correctly.

**Run:**
```bash
cd AIHarness/src-tauri
cargo test integration_tests
```

**Key tests:**
- `test_todo_project_isolation` - Verifies todos don't leak between projects
- `test_todo_crud_operations` - Verifies create/read/update/delete works
- `test_default_project_fallback` - Verifies default project handling

### 3. Unit Tests (inline in modules)

**Purpose:** Test individual functions and modules in isolation.

**Run:**
```bash
cd AIHarness/src-tauri
cargo test
```

## CI/CD

GitHub Actions runs all tests on every PR and push to main:

1. **Contract Tests** - Fast feedback on serialization (30s)
2. **Rust Tests** - Full test suite including integration tests
3. **Build Test** - Ensures the app compiles on macOS

## Pre-commit Hook

Install the pre-commit hook to catch serialization issues before they reach CI:

```bash
git config core.hooksPath .githooks
```

The hook runs contract tests before each commit.

## Debugging Failed Tests

### Serialization issues

If contract tests fail, check:

1. **Frontend:** Is it sending `{ args: { param: value } }`?
2. **Backend:** Does the struct use `#[serde(default)]` for optional fields?

Example of correct pattern:
```rust
#[derive(Debug, Deserialize)]
struct MyArgs {
    #[serde(default)]  // Required for optional params!
    project_id: Option<String>,
}

#[tauri::command]
async fn my_command(args: MyArgs) { ... }
```

### Database issues

Integration tests use temporary directories that are cleaned up after tests. If tests fail:

1. Check that `TempDir` is properly leaked for the test duration
2. Verify database paths are unique per test

## Adding New Tests

### For new commands

1. Create argument struct with `#[serde(default)]` on optional fields
2. Add contract test in `contract_tests.rs`
3. Add integration test if it touches the database
4. Update frontend to use `{ args: { ... } }` pattern

### Test naming convention

- Contract tests: `test_<command>_args_deserialization`
- Integration tests: `test_<feature>_<scenario>`

## Test Coverage

Current coverage:
- ✅ Serialization contract tests (6 tests)
- ✅ Todo project isolation (3 tests)
- ✅ Database CRUD operations (covered in integration tests)
- ⬜ Frontend unit tests (TODO)
- ⬜ End-to-end tests with WebDriver (TODO)
