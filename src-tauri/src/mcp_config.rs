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

    /// Get the configuration file path for this AI tool (for file-based configs)
    pub fn config_path(&self) -> Result<Option<PathBuf>, ContextError> {
        let home = dirs::home_dir().ok_or_else(|| {
            ContextError::Config("Could not determine home directory".to_string())
        })?;

        match self {
            AiTool::Claude => {
                // Claude uses CLI commands, not files
                Ok(None)
            }
            AiTool::Kimi => {
                // Kimi CLI: ~/.kimi/mcp.json
                Ok(Some(home.join(".kimi").join("mcp.json")))
            }
            AiTool::Gemini => {
                // Gemini CLI: ~/.gemini/settings.json
                // https://geminicli.com/docs/tools/mcp-server/
                Ok(Some(home.join(".gemini").join("settings.json")))
            }
            AiTool::Codex => {
                // Codex CLI: ~/.codex/config.yaml (YAML format!)
                // https://developers.openai.com/codex/mcp/
                Ok(Some(home.join(".codex").join("config.yaml")))
            }
        }
    }

    /// Whether this tool uses CLI commands (not files) for configuration
    pub fn uses_cli(&self) -> bool {
        matches!(self, AiTool::Claude)
    }
}

/// MCP server configuration structure (for file-based configs)
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
}

/// Result of an MCP configuration operation
#[derive(Debug, Clone, Serialize)]
pub struct McpSetupResult {
    pub success: bool,
    pub message: String,
    pub config_path: Option<String>,
}

/// Configure MCP for a specific AI tool and project
pub async fn configure_mcp(
    tool: AiTool,
    project_id: &str,
    server_port: u16,
) -> Result<McpSetupResult, ContextError> {
    let binary_path = detect_aiharness_binary()?;

    match tool {
        AiTool::Claude => configure_claude(project_id, &binary_path).await,
        AiTool::Kimi => configure_kimi(project_id, server_port).await,
        AiTool::Gemini => configure_gemini(project_id, server_port).await,
        AiTool::Codex => configure_codex(project_id, server_port).await,
    }
}

/// Detect the AIHarness binary path
/// 
/// This handles multiple scenarios:
/// - Running as built .app bundle on macOS
/// - Running from cargo run in development
/// - Running as installed binary
fn detect_aiharness_binary() -> Result<PathBuf, ContextError> {
    let current_exe = std::env::current_exe()
        .map_err(|e| ContextError::Config(format!("Cannot determine current executable: {}", e)))?;

    // If we're running from cargo build/debug, the exe is the binary directly
    // If we're in a .app bundle, we need to find the embedded binary
    
    // Check if we're in an .app bundle on macOS
    if cfg!(target_os = "macos") {
        let path_str = current_exe.to_string_lossy();
        if path_str.contains(".app/") {
            // We're in an app bundle - the binary should be at:
            // MyApp.app/Contents/MacOS/aiharness
            // But we might be running from the app itself
            if let Some(app_pos) = path_str.find(".app/") {
                let app_bundle = &path_str[..app_pos + 4];
                let binary_in_bundle = format!("{}/Contents/MacOS/aiharness", app_bundle);
                let bundle_path = PathBuf::from(&binary_in_bundle);
                if bundle_path.exists() {
                    return Ok(bundle_path);
                }
            }
        }
    }

    // Otherwise, use the current executable path
    if current_exe.exists() {
        return Ok(current_exe);
    }

    Err(ContextError::Config(
        "Cannot find AIHarness binary".to_string()
    ))
}

/// Find the Claude CLI binary
/// 
/// Checks PATH first, then common installation locations
fn find_claude_binary() -> Result<PathBuf, ContextError> {
    // First, check if 'claude' is in PATH
    match which::which("claude") {
        Ok(path) => return Ok(path),
        Err(_) => {
            // Check common installation locations
            let home = dirs::home_dir().ok_or_else(|| {
                ContextError::Config("Could not determine home directory".to_string())
            })?;
            
            #[cfg(target_os = "macos")]
            let common_paths = [
                home.join(".local").join("bin").join("claude"),
                home.join("bin").join("claude"),
                PathBuf::from("/usr/local/bin/claude"),
                PathBuf::from("/opt/homebrew/bin/claude"),
            ];
            
            #[cfg(target_os = "linux")]
            let common_paths = [
                home.join(".local").join("bin").join("claude"),
                home.join("bin").join("claude"),
                PathBuf::from("/usr/local/bin/claude"),
                PathBuf::from("/usr/bin/claude"),
            ];
            
            #[cfg(target_os = "windows")]
            let common_paths = [
                home.join("AppData").join("Local").join("Programs").join("claude").join("claude.exe"),
                home.join("bin").join("claude.exe"),
            ];
            
            for path in &common_paths {
                if path.exists() {
                    return Ok(path.clone());
                }
            }
            
            Err(ContextError::Config(
                "Claude Code not found. Please install Claude Code first:\n\
                 npm install -g @anthropic-ai/claude-code\n\
                 Or download from: https://claude.ai/download".to_string()
            ))
        }
    }
}

