//! AIHarness - Single binary with built-in HTTP server
//! 
//! Architecture:
//! - One process: GUI + HTTP server + tools
//! - Shared state: context, tool registry, event history
//! - Events flow directly from tool execution to UI

#![warn(clippy::all, clippy::pedantic)]

pub mod app_state;
pub mod build_commands;
pub mod context;
pub mod context_notes;
pub mod error;
pub mod http_server;
pub mod mcp_config;
pub mod mcp_proxy;
pub mod next_session;
pub mod projects;
pub mod todos;
pub mod tools;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{Manager, Emitter};
use tokio::sync::RwLock;

pub use app_state::AppState;

/// Tool call event for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallEvent {
    pub id: String,
    pub timestamp: String,
    pub tool_name: String,
    pub project_id: String,
    pub arguments: serde_json::Value,
    pub success: bool,
    pub content: String,
    pub duration_ms: u64,
}

/// Raw log event for debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawLogEvent {
    pub timestamp: String,
    pub source: String,
    pub message: String,
}

/// Server status
#[derive(Debug, Clone, Serialize)]
pub struct ServerStatus {
    pub running: bool,
    pub port: u16,
}

/// Context file info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextFileInfo {
    pub id: String,
    pub path: String,
    pub name: String,
    pub added_at: String,
    pub last_read_at: Option<String>,
}

/// Context note info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextNoteInfo {
    pub id: String,
    pub content: String,
    pub position: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// Project info for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub id: String,
    pub name: String,
    pub root_path: String,
    pub db_path: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Directory entry info for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryEntryInfo {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
}

/// Directory listing for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryListingInfo {
    pub path: String,
    pub parent_path: Option<String>,
    pub entries: Vec<DirectoryEntryInfo>,
}

/// Build command info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildCommandInfo {
    pub id: String,
    pub name: String,
    pub command: String,
    pub working_dir: Option<String>,
    pub is_default: bool,
    pub created_at: String,
}

/// Todo item for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItemInfo {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub position: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// Start the HTTP server
#[tauri::command]
async fn start_server(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
    app_handle: tauri::AppHandle,
) -> Result<ServerStatus, String> {
    // Check if already running
    {
        let state_read = state.read().await;
        if state_read.is_server_running().await {
            let port = state_read.get_port().await;
            return Ok(ServerStatus { running: true, port });
        }
    }
    
    // Get port
    let port = {
        let state_read = state.read().await;
        state_read.get_port().await
    };
    
    // Clone the Arc for the server
    let server_state = Arc::clone(&state);
    
    // Start HTTP server
    let handle = http_server::start_http_server(server_state, port)
        .await
        .map_err(|e| format!("Failed to start server: {}", e))?;
    
    // Store the handle
    {
    let state_write = state.write().await;
    state_write.set_server_handle(handle).await;
    }
    
    // Emit startup event
    let startup_event = RawLogEvent {
        timestamp: chrono::Utc::now().to_rfc3339(),
        source: "server".to_string(),
        message: format!("HTTP server started on port {}", port),
    };
    app_handle.emit("raw-log", &startup_event).ok();
    
    tracing::info!("HTTP server started on port {}", port);
    
    Ok(ServerStatus { running: true, port })
}

/// Stop the HTTP server
#[tauri::command]
async fn stop_server(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
) -> Result<ServerStatus, String> {
    let state = state.write().await;
    state.stop_server().await;
    Ok(ServerStatus { running: false, port: 0 })
}

/// Get server status
#[tauri::command]
async fn get_server_status(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
) -> Result<ServerStatus, String> {
    let state = state.read().await;
    let running = state.is_server_running().await;
    let port = state.get_port().await;
    Ok(ServerStatus { running, port })
}

