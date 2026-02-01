//! HTTP server for AI connections
//! 
//! Built into the main app - shares state with GUI

use axum::{
    routing::{get, post},
    extract::State,
    response::IntoResponse,
    Json, Router,
};
use serde_json::json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;

use crate::{
    app_state::AppState,
    tools::ToolDefinition,
    ToolCallEvent,
};

/// Shared state for HTTP handlers
type HttpState = Arc<RwLock<AppState>>;

/// Start HTTP server
pub async fn start_http_server(
    app_state: Arc<RwLock<AppState>>,
    port: u16,
) -> Result<tokio::task::JoinHandle<()>, String> {
    let app = create_router(app_state);
    
    let addr = format!("127.0.0.1:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| format!("Failed to bind: {}", e))?;
    
    tracing::info!("HTTP server starting on http://{}", addr);
    
    let handle = tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            tracing::error!("HTTP server error: {}", e);
        }
    });
    
    Ok(handle)
}

/// Create HTTP router
fn create_router(app_state: HttpState) -> Router {
    Router::new()
        .route("/", get(health_check))
        .route("/tools", get(list_tools))
        .route("/call", post(execute_tool))
        .route("/mcp", post(handle_mcp_request))
        .route("/events", get(get_events))
        .route("/events/stream", get(stream_events))
        .layer(CorsLayer::permissive())
        .with_state(app_state)
}

/// Health check
async fn health_check() -> &'static str {
    "AIHarness Server Running"
}

/// List available tools
async fn list_tools(State(state): State<HttpState>) -> Json<serde_json::Value> {
    let state = state.read().await;
    let mut tools = state.tool_registry.list();
    tools.extend(todo_tool_definitions());
    tools.extend(build_tool_definitions());
    Json(json!({ "tools": map_tools(&tools, "input_schema") }))
}

/// Execute a tool
async fn execute_tool(
    State(state): State<HttpState>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let (tool_name, arguments) = parse_tool_call_body(&body);
    let project_id = parse_project_id(&body);
    match execute_tool_call(state, &tool_name, arguments, project_id).await {
        Ok(result) => Json(json!({
            "success": true,
            "content": result.content,
            "duration_ms": result.duration_ms,
        })),
        Err(error) => Json(json!({
            "success": false,
            "error": error,
        })),
    }
}

/// Get event history
async fn get_events(State(state): State<HttpState>) -> Json<Vec<ToolCallEvent>> {
    let state = state.read().await;
    let history = state.get_history().await;
    Json(history)
}

/// MCP protocol version
const MCP_PROTOCOL_VERSION: &str = "2024-11-05";

/// JSON-RPC request
#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: Option<serde_json::Value>,
    pub id: Option<serde_json::Value>,
}

/// JSON-RPC response
#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: Option<serde_json::Value>,
}

/// JSON-RPC error
#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

fn json_rpc_error_response(
    code: i32,
    message: impl Into<String>,
    id: Option<serde_json::Value>,
) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: None,
        error: Some(JsonRpcError {
            code,
            message: message.into(),
        }),
        id,
    }
}

/// Handle MCP JSON-RPC requests over HTTP
async fn handle_mcp_request(
    State(state): State<HttpState>,
    Json(request): Json<serde_json::Value>,
) -> Json<JsonRpcResponse> {
    let request = match parse_json_rpc_request(request) {
        Ok(req) => req,
        Err(response) => return Json(response),
    };

    let response = match request.method.as_str() {
        "initialize" => handle_mcp_initialize(request.id),
        "tools/list" => handle_mcp_tools_list(&state, request.id).await,
        "tools/call" => handle_mcp_tools_call(&state, request.id, request.params).await,
        "resources/list" => handle_mcp_resources_list(&state, request.id, request.params).await,
        "resources/read" => handle_mcp_resources_read(&state, request.id, request.params).await,
        _ => json_rpc_error_response(-32601, format!("Method not found: {}", request.method), request.id),
    };

    Json(response)
}

/// Stream events (SSE - Server Sent Events)
async fn stream_events(State(state): State<HttpState>) -> axum::response::Response {
    use axum::response::sse::{Event, Sse};
    use std::convert::Infallible;
    
    let state = state.read().await;
    let mut rx = state.subscribe();
    
    let stream = async_stream::stream! {
        while let Ok(event) = rx.recv().await {
            yield Ok::<_, Infallible>(Event::default().json_data(&event).unwrap());
        }
    };
    
    Sse::new(stream).into_response()
}

