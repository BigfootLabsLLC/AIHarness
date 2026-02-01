//! MCP stdio proxy mode
//!
//! Forwards JSON-RPC over stdio to the running app's HTTP MCP endpoint.

use serde_json::Value;
use std::error::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

const DEFAULT_PORT: u16 = 8787;

fn mcp_url(port: u16) -> String {
    format!("http://127.0.0.1:{}/mcp", port)
}

fn health_url(port: u16) -> String {
    format!("http://127.0.0.1:{}/", port)
}

/// Run MCP stdio proxy mode, forwarding requests to the running HTTP server.
pub async fn run_stdio_proxy() -> Result<(), Box<dyn Error>> {
    let port = resolve_port();
    let client = reqwest::Client::new();

    ensure_server_available(&client, port).await?;

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();
    let mut stdout = stdout;

    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }

        let request = match parse_json_rpc_line(&line) {
            Ok(value) => value,
            Err(error_json) => {
                write_line(&mut stdout, &error_json).await?;
                continue;
            }
        };

        let is_notification = is_notification(&request);
        let response = forward_request(&client, port, &request).await;

        if is_notification {
            continue;
        }

        let output = match response {
            Ok(text) => text,
            Err(error_json) => error_json,
        };

        write_line(&mut stdout, &output).await?;
    }

    Ok(())
}

/// Resolve the HTTP server port from env or default.
fn resolve_port() -> u16 {
    std::env::var("AIH_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(DEFAULT_PORT)
}

/// Validate that the HTTP server is reachable before proxying.
async fn ensure_server_available(client: &reqwest::Client, port: u16) -> Result<(), Box<dyn Error>> {
    let health = health_url(port);
    if client.get(health).send().await.is_err() {
        let msg = format!(
            "AIHarness HTTP server not found on port {}. Start the app first.",
            port
        );
        eprintln!("{}", msg);
        return Err(msg.into());
    }

    Ok(())
}

/// Parse a JSON-RPC request line or return a JSON-RPC error payload.
fn parse_json_rpc_line(line: &str) -> Result<Value, String> {
    serde_json::from_str::<Value>(line).map_err(|e| {
        serde_json::json!({
            "jsonrpc": "2.0",
            "error": {
                "code": -32700,
                "message": format!("Invalid JSON: {}", e)
            },
            "id": null
        })
        .to_string()
    })
}

/// Determine whether a JSON-RPC request is a notification (no id).
fn is_notification(request: &Value) -> bool {
    request.get("id").is_none() || request.get("id") == Some(&Value::Null)
}

/// Forward a JSON-RPC request to the HTTP MCP endpoint.
async fn forward_request(
    client: &reqwest::Client,
    port: u16,
    request: &Value,
) -> Result<String, String> {
    let response = client
        .post(mcp_url(port))
        .json(request)
        .send()
        .await;

    match response {
        Ok(resp) => resp.text().await.map_err(|e| {
            serde_json::json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": -32603,
                    "message": format!("Failed reading response: {}", e)
                },
                "id": null
            })
            .to_string()
        }),
        Err(e) => Err(
            serde_json::json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": -32603,
                    "message": format!("HTTP MCP proxy error: {}", e)
                },
                "id": null
            })
            .to_string(),
        ),
    }
}

/// Write a single line response to stdout.
async fn write_line(stdout: &mut tokio::io::Stdout, text: &str) -> Result<(), Box<dyn Error>> {
    stdout.write_all(text.as_bytes()).await?;
    stdout.write_all(b"\n").await?;
    stdout.flush().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_port_defaults() {
        std::env::remove_var("AIH_PORT");
        assert_eq!(resolve_port(), DEFAULT_PORT);
    }

    #[test]
    fn resolve_port_from_env() {
        std::env::set_var("AIH_PORT", "9001");
        assert_eq!(resolve_port(), 9001);
        std::env::remove_var("AIH_PORT");
    }

    #[test]
    fn parse_json_rpc_line_rejects_invalid() {
        let err = parse_json_rpc_line("{bad json").unwrap_err();
        let value: Value = serde_json::from_str(&err).unwrap();
        assert_eq!(value.get("error").unwrap().get("code").unwrap(), -32700);
    }

    #[test]
    fn parse_json_rpc_line_accepts_valid() {
        let request = r#"{"jsonrpc":"2.0","method":"tools/list","id":1}"#;
        let value = parse_json_rpc_line(request).unwrap();
        assert_eq!(value.get("method").unwrap(), "tools/list");
    }

    #[test]
    fn is_notification_true_without_id() {
        let value = serde_json::json!({"jsonrpc":"2.0","method":"tools/list"});
        assert!(is_notification(&value));
    }

    #[test]
    fn is_notification_true_with_null_id() {
        let value = serde_json::json!({"jsonrpc":"2.0","method":"tools/list","id": null});
        assert!(is_notification(&value));
    }

    #[test]
    fn is_notification_false_with_id() {
        let value = serde_json::json!({"jsonrpc":"2.0","method":"tools/list","id": 1});
        assert!(!is_notification(&value));
    }
}