/// Execute a tool directly
#[tauri::command]
async fn execute_tool(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
    app_handle: tauri::AppHandle,
    tool_name: String,
    arguments: serde_json::Value,
    project_id: Option<String>,
) -> Result<String, String> {
    use std::time::Instant;
    use uuid::Uuid;
    
    let state = state.read().await;
    let start = Instant::now();
    let call_id = Uuid::new_v4().to_string();
    let timestamp = chrono::Utc::now().to_rfc3339();
    
    // Get tool
    let tool = state.tool_registry
        .get(&tool_name)
        .ok_or_else(|| format!("Tool not found: {}", tool_name))?;
    
    // Execute
    let result = tool.execute(arguments.clone()).await;
    let duration_ms = start.elapsed().as_millis() as u64;
    
    // Create and record event
    let project_id = project_id.unwrap_or_else(|| "default".to_string());
    let event = match &result {
        Ok(output) => ToolCallEvent {
            id: call_id.clone(),
            timestamp: timestamp.clone(),
            tool_name: tool_name.clone(),
            project_id: project_id.clone(),
            arguments: arguments.clone(),
            success: true,
            content: output.content.clone(),
            duration_ms,
        },
        Err(e) => ToolCallEvent {
            id: call_id.clone(),
            timestamp: timestamp.clone(),
            tool_name: tool_name.clone(),
            project_id: project_id.clone(),
            arguments: arguments.clone(),
            success: false,
            content: e.to_string(),
            duration_ms,
        },
    };
    
    // Record event (broadcasts to UI)
    state.record_event(event).await;
    
    // Also emit raw log event
    let raw_event = RawLogEvent {
        timestamp,
        source: "tool".to_string(),
        message: serde_json::json!({
            "event": "tool_call_end",
            "id": call_id,
            "tool_name": tool_name,
            "success": result.is_ok(),
            "duration_ms": duration_ms
        }).to_string(),
    };
    app_handle.emit("raw-log", &raw_event).ok();
    
    match result {
        Ok(output) => Ok(output.content),
        Err(e) => Err(e.to_string()),
    }
}

/// Get event history
#[tauri::command]
async fn get_event_history(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
    project_id: Option<String>,
) -> Result<Vec<ToolCallEvent>, String> {
    let state = state.read().await;
    let history = state.get_history().await;
    let project_id = project_id.unwrap_or_else(|| "default".to_string());
    Ok(history
        .into_iter()
        .filter(|event| event.project_id == project_id)
        .collect())
}

#[tauri::command]
async fn list_projects(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
) -> Result<Vec<ProjectInfo>, String> {
    let state = state.read().await;
    let projects = state
        .project_registry
        .list_projects()
        .await
        .map_err(|e| e.to_string())?;
    Ok(projects
        .into_iter()
        .map(|p| ProjectInfo {
            id: p.id,
            name: p.name,
            root_path: p.root_path,
            db_path: p.db_path,
            created_at: p.created_at.to_rfc3339(),
            updated_at: p.updated_at.to_rfc3339(),
        })
        .collect())
}

#[tauri::command]
async fn create_project(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
    name: String,
    root_path: String,
) -> Result<ProjectInfo, String> {
    let state = state.read().await;
    let project = state
        .project_registry
        .create_project(&name, &root_path)
        .await
        .map_err(|e| e.to_string())?;
    Ok(ProjectInfo {
        id: project.id,
        name: project.name,
        root_path: project.root_path,
        db_path: project.db_path,
        created_at: project.created_at.to_rfc3339(),
        updated_at: project.updated_at.to_rfc3339(),
    })
}

#[tauri::command]
async fn list_todos(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
    project_id: Option<String>,
) -> Result<Vec<TodoItemInfo>, String> {
    let project_id = project_id.unwrap_or_else(|| "default".to_string());
    let store = {
        let state_read = state.read().await;
        state_read
            .get_project_store(&project_id)
            .await
            .map_err(|e| e.to_string())?
    };
    let todos = store
        .todo_store
        .read()
        .await
        .list()
        .await
        .map_err(|e| e.to_string())?;
    let items = todos.into_iter().map(todo_info_from).collect();
    Ok(items)
}

