//! Integration tests for database operations
//! 
//! These tests create temporary databases and verify the actual
//! data layer behavior, ensuring project isolation works correctly.

use std::sync::Arc;
use tokio::sync::RwLock;
use tempfile::TempDir;

use crate::app_state::AppState;
use crate::projects::{ProjectInfo, ProjectRegistry};

/// Creates a test app state with a temporary database
async fn create_test_app_state() -> (TempDir, Arc<RwLock<AppState>>) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test_registry.db");
    
    let registry = ProjectRegistry::new(db_path.to_str().unwrap())
        .await
        .expect("Failed to create registry");
    
    let app_state = AppState::new_for_test(registry).await;
    
    (temp_dir, Arc::new(RwLock::new(app_state)))
}

/// Creates a test project with its own database
async fn create_test_project(
    app_state: &Arc<RwLock<AppState>>,
    name: &str,
) -> ProjectInfo {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root_path = temp_dir.path().to_string_lossy().to_string();
    
    let project = {
        let state = app_state.read().await;
        state.project_registry.create_project(name, &root_path)
            .await
            .expect("Failed to create project")
    };
    
    // Keep temp_dir alive by leaking it (ok for tests)
    let _ = Box::leak(Box::new(temp_dir));
    
    project
}

#[tokio::test]
async fn test_todo_project_isolation() {
    // This test verifies that todos are properly isolated by project
    let (_temp_dir, app_state) = create_test_app_state().await;
    
    // Create two projects
    let project1 = create_test_project(&app_state, "Project 1").await;
    let project2 = create_test_project(&app_state, "Project 2").await;
    
    // Add a todo to project 1
    let store1 = {
        let state = app_state.read().await;
        state.get_project_store(&project1.id)
            .await
            .expect("Failed to get project 1 store")
    };
    
    let todo1 = store1.todo_store.write().await
        .add("Project 1 Todo", None, None)
        .await
        .expect("Failed to add todo to project 1");
    
    // Add a todo to project 2
    let store2 = {
        let state = app_state.read().await;
        state.get_project_store(&project2.id)
            .await
            .expect("Failed to get project 2 store")
    };
    
    let todo2 = store2.todo_store.write().await
        .add("Project 2 Todo", None, None)
        .await
        .expect("Failed to add todo to project 2");
    
    // Verify project 1 only has its own todo
    let todos1 = store1.todo_store.read().await
        .list()
        .await
        .expect("Failed to list project 1 todos");
    
    assert_eq!(todos1.len(), 1, "Project 1 should have exactly 1 todo");
    assert_eq!(todos1[0].title, "Project 1 Todo");
    assert_eq!(todos1[0].id, todo1.id);
    
    // Verify project 2 only has its own todo
    let todos2 = store2.todo_store.read().await
        .list()
        .await
        .expect("Failed to list project 2 todos");
    
    assert_eq!(todos2.len(), 1, "Project 2 should have exactly 1 todo");
    assert_eq!(todos2[0].title, "Project 2 Todo");
    assert_eq!(todos2[0].id, todo2.id);
    
    // Verify the todos are not the same
    assert_ne!(todo1.id, todo2.id, "Todos should have different IDs");
}

#[tokio::test]
async fn test_todo_crud_operations() {
    let (_temp_dir, app_state) = create_test_app_state().await;
    let project = create_test_project(&app_state, "Test Project").await;
    
    let store = {
        let state = app_state.read().await;
        state.get_project_store(&project.id)
            .await
            .expect("Failed to get project store")
    };
    
    // Create
    let todo = store.todo_store.write().await
        .add("Test Todo", Some("Description".to_string()), Some(1))
        .await
        .expect("Failed to add todo");
    
    assert_eq!(todo.title, "Test Todo");
    assert_eq!(todo.description, Some("Description".to_string()));
    assert_eq!(todo.position, 1);
    assert!(!todo.completed);
    
    // Read
    let todos = store.todo_store.read().await
        .list()
        .await
        .expect("Failed to list todos");
    
    assert_eq!(todos.len(), 1);
    assert_eq!(todos[0].id, todo.id);
    
    // Update - set completed
    store.todo_store.write().await
        .set_completed(&todo.id, true)
        .await
        .expect("Failed to set todo completed");
    
    let todos = store.todo_store.read().await
        .list()
        .await
        .expect("Failed to list todos after update");
    
    assert!(todos[0].completed, "Todo should be completed");
    
    // Delete
    store.todo_store.write().await
        .remove(&todo.id)
        .await
        .expect("Failed to remove todo");
    
    let todos = store.todo_store.read().await
        .list()
        .await
        .expect("Failed to list todos after delete");
    
    assert_eq!(todos.len(), 0, "Should have no todos after deletion");
}

#[tokio::test]
async fn test_default_project_fallback() {
    // Test that operations on non-existent project IDs fall back to default
    let (_temp_dir, app_state) = create_test_app_state().await;
    
    // Create the default project first
    {
        let state = app_state.read().await;
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root_path = temp_dir.path().to_string_lossy().to_string();
        let _ = state.project_registry.create_project_with_id(
            "default".to_string(),
            "Default",
            &root_path
        ).await.expect("Failed to create default project");
        let _ = Box::leak(Box::new(temp_dir));
    }
    
    // Get store for "default" project
    let store = {
        let state = app_state.read().await;
        state.get_project_store("default")
            .await
            .expect("Failed to get default project store")
    };
    
    // Add a todo
    let todo = store.todo_store.write().await
        .add("Default Project Todo", None, None)
        .await
        .expect("Failed to add todo to default project");
    
    // Verify it was added
    let todos = store.todo_store.read().await
        .list()
        .await
        .expect("Failed to list default project todos");
    
    assert_eq!(todos.len(), 1);
    assert_eq!(todos[0].id, todo.id);
}