struct ToolCallResult {
    content: String,
    duration_ms: u64,
}

async fn execute_tool_call(
    state: HttpState,
    tool_name: &str,
    arguments: serde_json::Value,
    project_id: String,
) -> Result<ToolCallResult, String> {
    use std::time::Instant;
    use uuid::Uuid;

    let start = Instant::now();
    let call_id = Uuid::new_v4().to_string();
    let timestamp = chrono::Utc::now().to_rfc3339();

    let result = if is_todo_tool(tool_name) {
        execute_todo_tool_call(state.clone(), tool_name, arguments.clone(), &project_id).await
    } else if is_build_tool(tool_name) {
        execute_build_tool_call(state.clone(), tool_name, arguments.clone(), &project_id).await
    } else {
        let state_read = state.read().await;
        let tool = match state_read.tool_registry.get(tool_name) {
            Some(t) => t,
            None => return Err(format!("Tool not found: {}", tool_name)),
        };
        tool.execute(arguments.clone())
            .await
            .map(|r| r.content)
            .map_err(|e| e.to_string())
    };
    let duration_ms = start.elapsed().as_millis() as u64;

    let event = match &result {
        Ok(output) => ToolCallEvent {
            id: call_id.clone(),
            timestamp: timestamp.clone(),
            tool_name: tool_name.to_string(),
            project_id: project_id.clone(),
            arguments: arguments.clone(),
            success: true,
            content: output.clone(),
            duration_ms,
        },
        Err(e) => ToolCallEvent {
            id: call_id.clone(),
            timestamp: timestamp.clone(),
            tool_name: tool_name.to_string(),
            project_id: project_id.clone(),
            arguments: arguments.clone(),
            success: false,
            content: e.to_string(),
            duration_ms,
        },
    };

    {
        let state_read = state.read().await;
        state_read.record_event(event).await;
    }

    match result {
        Ok(output) => Ok(ToolCallResult {
            content: output,
            duration_ms,
        }),
        Err(e) => Err(e.to_string()),
    }
}

/// Extract tool call inputs from HTTP request body.
fn parse_tool_call_body(body: &serde_json::Value) -> (String, serde_json::Value) {
    let tool_name = body.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let arguments = body.get("arguments").cloned().unwrap_or(json!({}));
    (tool_name, arguments)
}

fn parse_project_id(body: &serde_json::Value) -> String {
    body.get("project_id")
        .or_else(|| body.get("projectId"))
        .and_then(|v| v.as_str())
        .unwrap_or("default")
        .to_string()
}

fn require_arg_string(args: &serde_json::Value, key: &str) -> Result<String, String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| format!("Missing '{}'", key))
}

fn require_arg_i64(args: &serde_json::Value, key: &str) -> Result<i64, String> {
    args.get(key)
        .and_then(|v| v.as_i64())
        .ok_or_else(|| format!("Missing '{}'", key))
}

/// Parse a JSON-RPC request and validate protocol version.
fn parse_json_rpc_request(request: serde_json::Value) -> Result<JsonRpcRequest, JsonRpcResponse> {
    let parsed: Result<JsonRpcRequest, _> = serde_json::from_value(request);
    let request = match parsed {
        Ok(req) => req,
        Err(e) => {
            return Err(json_rpc_error_response(-32700, format!("Invalid JSON-RPC request: {}", e), None));
        }
    };

    if request.jsonrpc != "2.0" {
        return Err(json_rpc_error_response(-32600, "Invalid JSON-RPC version", request.id));
    }

    Ok(request)
}

/// Require params for JSON-RPC methods.
fn require_params(
    params: Option<serde_json::Value>,
    id: Option<serde_json::Value>,
) -> Result<serde_json::Value, JsonRpcResponse> {
    params.ok_or_else(|| json_rpc_error_response(-32602, "Missing params", id))
}

/// Require a string parameter from JSON-RPC params.
fn require_str_param<'a>(
    params: &'a serde_json::Value,
    key: &str,
    id: Option<serde_json::Value>,
) -> Result<&'a str, JsonRpcResponse> {
    params
        .get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| json_rpc_error_response(-32602, format!("Missing {}", key), id))
}