#[tauri::command]
async fn add_todo(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
    title: String,
    description: Option<String>,
    position: Option<i64>,
    project_id: Option<String>,
) -> Result<TodoItemInfo, String> {
    let project_id = project_id.unwrap_or_else(|| "default".to_string());
    let store = {
        let state_read = state.read().await;
        state_read
            .get_project_store(&project_id)
            .await
            .map_err(|e| e.to_string())?
    };
    let todo = store
        .todo_store
        .read()
        .await
        .add(&title, description, position)
        .await
        .map_err(|e| e.to_string())?;
    let info = todo_info_from(todo);
    Ok(info)
}

#[tauri::command]
async fn set_todo_completed(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
    id: String,
    completed: bool,
    project_id: Option<String>,
) -> Result<(), String> {
    let project_id = project_id.unwrap_or_else(|| "default".to_string());
    let store = {
        let state_read = state.read().await;
        state_read
            .get_project_store(&project_id)
            .await
            .map_err(|e| e.to_string())?
    };
    let result = store
        .todo_store
        .read()
        .await
        .set_completed(&id, completed)
        .await
        .map_err(|e| e.to_string());
    result
}

#[tauri::command]
async fn remove_todo(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
    id: String,
    project_id: Option<String>,
) -> Result<(), String> {
    let project_id = project_id.unwrap_or_else(|| "default".to_string());
    let store = {
        let state_read = state.read().await;
        state_read
            .get_project_store(&project_id)
            .await
            .map_err(|e| e.to_string())?
    };
    let result = store
        .todo_store
        .read()
        .await
        .remove(&id)
        .await
        .map_err(|e| e.to_string());
    result
}

#[tauri::command]
async fn move_todo(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
    id: String,
    position: i64,
    project_id: Option<String>,
) -> Result<(), String> {
    let project_id = project_id.unwrap_or_else(|| "default".to_string());
    let store = {
        let state_read = state.read().await;
        state_read
            .get_project_store(&project_id)
            .await
            .map_err(|e| e.to_string())?
    };
    let result = store
        .todo_store
        .read()
        .await
        .move_to(&id, position)
        .await
        .map_err(|e| e.to_string());
    result
}

#[tauri::command]
async fn get_next_todo(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
    project_id: Option<String>,
) -> Result<Option<TodoItemInfo>, String> {
    let project_id = project_id.unwrap_or_else(|| "default".to_string());
    let store = {
        let state_read = state.read().await;
        state_read
            .get_project_store(&project_id)
            .await
            .map_err(|e| e.to_string())?
    };
    let todo = store
        .todo_store
        .read()
        .await
        .get_next()
        .await
        .map_err(|e| e.to_string())?;
    let info = todo.map(todo_info_from);
    Ok(info)
}

fn todo_info_from(todo: crate::todos::TodoItem) -> TodoItemInfo {
    TodoItemInfo {
        id: todo.id,
        title: todo.title,
        description: todo.description,
        completed: todo.completed,
        position: todo.position,
        created_at: todo.created_at.to_rfc3339(),
        updated_at: todo.updated_at.to_rfc3339(),
    }
}

