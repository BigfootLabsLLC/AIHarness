//! MCP Configuration Management
//!
//! Handles generating and writing MCP server configurations for various AI tools.

use crate::error::ContextError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Supported AI tools for MCP configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AiTool {
    Claude,
    Kimi,
    Gemini,
    Codex,
}

impl AiTool {
    /// Get all supported AI tools
    pub fn all() -> Vec<AiTool> {
        vec![AiTool::Claude, AiTool::Kimi, AiTool::Gemini, AiTool::Codex]
    }

    /// Get display name for the AI tool
    pub fn display_name(&self) -> &'static str {
        match self {
            AiTool::Claude => "Claude Code",
            AiTool::Kimi => "Kimi CLI",
            AiTool::Gemini => "Gemini CLI",
            AiTool::Codex => "Codex CLI",
        }
    }

    /// Get the configuration file path for this AI tool
    pub fn config_path(&self) -> Result<PathBuf, ContextError> {
        let home = dirs::home_dir().ok_or_else(|| {
            ContextError::Config("Could not determine home directory".to_string())
        })?;

        match self {
            AiTool::Claude => {
                // Claude Code: ~/.claude/settings.json or similar
                // TODO: Confirm exact path from docs
                Ok(home.join(".claude").join("settings.json"))
            }
            AiTool::Kimi => {
                // Kimi CLI: ~/.kimi/mcp.json (based on MCP docs)
                Ok(home.join(".kimi").join("mcp.json"))
            }
            AiTool::Gemini => {
                // Gemini CLI: TBD
                // TODO: Add path from docs
                Ok(home.join(".gemini").join("config.json"))
            }
            AiTool::Codex => {
                // Codex CLI: TBD
                // TODO: Add path from docs
                Ok(home.join(".codex").join("config.json"))
            }
        }
    }
}

/// MCP server configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    pub transport: String,
    pub url: Option<String>,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub headers: Option<HashMap<String, String>>,
    pub env: Option<HashMap<String, String>>,
}

impl McpServerConfig {
    /// Create a new HTTP-based MCP server config
    pub fn http(name: &str, url: &str) -> Self {
        Self {
            name: name.to_string(),
            transport: "http".to_string(),
            url: Some(url.to_string()),
            command: None,
            args: None,
            headers: None,
            env: None,
        }
    }

    /// Create a new stdio-based MCP server config
    pub fn stdio(name: &str, command: &str, args: Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            transport: "stdio".to_string(),
            url: None,
            command: Some(command.to_string()),
            args: Some(args),
            headers: None,
            env: None,
        }
    }

    /// Add a header (for HTTP transport)
    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        let headers = self.headers.get_or_insert_with(HashMap::new);
        headers.insert(key.to_string(), value.to_string());
        self
    }

    /// Add an environment variable (for stdio transport)
    pub fn with_env(mut self, key: &str, value: &str) -> Self {
        let env = self.env.get_or_insert_with(HashMap::new);
        env.insert(key.to_string(), value.to_string());
        self
    }
}

/// Generate MCP configuration for a specific AI tool and project
pub async fn generate_mcp_config(
    tool: AiTool,
    _project_name: &str,
    project_id: &str,
    server_port: u16,
) -> Result<String, ContextError> {
    let server_name = format!("aiharness-{}", project_id);
    let server_url = format!("http://127.0.0.1:{}/mcp/{}", server_port, project_id);

    match tool {
        AiTool::Claude => generate_claude_config(&server_name, &server_url).await,
        AiTool::Kimi => generate_kimi_config(&server_name, &server_url).await,
        AiTool::Gemini => generate_gemini_config(&server_name, &server_url).await,
        AiTool::Codex => generate_codex_config(&server_name, &server_url).await,
    }
}

/// Write MCP configuration to the appropriate file for an AI tool
pub async fn write_mcp_config(
    tool: AiTool,
    project_name: &str,
    project_id: &str,
    server_port: u16,
) -> Result<(), ContextError> {
    let config_path = tool.config_path()?;
    let config_content = generate_mcp_config(tool, project_name, project_id, server_port).await?;

    // Ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| {
            ContextError::Config(format!("Failed to create config directory: {}", e))
        })?;
    }

    // Read existing config if present
    let existing_config = if config_path.exists() {
        tokio::fs::read_to_string(&config_path).await.ok()
    } else {
        None
    };

    // Merge or create new config
    let merged_config = merge_mcp_config(existing_config, &config_content, tool).await?;

    // Write the config
    tokio::fs::write(&config_path, merged_config).await.map_err(|e| {
        ContextError::Config(format!("Failed to write config file: {}", e))
    })?;

    Ok(())
}

/// Generate configuration for Claude Code
/// Format: TBD - waiting for documentation
async fn generate_claude_config(_server_name: &str, _server_url: &str) -> Result<String, ContextError> {
    // TODO: Update with correct format from docs
    // Placeholder format - will be replaced with actual Claude Code format
    let config = serde_json::json!({
        "mcpServers": {
            "aiharness": {
                "url": _server_url,
                "transport": "http"
            }
        }
    });
    
    serde_json::to_string_pretty(&config)
        .map_err(|e| ContextError::Config(format!("Failed to serialize config: {}", e)))
}