/// Map tool definitions into JSON payloads.
fn map_tools(tools: &[crate::tools::ToolDefinition], schema_key: &str) -> Vec<serde_json::Value> {
    tools
        .iter()
        .map(|t| {
            json!({
                "name": t.name,
                "description": t.description,
                schema_key: t.input_schema,
            })
        })
        .collect::<Vec<_>>()
}

/// Build a standard MCP tool call response.
fn mcp_content_response(id: Option<serde_json::Value>, content: String, is_error: bool) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: Some(json!({
            "content": [
                {
                    "type": "text",
                    "text": content
                }
            ],
            "isError": is_error
        })),
        error: None,
        id,
    }
}

/// Handle MCP initialize.
fn handle_mcp_initialize(id: Option<serde_json::Value>) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: Some(json!({
            "protocolVersion": MCP_PROTOCOL_VERSION,
            "capabilities": {
                "tools": {},
                "resources": {}
            },
            "serverInfo": {
                "name": "aiharness",
                "version": env!("CARGO_PKG_VERSION")
            }
        })),
        error: None,
        id,
    }
}

/// Handle MCP tools/list.
async fn handle_mcp_tools_list(
    state: &HttpState,
    id: Option<serde_json::Value>,
) -> JsonRpcResponse {
    let state = state.read().await;
    let mut tools = state.tool_registry.list();
    tools.extend(todo_tool_definitions());
    tools.extend(build_tool_definitions());
    let tools = map_tools(&tools, "inputSchema");
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: Some(json!({ "tools": tools })),
        error: None,
        id,
    }
}

/// Handle MCP tools/call.
async fn handle_mcp_tools_call(
    state: &HttpState,
    id: Option<serde_json::Value>,
    params: Option<serde_json::Value>,
) -> JsonRpcResponse {
    let params = match require_params(params, id.clone()) {
        Ok(p) => p,
        Err(e) => return e,
    };

    let tool_name = match require_str_param(&params, "name", id.clone()) {
        Ok(name) => name,
        Err(e) => return e,
    };

    let arguments = params.get("arguments").cloned().unwrap_or(json!({}));
    let project_id = params
        .get("projectId")
        .or_else(|| params.get("project_id"))
        .and_then(|v| v.as_str())
        .unwrap_or("default")
        .to_string();

    match execute_tool_call(state.clone(), tool_name, arguments, project_id).await {
        Ok(result) => mcp_content_response(id, result.content, false),
        Err(error) => mcp_content_response(id, error, true),
    }
}

/// Handle MCP resources/list.
async fn handle_mcp_resources_list(
    state: &HttpState,
    id: Option<serde_json::Value>,
    params: Option<serde_json::Value>,
) -> JsonRpcResponse {
    let project_id = params
        .as_ref()
        .and_then(|p| p.get("projectId").or_else(|| p.get("project_id")))
        .and_then(|v| v.as_str())
        .unwrap_or("default")
        .to_string();
    let store = {
        let state_read = state.read().await;
        match state_read.get_project_store(&project_id).await {
            Ok(store) => store,
            Err(e) => return json_rpc_error_response(-32603, e.to_string(), id),
        }
    };
    let files = match store.context_store.read().await.list_files().await {
        Ok(f) => f,
        Err(e) => return json_rpc_error_response(-32603, e.to_string(), id),
    };

    let resources = files
        .into_iter()
        .map(|f| {
            json!({
                "uri": format!("file://{}", f.path),
                "name": std::path::Path::new(&f.path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "unknown".to_string()),
                "mimeType": "text/plain"
            })
        })
        .collect::<Vec<_>>();

    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: Some(json!({ "resources": resources })),
        error: None,
        id,
    }
}

