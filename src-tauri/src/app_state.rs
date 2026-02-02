//! Shared application state
//! 
//! Single source of truth for:
//! - Project registry + per-project stores
//! - Tool registry
//! - Event history (tool calls)
//! - HTTP server control

use crate::{
    error::ContextError,
    projects::{ProjectRegistry, ProjectStore, ProjectStoreCache},
    tools::{create_standard_registry, ToolRegistry},
    ToolCallEvent,
};
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};

/// HTTP server handle
pub type ServerHandle = tokio::task::JoinHandle<()>;

/// Shared application state - single source of truth
pub struct AppState {
    /// Project registry (global)
    pub project_registry: ProjectRegistry,
    /// Cached project stores
    pub project_stores: ProjectStoreCache,
    /// Tool registry
    pub tool_registry: ToolRegistry,
    /// Event history (tool calls)
    event_history: RwLock<Vec<ToolCallEvent>>,
    /// Event broadcaster for real-time updates
    event_sender: broadcast::Sender<ToolCallEvent>,
    /// HTTP server handle
    http_server: RwLock<Option<ServerHandle>>,
    /// HTTP server port
    http_port: RwLock<u16>,
}

impl AppState {
    /// Create new app state
    pub async fn new(registry_path: &str, app_data_dir: &std::path::Path) -> anyhow::Result<Self> {
        let project_registry = ProjectRegistry::new(registry_path).await?;
        let project_stores = ProjectStoreCache::new();
        ensure_default_project(&project_registry, &project_stores, app_data_dir).await?;
        
        let port = 8787;
        let tool_registry = create_standard_registry(port);
        let event_history = RwLock::new(Vec::new());
        let (event_sender, _) = broadcast::channel(100);
        
        Ok(Self {
            project_registry,
            project_stores,
            tool_registry,
            event_history,
            event_sender,
            http_server: RwLock::new(None),
            http_port: RwLock::new(port),
        })
    }
    
    /// Create app state for tests (no default project setup)
    #[cfg(test)]
    pub async fn new_for_test(project_registry: ProjectRegistry) -> Self {
        let project_stores = ProjectStoreCache::new();
        let port = 8787;
        let tool_registry = create_standard_registry(port);
        let event_history = RwLock::new(Vec::new());
        let (event_sender, _) = broadcast::channel(100);
        
        Self {
            project_registry,
            project_stores,
            tool_registry,
            event_history,
            event_sender,
            http_server: RwLock::new(None),
            http_port: RwLock::new(port),
        }
    }
    
    /// Record a tool call event
    pub async fn record_event(&self, event: ToolCallEvent) {
        // Add to history
        let mut history = self.event_history.write().await;
        history.insert(0, event.clone());
        history.truncate(100); // Keep last 100
        drop(history);
        
        // Broadcast to subscribers
        let _ = self.event_sender.send(event);
    }
    
    /// Get event history
    pub async fn get_history(&self) -> Vec<ToolCallEvent> {
        self.event_history.read().await.clone()
    }
    
    /// Subscribe to events
    pub fn subscribe(&self) -> broadcast::Receiver<ToolCallEvent> {
        self.event_sender.subscribe()
    }

    pub async fn get_project_store(&self, project_id: &str) -> Result<Arc<ProjectStore>, ContextError> {
        crate::debug_log(&format!("get_project_store: START project_id={}", project_id));
        
        if let Some(store) = self.project_stores.get(project_id).await {
            crate::debug_log(&format!(
                "get_project_store: CACHE HIT project_id={} db_path={}",
                project_id, store.info.db_path
            ));
            return Ok(store);
        }

        crate::debug_log(&format!("get_project_store: CACHE MISS project_id={}", project_id));
        
        let project = self
            .project_registry
            .get_project(project_id)
            .await?
            .ok_or_else(|| ContextError::NotInContext(project_id.to_string()))?;

        crate::debug_log(&format!(
            "get_project_store: CREATING project_id={} db_path={}",
            project_id, project.db_path
        ));

        let store = Arc::new(ProjectStore::new(project).await?);
        
        crate::debug_log(&format!(
            "get_project_store: INSERTING project_id={} db_path={}",
            project_id, store.info.db_path
        ));
        
        self.project_stores.insert(store.clone()).await;
        Ok(store)
    }
    
    /// Check if HTTP server is running
    pub async fn is_server_running(&self) -> bool {
        self.http_server.read().await.is_some()
    }
    
    /// Get HTTP server port
    pub async fn get_port(&self) -> u16 {
        *self.http_port.read().await
    }
    
    /// Set HTTP server port
    pub async fn set_port(&self, port: u16) {
        *self.http_port.write().await = port;
    }
    
    /// Set HTTP server handle
    pub async fn set_server_handle(&self, handle: ServerHandle) {
        *self.http_server.write().await = Some(handle);
    }
    
    /// Stop the HTTP server
    pub async fn stop_server(&self) {
        if let Some(handle) = self.http_server.write().await.take() {
            handle.abort();
            tracing::info!("HTTP server stopped");
        }
    }
}

async fn ensure_default_project(
    registry: &ProjectRegistry,
    cache: &ProjectStoreCache,
    app_data_dir: &std::path::Path,
) -> Result<(), ContextError> {
    let projects = registry.list_projects().await?;
    if projects.iter().any(|p| p.id == "default") {
        return Ok(());
    }

    let root = crate::projects::default_project_root(app_data_dir);
    std::fs::create_dir_all(&root).map_err(|e| ContextError::Database(e.to_string()))?;
    let project = registry
        .create_project_with_id("default".to_string(), "Default", root.to_str().unwrap())
        .await?;
    let store = Arc::new(ProjectStore::new(project).await?);
    cache.insert(store).await;
    Ok(())
}