/// Configure Claude Code using CLI command
/// 
/// Command: claude mcp add --transport stdio <name> -- <binary> --mcp-stdio-proxy --project <project_id>
async fn configure_claude(project_id: &str, binary_path: &PathBuf) -> Result<McpSetupResult, ContextError> {
    let server_name = format!("aiharness-{}", project_id);
    let binary_str = binary_path.to_string_lossy();
    
    // Find the Claude binary
    let claude_path = find_claude_binary()?;

    // Build the command: claude mcp add --transport stdio <name> -- <binary> --mcp-stdio-proxy
    let output = tokio::process::Command::new(&claude_path)
        .args(&[
            "mcp",
            "add",
            "--transport",
            "stdio",
            &server_name,
            "--",
            &binary_str,
            "--mcp-stdio-proxy",
        ])
        .env("AIH_PORT", "8787")
        .env("AIH_PROJECT_ID", project_id)
        .output()
        .await
        .map_err(|e| ContextError::Config(format!("Failed to run claude command: {}", e)))?;

    if output.status.success() {
        Ok(McpSetupResult {
            success: true,
            message: format!("Added '{}' to Claude Code", server_name),
            config_path: None,
        })
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Check if it's already configured (not necessarily an error)
        if stderr.contains("already exists") {
            Ok(McpSetupResult {
                success: true,
                message: format!("'{}' is already configured in Claude Code", server_name),
                config_path: None,
            })
        } else {
            Ok(McpSetupResult {
                success: false,
                message: format!("Claude command failed: {}", stderr),
                config_path: None,
            })
        }
    }
}

/// Configure Kimi CLI using file-based config
async fn configure_kimi(project_id: &str, server_port: u16) -> Result<McpSetupResult, ContextError> {
    let config_path = match AiTool::Kimi.config_path()? {
        Some(p) => p,
        None => return Err(ContextError::Config("No config path for Kimi".to_string())),
    };

    let server_url = format!("http://127.0.0.1:{}/mcp/{}", server_port, project_id);
    let server_name = format!("aiharness-{}", project_id);

    // Create the config entry
    let config = serde_json::json!({
        "mcpServers": {
            server_name.clone(): {
                "url": server_url,
                "transport": "http"
            }
        }
    });

    // Read existing config if present
    let existing_config = if config_path.exists() {
        tokio::fs::read_to_string(&config_path).await.ok()
    } else {
        None
    };

    // Merge configs
    let merged = merge_mcp_config(existing_config, config).await?;

    // Ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| {
            ContextError::Config(format!("Failed to create config directory: {}", e))
        })?;
    }

    // Write the config
    tokio::fs::write(&config_path, merged).await.map_err(|e| {
        ContextError::Config(format!("Failed to write config file: {}", e))
    })?;

    Ok(McpSetupResult {
        success: true,
        message: format!("Added '{}' to Kimi CLI", server_name),
        config_path: Some(config_path.to_string_lossy().to_string()),
    })
}

/// Configure Gemini CLI using file-based config
/// 
/// Config location: ~/.gemini/settings.json
/// Format: { "mcpServers": { "name": { "url": "..." } } }
/// Docs: https://geminicli.com/docs/tools/mcp-server/
async fn configure_gemini(project_id: &str, server_port: u16) -> Result<McpSetupResult, ContextError> {
    let config_path = match AiTool::Gemini.config_path()? {
        Some(p) => p,
        None => return Err(ContextError::Config("No config path for Gemini".to_string())),
    };

    let server_url = format!("http://127.0.0.1:{}/mcp/{}", server_port, project_id);
    let server_name = format!("aiharness-{}", project_id);

    // Create the config entry
    let config = serde_json::json!({
        "mcpServers": {
            server_name.clone(): {
                "url": server_url
            }
        }
    });

    // Read existing config if present
    let existing_config = if config_path.exists() {
        tokio::fs::read_to_string(&config_path).await.ok()
    } else {
        None
    };

    // Merge configs
    let merged = merge_mcp_config(existing_config, config).await?;

    // Ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| {
            ContextError::Config(format!("Failed to create config directory: {}", e))
        })?;
    }

    // Write the config
    tokio::fs::write(&config_path, merged).await.map_err(|e| {
        ContextError::Config(format!("Failed to write config file: {}", e))
    })?;

    Ok(McpSetupResult {
        success: true,
        message: format!("Added '{}' to Gemini CLI", server_name),
        config_path: Some(config_path.to_string_lossy().to_string()),
    })
}

