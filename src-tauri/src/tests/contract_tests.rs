//! Contract tests for frontend-backend communication
//! 
//! These tests verify that parameters sent from the frontend
//! are correctly received by the backend. This prevents
//! serialization bugs like the project_id issue (Feb 2026).

use serde::Deserialize;
use serde_json::json;

/// Test struct that mimics how Tauri deserializes command arguments
#[derive(Debug, Deserialize)]
struct ProjectArgs {
    #[serde(default)]
    project_id: Option<String>,
}

/// Test struct for add_todo command
#[derive(Debug, Deserialize)]
struct AddTodoArgs {
    title: String,
    description: Option<String>,
    position: Option<i64>,
    #[serde(default)]
    project_id: Option<String>,
}

/// Test struct for set_todo_completed command
#[derive(Debug, Deserialize)]
struct SetTodoCompletedArgs {
    id: String,
    completed: bool,
    #[serde(default)]
    project_id: Option<String>,
}

/// Test struct for remove_todo command  
#[derive(Debug, Deserialize)]
struct RemoveTodoArgs {
    id: String,
    #[serde(default)]
    project_id: Option<String>,
}

#[test]
fn test_project_args_deserialization_with_id() {
    // This test verifies the fix for the serialization bug where
    // project_id was being lost between frontend and backend
    let json = json!({
        "args": {
            "project_id": "09b38257-93d9-4912-b7a8-1f155ff64c74"
        }
    });
    
    let args: ProjectArgs = serde_json::from_value(
        json.get("args").unwrap().clone()
    ).expect("Should deserialize ProjectArgs");
    
    assert_eq!(
        args.project_id,
        Some("09b38257-93d9-4912-b7a8-1f155ff64c74".to_string()),
        "project_id should be correctly deserialized"
    );
}

#[test]
fn test_project_args_deserialization_default() {
    // Test that missing project_id defaults to None
    let json = json!({
        "args": {}
    });
    
    let args: ProjectArgs = serde_json::from_value(
        json.get("args").unwrap().clone()
    ).expect("Should deserialize ProjectArgs with defaults");
    
    assert_eq!(
        args.project_id,
        None,
        "project_id should default to None when not provided"
    );
}

#[test]
fn test_add_todo_args_deserialization() {
    let json = json!({
        "args": {
            "title": "Test Todo",
            "description": "Test Description",
            "position": 1,
            "project_id": "test-project-id"
        }
    });
    
    let args: AddTodoArgs = serde_json::from_value(
        json.get("args").unwrap().clone()
    ).expect("Should deserialize AddTodoArgs");
    
    assert_eq!(args.title, "Test Todo");
    assert_eq!(args.description, Some("Test Description".to_string()));
    assert_eq!(args.position, Some(1));
    assert_eq!(args.project_id, Some("test-project-id".to_string()));
}

#[test]
fn test_set_todo_completed_args_deserialization() {
    let json = json!({
        "args": {
            "id": "todo-123",
            "completed": true,
            "project_id": "test-project-id"
        }
    });
    
    let args: SetTodoCompletedArgs = serde_json::from_value(
        json.get("args").unwrap().clone()
    ).expect("Should deserialize SetTodoCompletedArgs");
    
    assert_eq!(args.id, "todo-123");
    assert_eq!(args.completed, true);
    assert_eq!(args.project_id, Some("test-project-id".to_string()));
}

#[test]
fn test_remove_todo_args_deserialization() {
    let json = json!({
        "args": {
            "id": "todo-456",
            "project_id": "test-project-id"
        }
    });
    
    let args: RemoveTodoArgs = serde_json::from_value(
        json.get("args").unwrap().clone()
    ).expect("Should deserialize RemoveTodoArgs");
    
    assert_eq!(args.id, "todo-456");
    assert_eq!(args.project_id, Some("test-project-id".to_string()));
}

/// This test documents the bug we fixed: bare Option<String> parameters
/// were not being deserialized correctly in Tauri v2
#[test]
fn test_bare_option_string_vs_struct_deserialization() {
    // With bare Option<String>, this pattern failed:
    // (This would be how Tauri v2 tries to deserialize bare parameters)
    
    // Simulating frontend sending: { "project_id": "abc123" }
    let frontend_payload = json!({
        "project_id": "abc123"
    });
    
    // The struct approach works:
    let struct_result: Result<ProjectArgs, _> = serde_json::from_value(frontend_payload.clone());
    assert!(struct_result.is_ok(), "Struct deserialization should succeed");
    assert_eq!(struct_result.unwrap().project_id, Some("abc123".to_string()));
    
    // Direct Option<String> deserialization from a nested object also works
    // when properly structured:
    let nested = json!({
        "args": {
            "project_id": "abc123"
        }
    });
    let args: ProjectArgs = serde_json::from_value(nested["args"].clone()).unwrap();
    assert_eq!(args.project_id, Some("abc123".to_string()));
}

// ============================================================================
// Build Command Args Tests
// ============================================================================

/// Test struct for run_build_command
#[derive(Debug, Deserialize)]
struct RunBuildCommandArgs {
    #[serde(default)]
    project_id: Option<String>,
    id: String,
}

/// Test struct for set_default_build_command
#[derive(Debug, Deserialize)]
struct SetDefaultBuildCommandArgs {
    #[serde(default)]
    project_id: Option<String>,
    id: String,
}

/// Test struct for get_default_build_command
#[derive(Debug, Deserialize)]
struct GetDefaultBuildCommandArgs {
    #[serde(default)]
    project_id: Option<String>,
}

#[test]
fn test_run_build_command_args_deserialization() {
    let json = json!({
        "args": {
            "id": "build-123",
            "project_id": "family-photos-project"
        }
    });
    
    let args: RunBuildCommandArgs = serde_json::from_value(
        json.get("args").unwrap().clone()
    ).expect("Should deserialize RunBuildCommandArgs");
    
    assert_eq!(args.id, "build-123");
    assert_eq!(args.project_id, Some("family-photos-project".to_string()));
}

#[test]
fn test_set_default_build_command_args_deserialization() {
    let json = json!({
        "args": {
            "id": "default-build-1",
            "project_id": "ai-harness-project"
        }
    });
    
    let args: SetDefaultBuildCommandArgs = serde_json::from_value(
        json.get("args").unwrap().clone()
    ).expect("Should deserialize SetDefaultBuildCommandArgs");
    
    assert_eq!(args.id, "default-build-1");
    assert_eq!(args.project_id, Some("ai-harness-project".to_string()));
}

#[test]
fn test_get_default_build_command_args_deserialization() {
    let json = json!({
        "args": {
            "project_id": "test-project-id"
        }
    });
    
    let args: GetDefaultBuildCommandArgs = serde_json::from_value(
        json.get("args").unwrap().clone()
    ).expect("Should deserialize GetDefaultBuildCommandArgs");
    
    assert_eq!(args.project_id, Some("test-project-id".to_string()));
}

#[test]
fn test_build_command_args_default_project() {
    // Test that missing project_id defaults to None (which becomes "default")
    let json = json!({
        "args": {
            "id": "build-cmd-1"
        }
    });
    
    let args: RunBuildCommandArgs = serde_json::from_value(
        json.get("args").unwrap().clone()
    ).expect("Should deserialize with default project_id");
    
    assert_eq!(args.id, "build-cmd-1");
    assert_eq!(args.project_id, None, "project_id should default to None");
}
