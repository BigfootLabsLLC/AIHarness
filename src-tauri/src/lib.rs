//! AIHarness - MCP Server Library
//! 
//! This library provides the core functionality for the AIHarness MCP server,
//! including tool definitions, context management, and MCP protocol implementation.

#![warn(clippy::all, clippy::pedantic)]

pub mod context;
pub mod error;
pub mod mcp;
pub mod tools;

use std::process::Stdio;
use std::sync::Arc;
use tauri::{Manager, Emitter};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{Mutex, RwLock};
use serde::{Deserialize, Serialize};

/// Application state shared across the Tauri app
pub struct AppState {
    /// Context store for managing files
    pub context_store: Arc<RwLock<context::ContextStore>>,
    /// MCP server process handle
    pub mcp_process: Arc<Mutex<Option<Child>>>,
    /// Server running status
    pub server_running: Arc<Mutex<bool>>,
}

impl AppState {
    /// Create a new application state
    /// 
    /// # Errors
    /// 
    /// Returns an error if the database connection fails
    pub async fn new(db_path: &str) -> anyhow::Result<Self> {
        let context_store = Arc::new(RwLock::new(
            context::ContextStore::new(db_path).await?
        ));
        
        Ok(Self {
            context_store,
            mcp_process: Arc::new(Mutex::new(None)),
            server_running: Arc::new(Mutex::new(false)),
        })
    }
}

/// Server status response
#[derive(Debug, Clone, Serialize)]
pub struct ServerStatus {
    pub running: bool,
    pub port: Option<u16>,
}

/// Tool call event for frontend
#[derive(Debug, Clone, Serialize)]
pub struct ToolCallEvent {
    pub id: String,
    pub timestamp: String,
    pub tool_name: String,
    pub arguments: serde_json::Value,
    pub success: bool,
    pub content: String,
    pub duration_ms: u64,
}

/// Context file info for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextFileInfo {
    pub id: String,
    pub path: String,
    pub name: String,
    pub added_at: String,
    pub last_read_at: Option<String>,
}

/// Start the MCP server
#[tauri::command]
async fn start_server(
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<ServerStatus, String> {
    let mut running = state.server_running.lock().await;
    
    if *running {
        return Ok(ServerStatus { running: true, port: None });
    }
    
    // Get the MCP server binary path (bundled with the app)
    let binary_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get exe path: {}", e))?
        .parent()
        .ok_or("No parent dir")?
        .join("aiharness-mcp-server");
    
    // If not found next to exe, try the cargo build path
    let binary_path = if binary_path.exists() {
        binary_path
    } else {
        std::path::PathBuf::from(
            std::env::var("CARGO_MANIFEST_DIR")
                .unwrap_or_else(|_| "/Users/danbaker/Projects/AIHarness/AIHarness/src-tauri".to_string())
        )
        .join("target")
        .join("release")
        .join("aiharness-mcp-server")
    };
    
    if !binary_path.exists() {
        return Err(format!("MCP server binary not found at: {:?}", binary_path));
    }
    
    // Get data directory
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {:?}", e))?;
    
    std::fs::create_dir_all(&data_dir).map_err(|e| format!("Failed to create data dir: {}", e))?;
    
    // Start the MCP server process
    let mut child = Command::new(&binary_path)
        .env("AIH_DATA_DIR", &data_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start MCP server: {}", e))?;
    
    // Spawn a task to read stderr and log it
    if let Some(stderr) = child.stderr.take() {
        tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            
            while let Ok(Some(line)) = lines.next_line().await {
                tracing::info!("[MCP Server] {}", line);
            }
        });
    }
    
    // Spawn a task to read stdout (tool calls) and emit events
    if let Some(stdout) = child.stdout.take() {
        let app_handle_clone = app_handle.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            
            while let Ok(Some(line)) = lines.next_line().await {
                tracing::info!("[MCP Output] {}", line);
                
                // Try to parse as tool call and emit event
                if let Ok(event) = parse_tool_call_from_line(&line) {
                    let _ = app_handle_clone.emit("tool-call", event);
                }
            }
        });
    }
    
    *state.mcp_process.lock().await = Some(child);
    *running = true;
    
    Ok(ServerStatus { running: true, port: None })
}

