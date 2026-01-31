//! AIHarness MCP Server Binary
//!
//! Standalone MCP server that can be run as a subprocess.
//! Communicates via JSON-RPC over stdio.

use std::sync::Arc;
use tokio::sync::RwLock;

use aiharness_lib::context::ContextStore;
use aiharness_lib::mcp::McpServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Get database path from env or use default
    let db_path = std::env::var("AIH_DATA_DIR")
        .map(|dir| format!("{}/aiharness.db", dir))
        .unwrap_or_else(|_| "aiharness.db".to_string());

    // Create context store
    let context_store = Arc::new(RwLock::new(
        ContextStore::new(&db_path).await?,
    ));

    // Create and run MCP server
    let mut server = McpServer::new(context_store);
    
    tracing::info!("AIHarness MCP Server starting...");
    server.run_stdio().await?;
    tracing::info!("AIHarness MCP Server shutting down...");

    Ok(())
}
