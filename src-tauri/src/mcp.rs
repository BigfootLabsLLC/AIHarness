//! MCP (Model Context Protocol) server implementation
//!
//! Implements the MCP protocol for communication with AI tools.
//! Supports JSON-RPC 2.0 over stdio.

use crate::context::ContextStore;
use crate::error::McpError;
use crate::tools::{create_standard_registry, ToolDefinition, ToolRegistry};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::RwLock;

/// MCP protocol version
pub const PROTOCOL_VERSION: &str = "2024-11-05";

/// JSON-RPC request
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Request {
    /// A single request
    Single(JsonRpcRequest),
    /// A batch of requests
    Batch(Vec<JsonRpcRequest>),
}

/// Single JSON-RPC request
#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcRequest {
    /// JSON-RPC version
    pub jsonrpc: String,
    /// Method to call
    pub method: String,
    /// Parameters
    #[serde(default)]
    pub params: Option<Value>,
    /// Request ID
    pub id: Option<Value>,
}

/// JSON-RPC response
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum Response {
    /// Single response
    Single(JsonRpcResponse),
    /// Batch of responses
    Batch(Vec<JsonRpcResponse>),
}

/// Single JSON-RPC response
#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcResponse {
    /// JSON-RPC version
    pub jsonrpc: String,
    /// Result (if success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Error (if failure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    /// Request ID
    pub id: Option<Value>,
}

/// JSON-RPC error
#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcError {
    /// Error code
    pub code: i32,
    /// Error message
    pub message: String,
    /// Additional error data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcError {
    /// Parse error (-32700)
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self {
            code: -32700,
            message: message.into(),
            data: None,
        }
    }

    /// Invalid request (-32600)
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self {
            code: -32600,
            message: message.into(),
            data: None,
        }
    }

    /// Method not found (-32601)
    pub fn method_not_found(method: impl Into<String>) -> Self {
        Self {
            code: -32601,
            message: format!("Method not found: {}", method.into()),
            data: None,
        }
    }

    /// Invalid params (-32602)
    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self {
            code: -32602,
            message: message.into(),
            data: None,
        }
    }

    /// Internal error (-32603)
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self {
            code: -32603,
            message: message.into(),
            data: None,
        }
    }
}

/// MCP server state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerState {
    /// Not yet initialized
    Uninitialized,
    /// Initialized and ready
    Initialized,
    /// Shutting down
    ShuttingDown,
}

/// MCP Server
pub struct McpServer {
    state: ServerState,
    tool_registry: ToolRegistry,
    context_store: Arc<RwLock<ContextStore>>,
}

impl McpServer {
    /// Create a new MCP server
    #[must_use]
    pub fn new(context_store: Arc<RwLock<ContextStore>>) -> Self {
        Self {
            state: ServerState::Uninitialized,
            tool_registry: create_standard_registry(),
            context_store,
        }
    }