/// Generate configuration for Kimi CLI
/// Format: ~/.kimi/mcp.json
async fn generate_kimi_config(_server_name: &str, server_url: &str) -> Result<String, ContextError> {
    let config = serde_json::json!({
        "mcpServers": {
            "aiharness": {
                "url": server_url,
                "transport": "http"
            }
        }
    });
    
    serde_json::to_string_pretty(&config)
        .map_err(|e| ContextError::Config(format!("Failed to serialize config: {}", e)))
}

/// Generate configuration for Gemini CLI
/// Format: TBD - waiting for documentation
async fn generate_gemini_config(_server_name: &str, server_url: &str) -> Result<String, ContextError> {
    // TODO: Update with correct format from docs
    let config = serde_json::json!({
        "mcpServers": {
            "aiharness": {
                "url": server_url,
                "transport": "http"
            }
        }
    });
    
    serde_json::to_string_pretty(&config)
        .map_err(|e| ContextError::Config(format!("Failed to serialize config: {}", e)))
}

/// Generate configuration for Codex CLI
/// Format: TBD - waiting for documentation
async fn generate_codex_config(_server_name: &str, server_url: &str) -> Result<String, ContextError> {
    // TODO: Update with correct format from docs
    let config = serde_json::json!({
        "mcpServers": {
            "aiharness": {
                "url": server_url,
                "transport": "http"
            }
        }
    });
    
    serde_json::to_string_pretty(&config)
        .map_err(|e| ContextError::Config(format!("Failed to serialize config: {}", e)))
}

/// Merge new MCP config with existing config
async fn merge_mcp_config(
    existing: Option<String>,
    new_config: &str,
    tool: AiTool,
) -> Result<String, ContextError> {
    match tool {
        AiTool::Claude | AiTool::Kimi | AiTool::Gemini | AiTool::Codex => {
            // Standard JSON merge for mcpServers
            let mut existing_json: serde_json::Value = if let Some(content) = existing {
                serde_json::from_str(&content)
                    .map_err(|e| ContextError::Config(format!("Invalid existing config: {}", e)))?
            } else {
                serde_json::json!({})
            };

            let new_json: serde_json::Value = serde_json::from_str(new_config)
                .map_err(|e| ContextError::Config(format!("Invalid new config: {}", e)))?;

            // Merge mcpServers
            if let Some(new_servers) = new_json.get("mcpServers") {
                let existing_servers = existing_json
                    .as_object_mut()
                    .ok_or_else(|| ContextError::Config("Invalid config structure".to_string()))?
                    .entry("mcpServers")
                    .or_insert_with(|| serde_json::json!({}))
                    .as_object_mut()
                    .ok_or_else(|| ContextError::Config("Invalid mcpServers structure".to_string()))?;

                for (key, value) in new_servers.as_object().unwrap_or(&serde_json::Map::new()) {
                    existing_servers.insert(key.clone(), value.clone());
                }
            }

            serde_json::to_string_pretty(&existing_json)
                .map_err(|e| ContextError::Config(format!("Failed to serialize merged config: {}", e)))
        }
    }
}

/// Get information about MCP configuration for all supported tools
pub fn get_mcp_config_info() -> Vec<AiToolInfo> {
    AiTool::all()
        .into_iter()
        .map(|tool| AiToolInfo {
            tool,
            name: tool.display_name().to_string(),
            config_path: tool.config_path().map(|p| p.to_string_lossy().to_string()).unwrap_or_default(),
        })
        .collect()
}

/// Information about an AI tool's MCP configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiToolInfo {
    pub tool: AiTool,
    pub name: String,
    pub config_path: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ai_tool_all_returns_all_tools() {
        let tools = AiTool::all();
        assert_eq!(tools.len(), 4);
        assert!(tools.contains(&AiTool::Claude));
        assert!(tools.contains(&AiTool::Kimi));
        assert!(tools.contains(&AiTool::Gemini));
        assert!(tools.contains(&AiTool::Codex));
    }

    #[test]
    fn ai_tool_display_names() {
        assert_eq!(AiTool::Claude.display_name(), "Claude Code");
        assert_eq!(AiTool::Kimi.display_name(), "Kimi CLI");
        assert_eq!(AiTool::Gemini.display_name(), "Gemini CLI");
        assert_eq!(AiTool::Codex.display_name(), "Codex CLI");
    }

    #[tokio::test]
    async fn generate_kimi_config_creates_valid_json() {
        let config = generate_kimi_config("test-project", "http://127.0.0.1:8787/mcp/test")
            .await
            .unwrap();
        
        let parsed: serde_json::Value = serde_json::from_str(&config).unwrap();
        assert!(parsed.get("mcpServers").is_some());
    }

    #[tokio::test]
    async fn merge_config_adds_new_server() {
        let existing = Some(r#"{"mcpServers":{"existing":{"url":"http://test"}}}"#.to_string());
        let new = r#"{"mcpServers":{"new":{"url":"http://new"}}}"#;
        
        let merged = merge_mcp_config(existing, new, AiTool::Kimi).await.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&merged).unwrap();
        
        let servers = parsed.get("mcpServers").unwrap();
        assert!(servers.get("existing").is_some());
        assert!(servers.get("new").is_some());
    }
}