/// Handle MCP resources/read.
async fn handle_mcp_resources_read(
    state: &HttpState,
    id: Option<serde_json::Value>,
    params: Option<serde_json::Value>,
) -> JsonRpcResponse {
    let params = match require_params(params, id.clone()) {
        Ok(p) => p,
        Err(e) => return e,
    };

    let uri = match require_str_param(&params, "uri", id.clone()) {
        Ok(uri) => uri,
        Err(e) => return e,
    };

    let path = match uri.strip_prefix("file://") {
        Some(p) => p,
        None => return json_rpc_error_response(-32602, "Invalid uri", id),
    };

    let result = {
        let state_read = state.read().await;
        let read_file = match state_read
            .tool_registry
            .get("read_file")
            .ok_or_else(|| "read_file tool not found")
        {
            Ok(tool) => tool,
            Err(e) => return json_rpc_error_response(-32603, e, id),
        };

        match read_file.execute(json!({ "path": path })).await {
            Ok(r) => r,
            Err(e) => return json_rpc_error_response(-32603, e.to_string(), id),
        }
    };

    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: Some(json!({
            "contents": [
                {
                    "uri": uri,
                    "mimeType": "text/plain",
                    "text": result.content
                }
            ]
        })),
        error: None,
        id,
    }
}

fn todo_tool_definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "todo_add".to_string(),
            description: "Add a todo item to the ordered list".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "title": {"type": "string"},
                    "description": {"type": "string"},
                    "position": {"type": "integer"},
                    "project_id": {"type": "string"}
                },
                "required": ["title"]
            }),
        },
        ToolDefinition {
            name: "todo_remove".to_string(),
            description: "Remove a todo item".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {"id": {"type": "string"}},
                "required": ["id"]
            }),
        },
        ToolDefinition {
            name: "todo_check".to_string(),
            description: "Mark a todo item completed or not".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": {"type": "string"},
                    "completed": {"type": "boolean"}
                },
                "required": ["id"]
            }),
        },
        ToolDefinition {
            name: "todo_list".to_string(),
            description: "List all todos in order".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {"project_id": {"type": "string"}}
            }),
        },
        ToolDefinition {
            name: "todo_get_next".to_string(),
            description: "Get the next incomplete todo".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {"project_id": {"type": "string"}}
            }),
        },
        ToolDefinition {
            name: "todo_insert".to_string(),
            description: "Insert a todo at a specific position".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "title": {"type": "string"},
                    "description": {"type": "string"},
                    "position": {"type": "integer"}
                },
                "required": ["title", "position"]
            }),
        },
        ToolDefinition {
            name: "todo_move".to_string(),
            description: "Move a todo to a new position".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {"id": {"type": "string"}, "position": {"type": "integer"}},
                "required": ["id", "position"]
            }),
        },
    ]
}

fn is_todo_tool(tool_name: &str) -> bool {
    matches!(
        tool_name,
        "todo_add" | "todo_remove" | "todo_check" | "todo_list" | "todo_get_next" | "todo_insert" | "todo_move"
    )
}

fn build_tool_definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "build_add_command".to_string(),
            description: "Add a build command to the project.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string" },
                    "command": { "type": "string" },
                    "working_dir": { "type": "string" }
                },
                "required": ["name", "command"]
            }),
        },
        ToolDefinition {
            name: "build_remove_command".to_string(),
            description: "Remove a build command by id.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": { "type": "string" }
                },
                "required": ["id"]
            }),
        },
        ToolDefinition {
            name: "build_list_commands".to_string(),
            description: "List build commands for the project.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "build_run_command".to_string(),
            description: "Run a build command by id.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": { "type": "string" }
                },
                "required": ["id"]
            }),
        },
        ToolDefinition {
            name: "build_set_default".to_string(),
            description: "Set the default build command by id.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": { "type": "string" }
                },
                "required": ["id"]
            }),
        },
        ToolDefinition {
            name: "build_get_default".to_string(),
            description: "Get the default build command.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        },
    ]
}

fn is_build_tool(tool_name: &str) -> bool {
    matches!(
        tool_name,
        "build_add_command"
            | "build_remove_command"
            | "build_list_commands"
            | "build_run_command"
            | "build_set_default"
            | "build_get_default"
    )
}