/// Add context file
#[tauri::command]
async fn add_context_file(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
    path: String,
    project_id: Option<String>,
) -> Result<ContextFileInfo, String> {
    let project_id = project_id.unwrap_or_else(|| "default".to_string());
    let store = {
        let state_read = state.read().await;
        state_read
            .get_project_store(&project_id)
            .await
            .map_err(|e| e.to_string())?
    };
    let store = store.context_store.read().await;
    let file = store.add_file(&path).await.map_err(|e| e.to_string())?;
    
    let name = std::path::Path::new(&file.path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    
    Ok(ContextFileInfo {
        id: file.id,
        path: file.path,
        name,
        added_at: file.added_at.to_rfc3339(),
        last_read_at: file.last_read_at.map(|d| d.to_rfc3339()),
    })
}

/// Remove context file
#[tauri::command]
async fn remove_context_file(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
    path: String,
    project_id: Option<String>,
) -> Result<(), String> {
    let project_id = project_id.unwrap_or_else(|| "default".to_string());
    let store = {
        let state_read = state.read().await;
        state_read
            .get_project_store(&project_id)
            .await
            .map_err(|e| e.to_string())?
    };
    let store = store.context_store.read().await;
    store.remove_file(&path).await.map_err(|e| e.to_string())
}

/// List context files
#[tauri::command]
async fn list_context_files(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
    project_id: Option<String>,
) -> Result<Vec<ContextFileInfo>, String> {
    let project_id = project_id.unwrap_or_else(|| "default".to_string());
    let store = {
        let state_read = state.read().await;
        state_read
            .get_project_store(&project_id)
            .await
            .map_err(|e| e.to_string())?
    };
    let store = store.context_store.read().await;
    let files = store.list_files().await.map_err(|e| e.to_string())?;
    
    let infos: Vec<ContextFileInfo> = files
        .into_iter()
        .map(|file| {
            let name = std::path::Path::new(&file.path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            
            ContextFileInfo {
                id: file.id,
                path: file.path,
                name,
                added_at: file.added_at.to_rfc3339(),
                last_read_at: file.last_read_at.map(|d| d.to_rfc3339()),
            }
        })
        .collect();
    
    Ok(infos)
}

/// List context notes
#[tauri::command]
async fn list_context_notes(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
    project_id: Option<String>,
) -> Result<Vec<ContextNoteInfo>, String> {
    let project_id = project_id.unwrap_or_else(|| "default".to_string());
    let store = {
        let state_read = state.read().await;
        state_read
            .get_project_store(&project_id)
            .await
            .map_err(|e| e.to_string())?
    };
    let store = store.context_note_store.read().await;
    let notes = store.list().await.map_err(|e| e.to_string())?;
    Ok(notes.into_iter().map(context_note_info_from).collect())
}

/// Add context note
#[tauri::command]
async fn add_context_note(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
    project_id: Option<String>,
    content: String,
    position: Option<i64>,
) -> Result<ContextNoteInfo, String> {
    let project_id = project_id.unwrap_or_else(|| "default".to_string());
    let store = {
        let state_read = state.read().await;
        state_read
            .get_project_store(&project_id)
            .await
            .map_err(|e| e.to_string())?
    };
    let store = store.context_note_store.read().await;
    let note = store
        .add(&content, position)
        .await
        .map_err(|e| e.to_string())?;
    Ok(context_note_info_from(note))
}

/// Update context note
#[tauri::command]
async fn update_context_note(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
    project_id: Option<String>,
    id: String,
    content: String,
) -> Result<(), String> {
    let project_id = project_id.unwrap_or_else(|| "default".to_string());
    let store = {
        let state_read = state.read().await;
        state_read
            .get_project_store(&project_id)
            .await
            .map_err(|e| e.to_string())?
    };
    let store = store.context_note_store.read().await;
    store.update(&id, &content).await.map_err(|e| e.to_string())
}

/// Remove context note
#[tauri::command]
async fn remove_context_note(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
    project_id: Option<String>,
    id: String,
) -> Result<(), String> {
    let project_id = project_id.unwrap_or_else(|| "default".to_string());
    let store = {
        let state_read = state.read().await;
        state_read
            .get_project_store(&project_id)
            .await
            .map_err(|e| e.to_string())?
    };
    let store = store.context_note_store.read().await;
    store.remove(&id).await.map_err(|e| e.to_string())
}

/// Move context note
#[tauri::command]
async fn move_context_note(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
    project_id: Option<String>,
    id: String,
    position: i64,
) -> Result<(), String> {
    let project_id = project_id.unwrap_or_else(|| "default".to_string());
    let store = {
        let state_read = state.read().await;
        state_read
            .get_project_store(&project_id)
            .await
            .map_err(|e| e.to_string())?
    };
    let store = store.context_note_store.read().await;
    store.move_to(&id, position).await.map_err(|e| e.to_string())
}

fn context_note_info_from(note: crate::context_notes::ContextNote) -> ContextNoteInfo {
    ContextNoteInfo {
        id: note.id,
        content: note.content,
        position: note.position,
        created_at: note.created_at.to_rfc3339(),
        updated_at: note.updated_at.to_rfc3339(),
    }
}

/// List build commands
#[tauri::command]
async fn list_build_commands(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
    project_id: Option<String>,
) -> Result<Vec<BuildCommandInfo>, String> {
    let project_id = project_id.unwrap_or_else(|| "default".to_string());
    let store = {
        let state_read = state.read().await;
        state_read
            .get_project_store(&project_id)
            .await
            .map_err(|e| e.to_string())?
    };
    let store = store.build_command_store.read().await;
    let commands = store.list().await.map_err(|e| e.to_string())?;
    Ok(commands
        .into_iter()
        .map(build_command_info_from)
        .collect())
}

/// Add build command
#[tauri::command]
async fn add_build_command(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
    project_id: Option<String>,
    name: String,
    command: String,
    working_dir: Option<String>,
) -> Result<BuildCommandInfo, String> {
    let project_id = project_id.unwrap_or_else(|| "default".to_string());
    let store = {
        let state_read = state.read().await;
        state_read
            .get_project_store(&project_id)
            .await
            .map_err(|e| e.to_string())?
    };
    let store = store.build_command_store.read().await;
    let command = store
        .add(&name, &command, working_dir)
        .await
        .map_err(|e| e.to_string())?;
    Ok(build_command_info_from(command))
}

/// Remove build command
#[tauri::command]
async fn remove_build_command(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
    project_id: Option<String>,
    id: String,
) -> Result<(), String> {
    let project_id = project_id.unwrap_or_else(|| "default".to_string());
    let store = {
        let state_read = state.read().await;
        state_read
            .get_project_store(&project_id)
            .await
            .map_err(|e| e.to_string())?
    };
    let store = store.build_command_store.read().await;
    store.remove(&id).await.map_err(|e| e.to_string())
}

/// Run build command
#[tauri::command]
async fn run_build_command(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
    project_id: Option<String>,
    id: String,
) -> Result<String, String> {
    let project_id = project_id.unwrap_or_else(|| "default".to_string());
    let (command, root_path) = {
        let state_read = state.read().await;
        let project = state_read
            .project_registry
            .get_project(&project_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Unknown project: {}", project_id))?;
        let store = state_read
            .get_project_store(&project_id)
            .await
            .map_err(|e| e.to_string())?;
        let store = store.build_command_store.read().await;
        let command = store
            .get(&id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Build command not found".to_string())?;
        let working_dir = command
            .working_dir
            .clone()
            .unwrap_or_else(|| project.root_path.clone());
        (command.command, working_dir)
    };

    run_shell_command(&command, &root_path).await
}

/// Set default build command
#[tauri::command]
async fn set_default_build_command(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
    project_id: Option<String>,
    id: String,
) -> Result<(), String> {
    let project_id = project_id.unwrap_or_else(|| "default".to_string());
    let store = {
        let state_read = state.read().await;
        state_read
            .get_project_store(&project_id)
            .await
            .map_err(|e| e.to_string())?
    };
    let store = store.build_command_store.read().await;
    store.set_default(&id).await.map_err(|e| e.to_string())
}

/// Get default build command
#[tauri::command]
async fn get_default_build_command(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
    project_id: Option<String>,
) -> Result<Option<BuildCommandInfo>, String> {
    let project_id = project_id.unwrap_or_else(|| "default".to_string());
    let store = {
        let state_read = state.read().await;
        state_read
            .get_project_store(&project_id)
            .await
            .map_err(|e| e.to_string())?
    };
    let store = store.build_command_store.read().await;
    let command = store.get_default().await.map_err(|e| e.to_string())?;
    Ok(command.map(build_command_info_from))
}

fn build_command_info_from(command: crate::build_commands::BuildCommand) -> BuildCommandInfo {
    BuildCommandInfo {
        id: command.id,
        name: command.name,
        command: command.command,
        working_dir: command.working_dir,
        is_default: command.is_default,
        created_at: command.created_at.to_rfc3339(),
    }
}

pub(crate) async fn run_shell_command(command: &str, working_dir: &str) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    let mut cmd = {
        let mut cmd = tokio::process::Command::new("cmd");
        cmd.arg("/C").arg(command);
        cmd
    };

    #[cfg(not(target_os = "windows"))]
    let mut cmd = {
        let mut cmd = tokio::process::Command::new("sh");
        cmd.arg("-lc").arg(command);
        cmd
    };

    cmd.current_dir(working_dir);

    let output = cmd
        .output()
        .await
        .map_err(|e| format!("Failed to run command: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let combined = if stderr.is_empty() {
        stdout.clone()
    } else if stdout.is_empty() {
        stderr.clone()
    } else {
        format!("{}\n{}", stdout, stderr)
    };

    if output.status.success() {
        Ok(combined)
    } else {
        Err(format!("Command failed ({}): {}", output.status, combined))
    }
}

/// List a project's directory contents (relative to project root).
#[tauri::command]
async fn list_project_directory(
    state: tauri::State<'_, Arc<RwLock<AppState>>>,
    project_id: Option<String>,
    sub_path: Option<String>,
) -> Result<DirectoryListingInfo, String> {
    let project_id = project_id.unwrap_or_else(|| "default".to_string());
    let project = {
        let state_read = state.read().await;
        state_read
            .project_registry
            .get_project(&project_id)
            .await
            .map_err(|e| e.to_string())?
    }
    .ok_or_else(|| format!("Unknown project: {}", project_id))?;

    let root = std::path::PathBuf::from(&project.root_path);
    let requested = sub_path.unwrap_or_else(|| "".to_string());
    let requested_path = root.join(requested);
    let canonical = std::fs::canonicalize(&requested_path)
        .map_err(|e| format!("Invalid path: {}", e))?;

    if !canonical.starts_with(&root) {
        return Err("Path is outside project root".to_string());
    }

    list_directory_impl(&canonical, Some(&root)).await
}

/// List any absolute directory contents.
#[tauri::command]
async fn list_directory(path: String) -> Result<DirectoryListingInfo, String> {
    let requested = std::path::PathBuf::from(path);
    let canonical = std::fs::canonicalize(&requested)
        .map_err(|e| format!("Invalid path: {}", e))?;
    if !canonical.is_absolute() {
        return Err("Path must be absolute".to_string());
    }
    list_directory_impl(&canonical, None).await
}

async fn list_directory_impl(
    path: &std::path::Path,
    root: Option<&std::path::Path>,
) -> Result<DirectoryListingInfo, String> {
    if !path.is_dir() {
        return Err("Path is not a directory".to_string());
    }

    let mut entries = tokio::fs::read_dir(path)
        .await
        .map_err(|e| format!("Failed to read directory: {}", e))?;
    let mut dirs: Vec<DirectoryEntryInfo> = Vec::new();
    let mut files: Vec<DirectoryEntryInfo> = Vec::new();

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| format!("Failed to read directory: {}", e))?
    {
        let file_name = entry.file_name().to_string_lossy().to_string();
        let entry_path = entry.path();
        let metadata = entry
            .metadata()
            .await
            .map_err(|e| format!("Failed to read metadata: {}", e))?;
        let is_dir = metadata.is_dir();
        let entry_info = DirectoryEntryInfo {
            name: file_name,
            path: entry_path.to_string_lossy().to_string(),
            is_dir,
        };

        if is_dir {
            dirs.push(entry_info);
        } else {
            files.push(entry_info);
        }
    }

    dirs.sort_by(|a, b| a.name.cmp(&b.name));
    files.sort_by(|a, b| a.name.cmp(&b.name));

    let mut entries = Vec::with_capacity(dirs.len() + files.len());
    entries.extend(dirs);
    entries.extend(files);

    let parent_path = path
        .parent()
        .and_then(|parent| match root {
            Some(root) if parent.starts_with(root) => Some(parent),
            Some(_) => None,
            None => Some(parent),
        })
        .map(|parent| parent.to_string_lossy().to_string());

    Ok(DirectoryListingInfo {
        path: path.to_string_lossy().to_string(),
        parent_path,
        entries,
    })
}

/// MCP Configuration DTOs
#[derive(Debug, Clone, Serialize)]
pub struct McpConfigResult {
    pub success: bool,
    pub message: String,
    pub config_path: Option<String>,
}

/// Get list of supported AI tools for MCP configuration
#[tauri::command]
async fn get_mcp_supported_tools() -> Result<Vec<mcp_config::AiToolInfo>, String> {
    Ok(mcp_config::get_mcp_config_info())
}

/// Generate MCP configuration for a specific AI tool
#[tauri::command]
async fn generate_mcp_config_for_tool(
    tool: String,
    project_name: String,
    project_id: String,
    port: u16,
) -> Result<String, String> {
    let ai_tool = match tool.as_str() {
        "claude" => mcp_config::AiTool::Claude,
        "kimi" => mcp_config::AiTool::Kimi,
        "gemini" => mcp_config::AiTool::Gemini,
        "codex" => mcp_config::AiTool::Codex,
        _ => return Err(format!("Unknown AI tool: {}", tool)),
    };

    mcp_config::generate_mcp_config(ai_tool, &project_name, &project_id, port)
        .await
        .map_err(|e| e.to_string())
}

/// Write MCP configuration for a specific AI tool
#[tauri::command]
async fn write_mcp_config_for_tool(
    tool: String,
    project_name: String,
    project_id: String,
    port: u16,
) -> Result<McpConfigResult, String> {
    let ai_tool = match tool.as_str() {
        "claude" => mcp_config::AiTool::Claude,
        "kimi" => mcp_config::AiTool::Kimi,
        "gemini" => mcp_config::AiTool::Gemini,
        "codex" => mcp_config::AiTool::Codex,
        _ => {
            return Ok(McpConfigResult {
                success: false,
                message: format!("Unknown AI tool: {}", tool),
                config_path: None,
            })
        }
    };

    let config_path = ai_tool.config_path().map_err(|e| e.to_string())?;

    match mcp_config::write_mcp_config(ai_tool, &project_name, &project_id, port).await {
        Ok(_) => Ok(McpConfigResult {
            success: true,
            message: format!("MCP configuration added for {}", tool),
            config_path: Some(config_path.to_string_lossy().to_string()),
        }),
        Err(e) => Ok(McpConfigResult {
            success: false,
            message: format!("Failed to write config: {}", e),
            config_path: Some(config_path.to_string_lossy().to_string()),
        }),
    }
}

/// Configure MCP for all supported AI tools
#[tauri::command]
async fn configure_mcp_for_all_tools(
    project_name: String,
    project_id: String,
    port: u16,
) -> Result<Vec<McpConfigResult>, String> {
    let tools = vec![
        ("claude", mcp_config::AiTool::Claude),
        ("kimi", mcp_config::AiTool::Kimi),
        ("gemini", mcp_config::AiTool::Gemini),
        ("codex", mcp_config::AiTool::Codex),
    ];

    let mut results = Vec::new();

    for (name, tool) in tools {
        let config_path = match tool.config_path() {
            Ok(path) => path,
            Err(e) => {
                results.push(McpConfigResult {
                    success: false,
                    message: format!("Failed to get config path: {}", e),
                    config_path: None,
                });
                continue;
            }
        };

        let result = match mcp_config::write_mcp_config(tool, &project_name, &project_id, port).await {
            Ok(_) => McpConfigResult {
                success: true,
                message: format!("MCP configuration added for {}", name),
                config_path: Some(config_path.to_string_lossy().to_string()),
            },
            Err(e) => McpConfigResult {
                success: false,
                message: format!("Failed to write config: {}", e),
                config_path: Some(config_path.to_string_lossy().to_string()),
            },
        };
        results.push(result);
    }

    Ok(results)
}

/// Run the Tauri application
pub fn run() {
    tracing_subscriber::fmt::init();

    let is_stdio_proxy_mode = std::env::args().any(|arg| arg == "--mcp-stdio-proxy");
    if is_stdio_proxy_mode {
        tauri::async_runtime::block_on(async {
            if let Err(e) = mcp_proxy::run_stdio_proxy().await {
                eprintln!("{}", e);
            }
        });
        return;
    }
    
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let handle = app.handle().clone();
            
            tauri::async_runtime::block_on(async move {
                let app_dir = handle.path().app_data_dir().unwrap();
                std::fs::create_dir_all(&app_dir).ok();
                
                let registry_path = app_dir.join("registry.db");
                
                match AppState::new(registry_path.to_str().unwrap(), &app_dir).await {
                    Ok(state) => {
                        tracing::info!("App state initialized");
                        let state = Arc::new(RwLock::new(state));
                        
                        // Subscribe to events and forward to Tauri
                        let mut rx = {
                            let state_read = state.read().await;
                            state_read.subscribe()
                        };
                        let app_handle = handle.clone();
                        tauri::async_runtime::spawn(async move {
                            while let Ok(event) = rx.recv().await {
                                // Emit to Tauri UI
                                let _ = app_handle.emit("tool-call", &event);
                                
                                // Also emit raw log
                                let raw_event = RawLogEvent {
                                    timestamp: event.timestamp.clone(),
                                    source: "tool".to_string(),
                                    message: serde_json::json!({
                                        "event": "tool_call_end",
                                        "id": &event.id,
                                        "tool_name": &event.tool_name,
                                        "success": event.success,
                                        "duration_ms": event.duration_ms
                                    }).to_string(),
                                };
                                let _ = app_handle.emit("raw-log", &raw_event);
                            }
                        });
                        
                        handle.manage(state.clone());

                        let app_handle = handle.clone();
                        tauri::async_runtime::spawn(async move {
                            let port = {
                                let state_read = state.read().await;
                                state_read.get_port().await
                            };

                            let already_running = {
                                let state_read = state.read().await;
                                state_read.is_server_running().await
                            };

                            if already_running {
                                return;
                            }

                            match http_server::start_http_server(state.clone(), port).await {
                                Ok(server_handle) => {
                                    {
                                        let state_write = state.write().await;
                                        state_write.set_server_handle(server_handle).await;
                                    }

                                    let startup_event = RawLogEvent {
                                        timestamp: chrono::Utc::now().to_rfc3339(),
                                        source: "server".to_string(),
                                        message: format!("HTTP server auto-started on port {}", port),
                                    };
                                    let _ = app_handle.emit("raw-log", &startup_event);
                                }
                                Err(e) => {
                                    tracing::error!("Failed to auto-start HTTP server: {}", e);
                                    let error_event = RawLogEvent {
                                        timestamp: chrono::Utc::now().to_rfc3339(),
                                        source: "server".to_string(),
                                        message: format!("HTTP server auto-start failed: {}", e),
                                    };
                                    let _ = app_handle.emit("raw-log", &error_event);
                                }
                            }
                        });
                    }
                    Err(e) => {
                        tracing::error!("Failed to initialize: {}", e);
                    }
                }
            });
            
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            start_server,
            stop_server,
            get_server_status,
            execute_tool,
            get_event_history,
            list_projects,
            create_project,
            add_context_file,
            remove_context_file,
            list_context_files,
            list_context_notes,
            add_context_note,
            update_context_note,
            remove_context_note,
            move_context_note,
            list_project_directory,
            list_directory,
            list_build_commands,
            add_build_command,
            remove_build_command,
            run_build_command,
            set_default_build_command,
            get_default_build_command,
            list_todos,
            add_todo,
            set_todo_completed,
            remove_todo,
            move_todo,
            get_next_todo,
            get_mcp_supported_tools,
            generate_mcp_config_for_tool,
            write_mcp_config_for_tool,
            configure_mcp_for_all_tools,
        ])
        .run(tauri::generate_context!())
        .expect("error running app");
}