    /// Handle a request and return a response
    pub async fn handle_request(&mut self, request: JsonRpcRequest) -> JsonRpcResponse {
        // Check JSON-RPC version
        if request.jsonrpc != "2.0" {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError::invalid_request("Invalid JSON-RPC version")),
                id: request.id,
            };
        }

        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(request.params).await,
            "notifications/initialized" => {
                // Notification, no response needed
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: Some(json!({})),
                    error: None,
                    id: request.id,
                };
            }
            "tools/list" => self.handle_tools_list().await,
            "tools/call" => self.handle_tools_call(request.params).await,
            "resources/list" => self.handle_resources_list().await,
            "resources/read" => self.handle_resources_read(request.params).await,
            _ => Err(McpError::UnknownMethod(request.method)),
        };

        match result {
            Ok(value) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(value),
                error: None,
                id: request.id,
            },
            Err(e) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError::internal_error(e.to_string())),
                id: request.id,
            },
        }
    }

    /// Handle initialize request
    async fn handle_initialize(&mut self, _params: Option<Value>) -> Result<Value, McpError> {
        if self.state == ServerState::Initialized {
            return Err(McpError::AlreadyInitialized);
        }

        self.state = ServerState::Initialized;

        Ok(json!({
            "protocolVersion": PROTOCOL_VERSION,
            "capabilities": {
                "tools": {},
                "resources": {}
            },
            "serverInfo": {
                "name": "aiharness",
                "version": env!("CARGO_PKG_VERSION")
            }
        }))
    }

    /// Handle tools/list request
    async fn handle_tools_list(&self) -> Result<Value, McpError> {
        if self.state != ServerState::Initialized {
            return Err(McpError::NotInitialized);
        }

        let tools: Vec<ToolDefinition> = self.tool_registry.list();
        Ok(json!({ "tools": tools }))
    }

    /// Handle tools/call request
    async fn handle_tools_call(&self, params: Option<Value>) -> Result<Value, McpError> {
        if self.state != ServerState::Initialized {
            return Err(McpError::NotInitialized);
        }

        let params = params.ok_or_else(|| McpError::MissingParameter("params".to_string()))?;
        
        let name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::MissingParameter("name".to_string()))?;

        let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

        let tool = self.tool_registry
            .get(name)
            .ok_or_else(|| McpError::ToolNotFound(name.to_string()))?;

        let result = tool.execute(arguments).await
            .map_err(|e| McpError::ToolExecutionFailed(e.to_string()))?;

        Ok(json!({
            "content": [
                {
                    "type": "text",
                    "text": result.content
                }
            ],
            "isError": !result.success
        }))
    }

    /// Handle resources/list request
    async fn handle_resources_list(&self) -> Result<Value, McpError> {
        if self.state != ServerState::Initialized {
            return Err(McpError::NotInitialized);
        }

        let files = self.context_store.read().await.list_files().await
            .map_err(|e| McpError::InternalError(e.to_string()))?;

        let resources: Vec<Value> = files.into_iter().map(|f| {
            json!({
                "uri": format!("file://{}", f.path),
                "name": std::path::Path::new(&f.path).file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "unknown".to_string()),
                "mimeType": "text/plain"
            })
        }).collect();

        Ok(json!({ "resources": resources }))
    }

    /// Handle resources/read request
    async fn handle_resources_read(&self, params: Option<Value>) -> Result<Value, McpError> {
        if self.state != ServerState::Initialized {
            return Err(McpError::NotInitialized);
        }

        let params = params.ok_or_else(|| McpError::MissingParameter("params".to_string()))?;
        
        let uri = params
            .get("uri")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::MissingParameter("uri".to_string()))?;

        // Parse file:// URI
        let path = uri.strip_prefix("file://")
            .ok_or_else(|| McpError::InvalidParameter { 
                name: "uri".to_string(), 
                value: uri.to_string() 
            })?;

        // Read file using the read_file tool
        let read_file_tool = self.tool_registry
            .get("read_file")
            .ok_or_else(|| McpError::InternalError("read_file tool not found".to_string()))?;

        let result = read_file_tool
            .execute(json!({"path": path}))
            .await
            .map_err(|e| McpError::ResourceNotFound(e.to_string()))?;

        Ok(json!({
            "contents": [
                {
                    "uri": uri,
                    "mimeType": "text/plain",
                    "text": result.content
                }
            ]
        }))
    }

    /// Run the MCP server over stdio
    pub async fn run_stdio(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();
        let reader = BufReader::new(stdin);
        let mut lines = reader.lines();
        let mut stdout = stdout;

        while let Some(line) = lines.next_line().await? {
            if line.is_empty() {
                continue;
            }

            // Parse request
            let request: Result<JsonRpcRequest, _> = serde_json::from_str(&line);
            
            let response = match request {
                Ok(req) => self.handle_request(req).await,
                Err(e) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError::parse_error(e.to_string())),
                    id: None,
                },
            };

            // Send response
            let response_json = serde_json::to_string(&response)?;
            stdout.write_all(response_json.as_bytes()).await?;
            stdout.write_all(b"\n").await?;
            stdout.flush().await?;
        }

        Ok(())
    }

    /// Get current server state
    #[must_use]
    pub fn state(&self) -> ServerState {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_server() -> (McpServer, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let context_store = Arc::new(RwLock::new(
            ContextStore::new(db_path.to_str().unwrap()).await.unwrap()
        ));
        let server = McpServer::new(context_store);
        (server, temp_dir)
    }

    #[tokio::test]
    async fn mcp_server_new_is_uninitialized() {
        let (server, _temp) = create_test_server().await;
        assert_eq!(server.state(), ServerState::Uninitialized);
    }

    #[tokio::test]
    async fn handle_request_invalid_json_rpc_version() {
        let (mut server, _temp) = create_test_server().await;
        
        let request = JsonRpcRequest {
            jsonrpc: "1.0".to_string(),
            method: "initialize".to_string(),
            params: None,
            id: Some(json!(1)),
        };

        let response = server.handle_request(request).await;
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, -32600);
    }

    #[tokio::test]
    async fn handle_initialize_sets_initialized_state() {
        let (mut server, _temp) = create_test_server().await;
        
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "initialize".to_string(),
            params: None,
            id: Some(json!(1)),
        };

        let response = server.handle_request(request).await;
        assert!(response.result.is_some());
        assert_eq!(server.state(), ServerState::Initialized);
    }

    #[tokio::test]
    async fn handle_initialize_fails_if_already_initialized() {
        let (mut server, _temp) = create_test_server().await;
        
        // First initialize
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "initialize".to_string(),
            params: None,
            id: Some(json!(1)),
        };
        server.handle_request(request).await;

        // Second initialize should fail
        let request2 = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "initialize".to_string(),
            params: None,
            id: Some(json!(2)),
        };
        let response = server.handle_request(request2).await;
        assert!(response.error.is_some());
    }

    #[tokio::test]
    async fn handle_tools_list_requires_initialization() {
        let (mut server, _temp) = create_test_server().await;
        
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/list".to_string(),
            params: None,
            id: Some(json!(1)),
        };

        let response = server.handle_request(request).await;
        assert!(response.error.is_some());
    }

    #[tokio::test]
    async fn handle_tools_list_returns_tools_after_init() {
        let (mut server, _temp) = create_test_server().await;
        
        // Initialize
        server.handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "initialize".to_string(),
            params: None,
            id: Some(json!(1)),
        }).await;

        // List tools
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/list".to_string(),
            params: None,
            id: Some(json!(2)),
        };

        let response = server.handle_request(request).await;
        assert!(response.result.is_some());
        
        let result = response.result.unwrap();
        assert!(result.get("tools").is_some());
    }

    #[tokio::test]
    async fn handle_tools_call_executes_tool() {
        let (mut server, temp) = create_test_server().await;
        
        // Create test file
        let file_path = temp.path().join("test.txt");
        tokio::fs::write(&file_path, "Hello, World!").await.unwrap();

        // Initialize
        server.handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "initialize".to_string(),
            params: None,
            id: Some(json!(1)),
        }).await;

        // Call read_file tool
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "read_file",
                "arguments": {"path": file_path.to_str().unwrap()}
            })),
            id: Some(json!(2)),
        };

        let response = server.handle_request(request).await;
        assert!(response.result.is_some());
        
        let result = response.result.unwrap();
        let content = result.get("content").unwrap();
        assert!(content.to_string().contains("Hello, World!"));
    }

    #[tokio::test]
    async fn handle_tools_call_fails_for_unknown_tool() {
        let (mut server, _temp) = create_test_server().await;
        
        // Initialize
        server.handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "initialize".to_string(),
            params: None,
            id: Some(json!(1)),
        }).await;

        // Call unknown tool
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "unknown_tool",
                "arguments": {}
            })),
            id: Some(json!(2)),
        };

        let response = server.handle_request(request).await;
        assert!(response.error.is_some());
    }

    #[tokio::test]
    async fn handle_tools_call_requires_params() {
        let (mut server, _temp) = create_test_server().await;
        
        // Initialize
        server.handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "initialize".to_string(),
            params: None,
            id: Some(json!(1)),
        }).await;

        // Call without params
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/call".to_string(),
            params: None,
            id: Some(json!(2)),
        };

        let response = server.handle_request(request).await;
        assert!(response.error.is_some());
    }

    #[tokio::test]
    async fn handle_unknown_method_returns_error() {
        let (mut server, _temp) = create_test_server().await;
        
        // Initialize
        server.handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "initialize".to_string(),
            params: None,
            id: Some(json!(1)),
        }).await;

        // Unknown method
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "unknown/method".to_string(),
            params: None,
            id: Some(json!(2)),
        };

        let response = server.handle_request(request).await;
        assert!(response.error.is_some());
    }

    #[tokio::test]
    async fn handle_resources_list_returns_context_files() {
        let (mut server, temp) = create_test_server().await;
        
        // Add file to context
        let file_path = temp.path().join("context.txt");
        tokio::fs::write(&file_path, "content").await.unwrap();
        {
            let store = server.context_store.read().await;
            store.add_file(file_path.to_str().unwrap()).await.unwrap();
        }

        // Initialize
        server.handle_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "initialize".to_string(),
            params: None,
            id: Some(json!(1)),
        }).await;

        // List resources
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "resources/list".to_string(),
            params: None,
            id: Some(json!(2)),
        };

        let response = server.handle_request(request).await;
        assert!(response.result.is_some());
        
        let result = response.result.unwrap();
        let resources = result.get("resources").unwrap().as_array().unwrap();
        assert_eq!(resources.len(), 1);
    }

    #[tokio::test]
    async fn json_rpc_error_codes() {
        let parse_error = JsonRpcError::parse_error("test");
        assert_eq!(parse_error.code, -32700);

        let invalid_req = JsonRpcError::invalid_request("test");
        assert_eq!(invalid_req.code, -32600);

        let method_not_found = JsonRpcError::method_not_found("foo");
        assert_eq!(method_not_found.code, -32601);

        let invalid_params = JsonRpcError::invalid_params("test");
        assert_eq!(invalid_params.code, -32602);

        let internal_error = JsonRpcError::internal_error("test");
        assert_eq!(internal_error.code, -32603);
    }

    #[tokio::test]
    async fn request_deserialization() {
        let json = r#"{"jsonrpc":"2.0","method":"initialize","id":1}"#;
        let request: JsonRpcRequest = serde_json::from_str(json).unwrap();
        
        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method, "initialize");
        assert_eq!(request.id, Some(json!(1)));
    }

    #[tokio::test]
    async fn response_serialization() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(json!({"key": "value"})),
            error: None,
            id: Some(json!(1)),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("key"));
        assert!(json.contains("value"));
    }

    #[tokio::test]
    async fn response_with_error_serialization() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError::internal_error("oops")),
            id: Some(json!(1)),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("error"));
        assert!(json.contains("oops"));
    }
}