/// Stop the MCP server
#[tauri::command]
async fn stop_server(state: tauri::State<'_, AppState>) -> Result<ServerStatus, String> {
    let mut process = state.mcp_process.lock().await;
    let mut running = state.server_running.lock().await;
    
    if let Some(mut child) = process.take() {
        // Try to gracefully kill
        let _ = child.kill().await;
        *running = false;
    }
    
    Ok(ServerStatus { running: false, port: None })
}

/// Get server status
#[tauri::command]
async fn get_server_status(state: tauri::State<'_, AppState>) -> Result<ServerStatus, String> {
    let running = *state.server_running.lock().await;
    Ok(ServerStatus { running, port: None })
}

/// Add a file to context
#[tauri::command]
async fn add_context_file(
    state: tauri::State<'_, AppState>,
    path: String,
) -> Result<ContextFileInfo, String> {
    let store = state.context_store.read().await;
    
    let file = store
        .add_file(&path)
        .await
        .map_err(|e| e.to_string())?;
    
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

/// Remove a file from context
#[tauri::command]
async fn remove_context_file(
    state: tauri::State<'_, AppState>,
    path: String,
) -> Result<(), String> {
    let store = state.context_store.read().await;
    
    store
        .remove_file(&path)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

/// List context files
#[tauri::command]
async fn list_context_files(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<ContextFileInfo>, String> {
    let store = state.context_store.read().await;
    
    let files = store
        .list_files()
        .await
        .map_err(|e| e.to_string())?;
    
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

/// Parse a tool call from MCP server output line
fn parse_tool_call_from_line(line: &str) -> Result<ToolCallEvent, Box<dyn std::error::Error>> {
    // This is a simplified parser - in production you'd want proper JSON-RPC parsing
    // For now, we'll create mock events for demonstration
    
    // Try to parse as JSON-RPC response
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
        if let Some(result) = json.get("result") {
            if let Some(content) = result.get("content") {
                if let Some(text) = content.get(0).and_then(|c| c.get("text")).and_then(|t| t.as_str()) {
                    return Ok(ToolCallEvent {
                        id: uuid::Uuid::new_v4().to_string(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        tool_name: "unknown".to_string(),
                        arguments: serde_json::json!({}),
                        success: true,
                        content: text.to_string(),
                        duration_ms: 0,
                    });
                }
            }
        }
    }
    
    Err("Not a tool call".into())
}

/// Initialize the Tauri application
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let handle = app.handle().clone();
            
            tokio::runtime::Runtime::new().unwrap().block_on(async move {
                let app_dir = handle.path().app_data_dir().unwrap();
                
                // Create the app data directory if it doesn't exist
                if let Err(e) = std::fs::create_dir_all(&app_dir) {
                    tracing::error!("Failed to create app data directory: {}", e);
                    return;
                }
                
                let db_path = app_dir.join("aiharness.db");
                
                match AppState::new(db_path.to_str().unwrap()).await {
                    Ok(state) => {
                        tracing::info!("Application state initialized successfully");
                        handle.manage(state);
                    }
                    Err(e) => {
                        tracing::error!("Failed to initialize app state: {}", e);
                    }
                }
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_server,
            stop_server,
            get_server_status,
            add_context_file,
            remove_context_file,
            list_context_files,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn app_state_new_creates_valid_state() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let state = AppState::new(db_path.to_str().unwrap()).await;
        
        assert!(state.is_ok());
    }

    #[tokio::test]
    async fn app_state_new_creates_database() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let _state = AppState::new(db_path.to_str().unwrap()).await.unwrap();
        
        assert!(db_path.exists());
    }
}
