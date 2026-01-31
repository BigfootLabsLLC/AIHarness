//! AIHarness - MCP Server Library
//! 
//! This library provides the core functionality for the AIHarness MCP server,
//! including tool definitions, context management, and MCP protocol implementation.

#![warn(clippy::all, clippy::pedantic)]

pub mod context;
pub mod error;
pub mod mcp;
pub mod tools;

use std::sync::Arc;
use tauri::Manager;
use tokio::sync::RwLock;

/// Application state shared across the Tauri app
pub struct AppState {
    /// The MCP server instance
    pub mcp_server: Arc<RwLock<mcp::McpServer>>,
    /// Context store for managing files
    pub context_store: Arc<RwLock<context::ContextStore>>,
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
        
        let mcp_server = Arc::new(RwLock::new(
            mcp::McpServer::new(context_store.clone())
        ));
        
        Ok(Self {
            mcp_server,
            context_store,
        })
    }
}

/// Initialize the Tauri application
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let handle = app.handle().clone();
            
            tokio::spawn(async move {
                let app_dir = handle.path().app_data_dir().unwrap();
                let db_path = app_dir.join("aiharness.db");
                
                match AppState::new(db_path.to_str().unwrap()).await {
                    Ok(state) => {
                        tracing::info!("Application state initialized successfully");
                        // Store state in app handle for access from commands
                        handle.manage(state);
                    }
                    Err(e) => {
                        tracing::error!("Failed to initialize app state: {}", e);
                    }
                }
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
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

    #[tokio::test]
    async fn app_state_components_are_initialized() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let state = AppState::new(db_path.to_str().unwrap()).await.unwrap();
        
        // Verify we can acquire locks
        let _mcp = state.mcp_server.read().await;
        let _context = state.context_store.read().await;
    }

    #[tokio::test]
    async fn app_state_handles_concurrent_access() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let state = Arc::new(AppState::new(db_path.to_str().unwrap()).await.unwrap());
        
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let state = state.clone();
                tokio::spawn(async move {
                    let _ = state.mcp_server.read().await;
                    let _ = state.context_store.read().await;
                })
            })
            .collect();
        
        for handle in handles {
            handle.await.unwrap();
        }
    }

    #[tokio::test]
    async fn app_state_persists_across_instances() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        // First instance
        {
            let _state = AppState::new(db_path.to_str().unwrap()).await.unwrap();
        }
        
        // Second instance should connect to same database
        let state2 = AppState::new(db_path.to_str().unwrap()).await;
        assert!(state2.is_ok());
    }
}