async fn execute_build_tool_call(
    state: HttpState,
    tool_name: &str,
    arguments: serde_json::Value,
    project_id: &str,
) -> Result<String, String> {
    match tool_name {
        "build_add_command" => {
            let name = arguments
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing name".to_string())?;
            let command = arguments
                .get("command")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing command".to_string())?;
            let working_dir = arguments
                .get("working_dir")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let store = {
                let state_read = state.read().await;
                state_read
                    .get_project_store(project_id)
                    .await
                    .map_err(|e| e.to_string())?
            };
            let store = store.build_command_store.read().await;
            let command = store
                .add(&name, &command, working_dir)
                .await
                .map_err(|e| e.to_string())?;
            Ok(serde_json::to_string(&command).unwrap_or_default())
        }
        "build_remove_command" => {
            let id = arguments
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing id".to_string())?;
            let store = {
                let state_read = state.read().await;
                state_read
                    .get_project_store(project_id)
                    .await
                    .map_err(|e| e.to_string())?
            };
            let store = store.build_command_store.read().await;
            store.remove(&id).await.map_err(|e| e.to_string())?;
            Ok("ok".to_string())
        }
        "build_list_commands" => {
            let store = {
                let state_read = state.read().await;
                state_read
                    .get_project_store(project_id)
                    .await
                    .map_err(|e| e.to_string())?
            };
            let store = store.build_command_store.read().await;
            let list = store.list().await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_string(&list).unwrap_or_default())
        }
        "build_run_command" => {
            let id = arguments
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing id".to_string())?;
            let (command, root_path) = {
                let state_read = state.read().await;
                let project = state_read
                    .project_registry
                    .get_project(project_id)
                    .await
                    .map_err(|e| e.to_string())?
                    .ok_or_else(|| "Project not found".to_string())?;
                let store = state_read
                    .get_project_store(project_id)
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
            crate::run_shell_command(&command, &root_path).await
        }
        "build_set_default" => {
            let id = arguments
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing id".to_string())?;
            let store = {
                let state_read = state.read().await;
                state_read
                    .get_project_store(project_id)
                    .await
                    .map_err(|e| e.to_string())?
            };
            let store = store.build_command_store.read().await;
            store.set_default(&id).await.map_err(|e| e.to_string())?;
            Ok("ok".to_string())
        }
        "build_get_default" => {
            let store = {
                let state_read = state.read().await;
                state_read
                    .get_project_store(project_id)
                    .await
                    .map_err(|e| e.to_string())?
            };
            let store = store.build_command_store.read().await;
            let command = store.get_default().await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_string(&command).unwrap_or_default())
        }
        _ => Err(format!("Unknown build tool: {}", tool_name)),
    }
}