/// Configure Codex CLI using YAML-based config
/// 
/// Config location: ~/.codex/config.yaml
/// Format: 
///   mcpServers:
///     name:
///       url: https://...
/// Docs: https://developers.openai.com/codex/mcp/
async fn configure_codex(project_id: &str, server_port: u16) -> Result<McpSetupResult, ContextError> {
    let config_path = match AiTool::Codex.config_path()? {
        Some(p) => p,
        None => return Err(ContextError::Config("No config path for Codex".to_string())),
    };

    let server_url = format!("http://127.0.0.1:{}/mcp/{}", server_port, project_id);
    let server_name = format!("aiharness-{}", project_id);

    // Read existing config if present
    let existing_yaml = if config_path.exists() {
        tokio::fs::read_to_string(&config_path).await.ok()
    } else {
        None
    };

    // Merge YAML configs
    let merged = merge_codex_config(existing_yaml, &server_name, &server_url)?;

    // Ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| {
            ContextError::Config(format!("Failed to create config directory: {}", e))
        })?;
    }

    // Write the config
    tokio::fs::write(&config_path, merged).await.map_err(|e| {
        ContextError::Config(format!("Failed to write config file: {}", e))
    })?;

    Ok(McpSetupResult {
        success: true,
        message: format!("Added '{}' to Codex CLI", server_name),
        config_path: Some(config_path.to_string_lossy().to_string()),
    })
}

/// Merge new Codex MCP config with existing YAML config
fn merge_codex_config(
    existing: Option<String>,
    server_name: &str,
    server_url: &str,
) -> Result<String, ContextError> {
    use serde_yaml::Value;

    let mut config: Value = if let Some(content) = existing {
        serde_yaml::from_str(&content)
            .map_err(|e| ContextError::Config(format!("Invalid existing YAML config: {}", e)))?
    } else {
        Value::Mapping(serde_yaml::Mapping::new())
    };

    // Ensure mcpServers exists
    let mcp_servers = config
        .as_mapping_mut()
        .ok_or_else(|| ContextError::Config("Invalid YAML config structure".to_string()))?
        .entry(Value::String("mcpServers".to_string()))
        .or_insert_with(|| Value::Mapping(serde_yaml::Mapping::new()));

    // Add our server
    let server_mapping = mcp_servers
        .as_mapping_mut()
        .ok_or_else(|| ContextError::Config("Invalid mcpServers structure".to_string()))?;
    
    let mut server_config = serde_yaml::Mapping::new();
    server_config.insert(
        Value::String("url".to_string()),
        Value::String(server_url.to_string()),
    );
    
    server_mapping.insert(
        Value::String(server_name.to_string()),
        Value::Mapping(server_config),
    );

    serde_yaml::to_string(&config)
        .map_err(|e| ContextError::Config(format!("Failed to serialize YAML config: {}", e)))
}

/// Merge new MCP config with existing config
async fn merge_mcp_config(
    existing: Option<String>,
    new_config: serde_json::Value,
) -> Result<String, ContextError> {
    let mut existing_json: serde_json::Value = if let Some(content) = existing {
        serde_json::from_str(&content)
            .map_err(|e| ContextError::Config(format!("Invalid existing config: {}", e)))?
    } else {
        serde_json::json!({})
    };

    // Merge mcpServers
    if let Some(new_servers) = new_config.get("mcpServers") {
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

/// Get information about MCP configuration for all supported tools
pub fn get_mcp_config_info() -> Vec<AiToolInfo> {
    AiTool::all()
        .into_iter()
        .map(|tool| {
            let config_path_str = tool.config_path()
                .map(|p| p.map(|path| path.to_string_lossy().to_string()).unwrap_or_default())
                .unwrap_or_default();
            
            AiToolInfo {
                tool,
                name: tool.display_name().to_string(),
                uses_cli: tool.uses_cli(),
                config_path: if config_path_str.is_empty() { None } else { Some(config_path_str) },
            }
        })
        .collect()
}

/// Information about an AI tool's MCP configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiToolInfo {
    pub tool: AiTool,
    pub name: String,
    pub uses_cli: bool,
    pub config_path: Option<String>,
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
    }

    #[test]
    fn ai_tool_uses_cli() {
        assert!(AiTool::Claude.uses_cli());
        assert!(!AiTool::Kimi.uses_cli());
    }

    #[test]
    fn ai_tool_config_path() {
        // Claude returns None (uses CLI)
        assert!(AiTool::Claude.config_path().unwrap().is_none());
        // Kimi returns Some path
        assert!(AiTool::Kimi.config_path().unwrap().is_some());
    }

    #[tokio::test]
    async fn merge_config_adds_new_server() {
        let existing = Some(r#"{"mcpServers":{"existing":{"url":"http://test"}}}"#.to_string());
        let new = serde_json::json!({"mcpServers":{"new":{"url":"http://new"}}});
        
        let merged = merge_mcp_config(existing, new).await.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&merged).unwrap();
        
        let servers = parsed.get("mcpServers").unwrap();
        assert!(servers.get("existing").is_some());
        assert!(servers.get("new").is_some());
    }
}
