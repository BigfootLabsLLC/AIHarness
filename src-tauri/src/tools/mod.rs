//! Tool system for AIHarness
//!
//! Provides a pluggable tool architecture where each tool is a pure function
//! that takes arguments and returns a result.

use crate::error::ToolError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

pub mod file;

/// The result of executing a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Whether the tool execution succeeded
    pub success: bool,
    /// The output content (text)
    pub content: String,
    /// Optional structured data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl ToolResult {
    /// Create a successful tool result
    #[must_use]
    pub fn success(content: impl Into<String>) -> Self {
        Self {
            success: true,
            content: content.into(),
            data: None,
        }
    }

    /// Create a successful tool result with data
    #[must_use]
    pub fn success_with_data(content: impl Into<String>, data: Value) -> Self {
        Self {
            success: true,
            content: content.into(),
            data: Some(data),
        }
    }

    /// Create a failed tool result
    #[must_use]
    pub fn error(content: impl Into<String>) -> Self {
        Self {
            success: false,
            content: content.into(),
            data: None,
        }
    }
}

/// Definition of a tool for the MCP protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// The tool name
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// JSON schema for input validation
    pub input_schema: Value,
}

/// Trait that all tools must implement
#[async_trait]
pub trait Tool: Send + Sync {
    /// Get the tool name
    fn name(&self) -> &str;
    
    /// Get the tool description
    fn description(&self) -> &str;
    
    /// Get the input schema
    fn input_schema(&self) -> Value;
    
    /// Execute the tool with the given arguments
    /// 
    /// # Errors
    /// 
    /// Returns a `ToolError` if execution fails
    async fn execute(&self, args: Value) -> Result<ToolResult, ToolError>;
    
    /// Get the full tool definition
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name().to_string(),
            description: self.description().to_string(),
            input_schema: self.input_schema(),
        }
    }
}

/// Registry of available tools
#[derive(Default)]
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    /// Create a new empty tool registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Register a tool
    pub fn register(&mut self, tool: Box<dyn Tool>) {
        let name = tool.name().to_string();
        self.tools.insert(name, tool);
    }

    /// Get a tool by name
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|t| t.as_ref())
    }

    /// Check if a tool exists
    #[must_use]
    pub fn has(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// List all available tools
    pub fn list(&self) -> Vec<ToolDefinition> {
        self.tools
            .values()
            .map(|t| t.definition())
            .collect()
    }

    /// Get the number of registered tools
    #[must_use]
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Check if the registry is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }
}

/// Create a standard tool registry with all built-in tools
#[must_use]
pub fn create_standard_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    
    registry.register(Box::new(file::ReadFileTool));
    registry.register(Box::new(file::WriteFileTool));
    registry.register(Box::new(file::ListDirectoryTool));
    registry.register(Box::new(file::SearchFilesTool));
    
    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    // ToolResult tests
    #[test]
    fn tool_result_success_creates_success_result() {
        let result = ToolResult::success("Hello");
        assert!(result.success);
        assert_eq!(result.content, "Hello");
        assert!(result.data.is_none());
    }

    #[test]
    fn tool_result_success_with_data_creates_result_with_data() {
        let data = serde_json::json!({"key": "value"});
        let result = ToolResult::success_with_data("Hello", data.clone());
        assert!(result.success);
        assert_eq!(result.data, Some(data));
    }

    #[test]
    fn tool_result_error_creates_failed_result() {
        let result = ToolResult::error("Something went wrong");
        assert!(!result.success);
        assert_eq!(result.content, "Something went wrong");
    }

    #[test]
    fn tool_result_serialization_roundtrip() {
        let result = ToolResult::success_with_data("test", serde_json::json!({"a": 1}));
        let json = serde_json::to_string(&result).unwrap();
        let decoded: ToolResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result.success, decoded.success);
        assert_eq!(result.content, decoded.content);
    }

    // ToolDefinition tests
    #[test]
    fn tool_definition_creates_correctly() {
        let schema = serde_json::json!({"type": "object"});
        let def = ToolDefinition {
            name: "test".to_string(),
            description: "A test tool".to_string(),
            input_schema: schema.clone(),
        };
        assert_eq!(def.name, "test");
        assert_eq!(def.description, "A test tool");
        assert_eq!(def.input_schema, schema);
    }

    // ToolRegistry tests
    #[test]
    fn registry_new_is_empty() {
        let registry = ToolRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn registry_register_adds_tool() {
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(file::ReadFileTool));
        assert_eq!(registry.len(), 1);
        assert!(registry.has("read_file"));
    }

    #[test]
    fn registry_get_returns_tool() {
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(file::ReadFileTool));
        
        let tool = registry.get("read_file");
        assert!(tool.is_some());
        assert_eq!(tool.unwrap().name(), "read_file");
    }

    #[test]
    fn registry_get_missing_returns_none() {
        let registry = ToolRegistry::new();
        assert!(registry.get("missing").is_none());
    }

    #[test]
    fn registry_list_returns_all_tools() {
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(file::ReadFileTool));
        registry.register(Box::new(file::WriteFileTool));
        
        let tools = registry.list();
        assert_eq!(tools.len(), 2);
    }

    #[test]
    fn registry_has_returns_false_for_missing() {
        let registry = ToolRegistry::new();
        assert!(!registry.has("missing"));
    }

    #[test]
    fn create_standard_registry_has_expected_tools() {
        let registry = create_standard_registry();
        assert!(registry.has("read_file"));
        assert!(registry.has("write_file"));
        assert!(registry.has("list_directory"));
        assert!(registry.has("search_files"));
    }

    #[test]
    fn tool_definition_from_trait() {
        let tool = file::ReadFileTool;
        let def = tool.definition();
        assert_eq!(def.name, "read_file");
        assert!(!def.description.is_empty());
        assert!(!def.input_schema.is_null());
    }
}