async fn execute_todo_tool_call(
    state: HttpState,
    tool_name: &str,
    arguments: serde_json::Value,
    project_id: &str,
) -> Result<String, String> {
    let store = {
        let state_read = state.read().await;
        state_read
            .get_project_store(project_id)
            .await
            .map_err(|e| e.to_string())?
    };

    match tool_name {
        "todo_add" => {
            let title = require_arg_string(&arguments, "title")?;
            let description = arguments.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
            let position = arguments.get("position").and_then(|v| v.as_i64());
            let todo = store
                .todo_store
                .read()
                .await
                .add(&title, description, position)
                .await
                .map_err(|e| e.to_string())?;
            Ok(serde_json::to_string_pretty(&todo).unwrap_or_else(|_| "{}".to_string()))
        }
        "todo_insert" => {
            let title = require_arg_string(&arguments, "title")?;
            let description = arguments.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
            let position = require_arg_i64(&arguments, "position")?;
            let todo = store
                .todo_store
                .read()
                .await
                .add(&title, description, Some(position))
                .await
                .map_err(|e| e.to_string())?;
            Ok(serde_json::to_string_pretty(&todo).unwrap_or_else(|_| "{}".to_string()))
        }
        "todo_remove" => {
            let id = require_arg_string(&arguments, "id")?;
            store
                .todo_store
                .read()
                .await
                .remove(&id)
                .await
                .map_err(|e| e.to_string())?;
            Ok("removed".to_string())
        }
        "todo_check" => {
            let id = require_arg_string(&arguments, "id")?;
            let completed = arguments.get("completed").and_then(|v| v.as_bool()).unwrap_or(true);
            store
                .todo_store
                .read()
                .await
                .set_completed(&id, completed)
                .await
                .map_err(|e| e.to_string())?;
            Ok("updated".to_string())
        }
        "todo_list" => {
            let todos = store
                .todo_store
                .read()
                .await
                .list()
                .await
                .map_err(|e| e.to_string())?;
            Ok(serde_json::to_string_pretty(&todos).unwrap_or_else(|_| "[]".to_string()))
        }
        "todo_get_next" => {
            let todo = store
                .todo_store
                .read()
                .await
                .get_next()
                .await
                .map_err(|e| e.to_string())?;
            Ok(serde_json::to_string_pretty(&todo).unwrap_or_else(|_| "null".to_string()))
        }
        "todo_move" => {
            let id = require_arg_string(&arguments, "id")?;
            let position = require_arg_i64(&arguments, "position")?;
            store
                .todo_store
                .read()
                .await
                .move_to(&id, position)
                .await
                .map_err(|e| e.to_string())?;
            Ok("moved".to_string())
        }
        _ => Err("Unknown todo tool".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_state::AppState;
    use tempfile::TempDir;

    #[test]
    fn parse_tool_call_body_defaults() {
        let body = json!({});
        let (name, args) = parse_tool_call_body(&body);
        assert_eq!(name, "");
        assert_eq!(args, json!({}));
    }

    #[test]
    fn parse_tool_call_body_extracts_fields() {
        let body = json!({
            "name": "read_file",
            "arguments": { "path": "/tmp/a.txt" }
        });
        let (name, args) = parse_tool_call_body(&body);
        assert_eq!(name, "read_file");
        assert_eq!(args, json!({ "path": "/tmp/a.txt" }));
    }

    #[test]
    fn parse_project_id_defaults_to_default() {
        let body = json!({});
        assert_eq!(parse_project_id(&body), "default");
    }

    #[test]
    fn parse_project_id_accepts_project_id() {
        let body = json!({ "project_id": "alpha" });
        assert_eq!(parse_project_id(&body), "alpha");
    }

    #[test]
    fn parse_project_id_accepts_project_id_camel() {
        let body = json!({ "projectId": "beta" });
        assert_eq!(parse_project_id(&body), "beta");
    }

    #[test]
    fn parse_json_rpc_request_rejects_wrong_version() {
        let request = json!({
            "jsonrpc": "1.0",
            "method": "tools/list",
            "id": 1
        });
        let err = parse_json_rpc_request(request).unwrap_err();
        assert_eq!(err.error.unwrap().code, -32600);
    }

    #[test]
    fn require_params_errors_on_none() {
        let err = require_params(None, Some(json!(1))).unwrap_err();
        assert_eq!(err.error.unwrap().code, -32602);
    }

    #[test]
    fn require_str_param_errors_on_missing() {
        let params = json!({});
        let err = require_str_param(&params, "name", Some(json!(1))).unwrap_err();
        assert_eq!(err.error.unwrap().code, -32602);
    }

    #[test]
    fn map_tools_uses_schema_key() {
        let tools = vec![crate::tools::ToolDefinition {
            name: "t".to_string(),
            description: "d".to_string(),
            input_schema: json!({"type": "object"}),
        }];
        let mapped = map_tools(&tools, "inputSchema");
        assert_eq!(mapped.len(), 1);
        assert!(mapped[0].get("inputSchema").is_some());
    }

    #[test]
    fn mcp_content_response_sets_error_flag() {
        let response = mcp_content_response(Some(json!(1)), "oops".to_string(), true);
        let result = response.result.unwrap();
        assert_eq!(result.get("isError").unwrap(), &json!(true));
    }

    #[test]
    fn handle_mcp_initialize_returns_protocol_version() {
        let response = handle_mcp_initialize(Some(json!(1)));
        let result = response.result.unwrap();
        assert_eq!(result.get("protocolVersion").unwrap(), MCP_PROTOCOL_VERSION);
    }

    #[tokio::test]
    async fn handle_mcp_tools_list_returns_tools() {
        let temp_dir = TempDir::new().unwrap();
        let registry_path = temp_dir.path().join("registry.db");
        let state = AppState::new(registry_path.to_str().unwrap(), temp_dir.path())
            .await
            .unwrap();
        let state = Arc::new(RwLock::new(state));

        let response = handle_mcp_tools_list(&state, Some(json!(1))).await;
        let result = response.result.unwrap();
        let tools = result.get("tools").unwrap().as_array().unwrap();
        assert!(!tools.is_empty());
    }
}
