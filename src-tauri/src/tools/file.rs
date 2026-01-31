//! File system tools for AIHarness

use super::{Tool, ToolResult};
use crate::error::ToolError;
use async_trait::async_trait;
use serde_json::json;
use std::path::Path;

const MAX_FILE_SIZE: u64 = 1024 * 1024; // 1MB limit

/// Tool for reading file contents
pub struct ReadFileTool;

#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "Read the contents of a file. Returns the file content as text. \
         Will not read binary files. Limited to 1MB."
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The absolute path to the file to read"
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, ToolError> {
        let path_str = args
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArguments("Missing 'path' parameter".to_string()))?;

        let path = Path::new(path_str);
        
        // Validate path is absolute
        if !path.is_absolute() {
            return Err(ToolError::InvalidPath(
                format!("Path must be absolute: {}", path_str)
            ));
        }

        // Check file exists and get metadata
        let metadata = tokio::fs::metadata(path).await.map_err(ToolError::from)?;
        
        if !metadata.is_file() {
            return Err(ToolError::InvalidPath(
                format!("Path is not a file: {}", path_str)
            ));
        }

        // Check file size
        if metadata.len() > MAX_FILE_SIZE {
            return Err(ToolError::FileTooLarge {
                path: path_str.to_string(),
                size: metadata.len(),
                max_size: MAX_FILE_SIZE,
            });
        }

        // Read file content
        let content = tokio::fs::read_to_string(path).await?;
        
        Ok(ToolResult::success(content))
    }
}

/// Tool for writing file contents
pub struct WriteFileTool;

#[async_trait]
impl Tool for WriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }

    fn description(&self) -> &str {
        "Write content to a file. Creates the file if it doesn't exist, \
         overwrites if it does. Creates parent directories as needed."
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The absolute path to the file to write"
                },
                "content": {
                    "type": "string",
                    "description": "The content to write to the file"
                }
            },
            "required": ["path", "content"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, ToolError> {
        let path_str = args
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArguments("Missing 'path' parameter".to_string()))?;

        let content = args
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArguments("Missing 'content' parameter".to_string()))?;

        let path = Path::new(path_str);
        
        // Validate path is absolute
        if !path.is_absolute() {
            return Err(ToolError::InvalidPath(
                format!("Path must be absolute: {}", path_str)
            ));
        }

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(ToolError::from)?;
        }

        // Write file atomically (write to temp, then rename)
        let temp_path = path.with_extension("tmp");
        tokio::fs::write(&temp_path, content).await.map_err(ToolError::from)?;
        tokio::fs::rename(&temp_path, path).await.map_err(ToolError::from)?;

        Ok(ToolResult::success(format!(
            "Successfully wrote {} bytes to {}",
            content.len(),
            path_str
        )))
    }
}

/// Tool for listing directory contents
pub struct ListDirectoryTool;

#[async_trait]
impl Tool for ListDirectoryTool {
    fn name(&self) -> &str {
        "list_directory"
    }

    fn description(&self) -> &str {
        "List the contents of a directory. Returns files and subdirectories."
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The absolute path to the directory to list"
                },
                "recursive": {
                    "type": "boolean",
                    "description": "Whether to list recursively",
                    "default": false
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, ToolError> {
        let path_str = args
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArguments("Missing 'path' parameter".to_string()))?;

        let recursive = args.get("recursive").and_then(|v| v.as_bool()).unwrap_or(false);
        let path = Path::new(path_str);
        
        // Validate path is absolute
        if !path.is_absolute() {
            return Err(ToolError::InvalidPath(
                format!("Path must be absolute: {}", path_str)
            ));
        }

        if recursive {
            list_recursive(path, path_str).await
        } else {
            list_flat(path, path_str).await
        }
    }
}

async fn list_flat(path: &Path, base_path: &str) -> Result<ToolResult, ToolError> {
    let mut entries = tokio::fs::read_dir(path).await.map_err(ToolError::from)?;
    let mut files = Vec::new();
    let mut dirs = Vec::new();

    while let Some(entry) = entries.next_entry().await.map_err(ToolError::from)? {
        let name = entry.file_name().to_string_lossy().to_string();
        let metadata = entry.metadata().await.map_err(ToolError::from)?;
        
        if metadata.is_dir() {
            dirs.push(name);
        } else {
            files.push(name);
        }
    }

    // Sort for consistent output
    dirs.sort();
    files.sort();

    let output = format!(
        "Directory: {}\n\nSubdirectories ({}):\n{}\n\nFiles ({}):\n{}",
        base_path,
        dirs.len(),
        dirs.join("\n"),
        files.len(),
        files.join("\n")
    );

    Ok(ToolResult::success(output))
}

async fn list_recursive(path: &Path, base_path: &str) -> Result<ToolResult, ToolError> {
    let mut result = vec![format!("Directory tree: {}", base_path)];
    
    async fn walk(dir: &Path, prefix: &str, result: &mut Vec<String>) -> Result<(), ToolError> {
        let mut entries = tokio::fs::read_dir(dir).await.map_err(ToolError::from)?;
        let mut items = Vec::new();

        while let Some(entry) = entries.next_entry().await.map_err(ToolError::from)? {
            let name = entry.file_name().to_string_lossy().to_string();
            let metadata = entry.metadata().await.map_err(ToolError::from)?;
            items.push((name, metadata.is_dir(), entry.path()));
        }

        // Sort: directories first, then alphabetically
        items.sort_by(|a, b| {
            match (a.1, b.1) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.0.cmp(&b.0),
            }
        });

        let count = items.len();
        for (i, (name, is_dir, path)) in items.into_iter().enumerate() {
            let is_last = i == count - 1;
            let connector = if is_last { "└── " } else { "├── " };
            let symbol = if is_dir { "📁" } else { "📄" };
            
            result.push(format!("{}{}{} {}", prefix, connector, symbol, name));
            
            if is_dir {
                let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
                Box::pin(walk(&path, &new_prefix, result)).await?;
            }
        }
        
        Ok(())
    }

    walk(path, "", &mut result).await?;
    Ok(ToolResult::success(result.join("\n")))
}

/// Tool for searching files
pub struct SearchFilesTool;

#[async_trait]
impl Tool for SearchFilesTool {
    fn name(&self) -> &str {
        "search_files"
    }

    fn description(&self) -> &str {
        "Search for files containing a pattern. Supports simple string search. \
         Returns matching file paths with line numbers."
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The directory to search in"
                },
                "pattern": {
                    "type": "string",
                    "description": "The pattern to search for"
                },
                "recursive": {
                    "type": "boolean",
                    "description": "Whether to search recursively",
                    "default": true
                }
            },
            "required": ["path", "pattern"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult, ToolError> {
        let path_str = args
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArguments("Missing 'path' parameter".to_string()))?;

        let pattern = args
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArguments("Missing 'pattern' parameter".to_string()))?;

        let recursive = args.get("recursive").and_then(|v| v.as_bool()).unwrap_or(true);
        let path = Path::new(path_str);
        
        // Validate path is absolute
        if !path.is_absolute() {
            return Err(ToolError::InvalidPath(
                format!("Path must be absolute: {}", path_str)
            ));
        }

        let mut matches = Vec::new();
        let mut files_searched = 0;
        
        if recursive {
            search_recursive(path, pattern, &mut matches, &mut files_searched).await?;
        } else {
            search_flat(path, pattern, &mut matches, &mut files_searched).await?;
        }

        let output = if matches.is_empty() {
            format!("No matches found for '{}' in {} (searched {} files)", 
                    pattern, path_str, files_searched)
        } else {
            format!(
                "Found {} matches in {} files (searched {} files total):\n\n{}",
                matches.len(),
                matches.iter().map(|m: &String| m.split(':').next().unwrap()).collect::<std::collections::HashSet<_>>().len(),
                files_searched,
                matches.join("\n")
            )
        };

        Ok(ToolResult::success(output))
    }
}

async fn search_file(file_path: &Path, pattern: &str) -> Result<Vec<String>, ToolError> {
    let content = match tokio::fs::read_to_string(file_path).await {
        Ok(c) => c,
        Err(_) => return Ok(Vec::new()), // Skip binary/unreadable files
    };

    let path_str = file_path.to_string_lossy();
    let mut matches = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        if line.contains(pattern) {
            // Truncate long lines
            let display_line = if line.len() > 200 {
                format!("{}...", &line[..200])
            } else {
                line.to_string()
            };
            matches.push(format!("{}:{}: {}", path_str, line_num + 1, display_line));
        }
    }

    Ok(matches)
}

async fn search_flat(
    dir: &Path, 
    pattern: &str, 
    matches: &mut Vec<String>,
    files_searched: &mut usize
) -> Result<(), ToolError> {
    let mut entries = tokio::fs::read_dir(dir).await.map_err(ToolError::from)?;

    while let Some(entry) = entries.next_entry().await.map_err(ToolError::from)? {
        let path = entry.path();
        let metadata = entry.metadata().await.map_err(ToolError::from)?;

        if metadata.is_file() {
            *files_searched += 1;
            let file_matches = search_file(&path, pattern).await?;
            matches.extend(file_matches);
        }
    }

    Ok(())
}

async fn search_recursive(
    dir: &Path, 
    pattern: &str, 
    matches: &mut Vec<String>,
    files_searched: &mut usize
) -> Result<(), ToolError> {
    let mut entries = tokio::fs::read_dir(dir).await.map_err(ToolError::from)?;

    while let Some(entry) = entries.next_entry().await.map_err(ToolError::from)? {
        let path = entry.path();
        let metadata = entry.metadata().await.map_err(ToolError::from)?;

        if metadata.is_dir() {
            Box::pin(search_recursive(&path, pattern, matches, files_searched)).await?;
        } else if metadata.is_file() {
            *files_searched += 1;
            let file_matches = search_file(&path, pattern).await?;
            matches.extend(file_matches);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ReadFileTool tests
    #[tokio::test]
    async fn read_file_tool_reads_existing_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        tokio::fs::write(&file_path, "Hello, World!").await.unwrap();

        let tool = ReadFileTool;
        let args = json!({"path": file_path.to_str().unwrap()});
        let result = tool.execute(args).await.unwrap();

        assert!(result.success);
        assert_eq!(result.content, "Hello, World!");
    }

    #[tokio::test]
    async fn read_file_tool_fails_for_missing_file() {
        let tool = ReadFileTool;
        let args = json!({"path": "/tmp/nonexistent/file.txt"});
        let result = tool.execute(args).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ToolError::FileNotFound(_)));
    }

    #[tokio::test]
    async fn read_file_tool_fails_for_relative_path() {
        let tool = ReadFileTool;
        let args = json!({"path": "relative/path.txt"});
        let result = tool.execute(args).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ToolError::InvalidPath(_)));
    }

    #[tokio::test]
    async fn read_file_tool_fails_for_directory() {
        let temp_dir = TempDir::new().unwrap();
        let tool = ReadFileTool;
        let args = json!({"path": temp_dir.path().to_str().unwrap()});
        let result = tool.execute(args).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn read_file_tool_fails_for_large_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("large.txt");
        let large_content = "x".repeat((MAX_FILE_SIZE + 1) as usize);
        tokio::fs::write(&file_path, large_content).await.unwrap();

        let tool = ReadFileTool;
        let args = json!({"path": file_path.to_str().unwrap()});
        let result = tool.execute(args).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ToolError::FileTooLarge { .. }));
    }

    // WriteFileTool tests
    #[tokio::test]
    async fn write_file_tool_creates_new_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("new.txt");

        let tool = WriteFileTool;
        let args = json!({
            "path": file_path.to_str().unwrap(),
            "content": "New content"
        });
        let result = tool.execute(args).await.unwrap();

        assert!(result.success);
        let content = tokio::fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(content, "New content");
    }

    #[tokio::test]
    async fn write_file_tool_overwrites_existing_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("existing.txt");
        tokio::fs::write(&file_path, "Old content").await.unwrap();

        let tool = WriteFileTool;
        let args = json!({
            "path": file_path.to_str().unwrap(),
            "content": "New content"
        });
        let result = tool.execute(args).await.unwrap();

        assert!(result.success);
        let content = tokio::fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(content, "New content");
    }

    #[tokio::test]
    async fn write_file_tool_creates_parent_directories() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("a/b/c/deep.txt");

        let tool = WriteFileTool;
        let args = json!({
            "path": file_path.to_str().unwrap(),
            "content": "Deep content"
        });
        let result = tool.execute(args).await.unwrap();

        assert!(result.success);
        assert!(file_path.exists());
    }

    #[tokio::test]
    async fn write_file_tool_fails_for_missing_path() {
        let tool = WriteFileTool;
        let args = json!({"content": "test"});
        let result = tool.execute(args).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ToolError::InvalidArguments(_)));
    }

    #[tokio::test]
    async fn write_file_tool_fails_for_relative_path() {
        let tool = WriteFileTool;
        let args = json!({
            "path": "relative.txt",
            "content": "test"
        });
        let result = tool.execute(args).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ToolError::InvalidPath(_)));
    }

    // ListDirectoryTool tests
    #[tokio::test]
    async fn list_directory_tool_lists_flat_directory() {
        let temp_dir = TempDir::new().unwrap();
        tokio::fs::write(temp_dir.path().join("file1.txt"), "").await.unwrap();
        tokio::fs::write(temp_dir.path().join("file2.txt"), "").await.unwrap();
        tokio::fs::create_dir(temp_dir.path().join("subdir")).await.unwrap();

        let tool = ListDirectoryTool;
        let args = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "recursive": false
        });
        let result = tool.execute(args).await.unwrap();

        assert!(result.success);
        assert!(result.content.contains("file1.txt"));
        assert!(result.content.contains("file2.txt"));
        assert!(result.content.contains("subdir"));
    }

    #[tokio::test]
    async fn list_directory_tool_lists_recursively() {
        let temp_dir = TempDir::new().unwrap();
        let subdir = temp_dir.path().join("subdir");
        tokio::fs::create_dir(&subdir).await.unwrap();
        tokio::fs::write(subdir.join("nested.txt"), "").await.unwrap();

        let tool = ListDirectoryTool;
        let args = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "recursive": true
        });
        let result = tool.execute(args).await.unwrap();

        assert!(result.success);
        assert!(result.content.contains("nested.txt"));
    }

    #[tokio::test]
    async fn list_directory_tool_fails_for_missing_directory() {
        let tool = ListDirectoryTool;
        let args = json!({"path": "/tmp/nonexistent/dir"});
        let result = tool.execute(args).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn list_directory_tool_fails_for_relative_path() {
        let tool = ListDirectoryTool;
        let args = json!({"path": "relative/dir"});
        let result = tool.execute(args).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ToolError::InvalidPath(_)));
    }

    #[tokio::test]
    async fn list_directory_tool_handles_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        
        let tool = ListDirectoryTool;
        let args = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "recursive": false
        });
        let result = tool.execute(args).await.unwrap();

        assert!(result.success);
        assert!(result.content.contains("Files (0)"));
    }

    // SearchFilesTool tests
    #[tokio::test]
    async fn search_files_tool_finds_matches() {
        let temp_dir = TempDir::new().unwrap();
        tokio::fs::write(temp_dir.path().join("file1.txt"), "Hello world").await.unwrap();
        tokio::fs::write(temp_dir.path().join("file2.txt"), "Goodbye world").await.unwrap();

        let tool = SearchFilesTool;
        let args = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "pattern": "world"
        });
        let result = tool.execute(args).await.unwrap();

        assert!(result.success);
        assert!(result.content.contains("file1.txt"));
        assert!(result.content.contains("file2.txt"));
    }

    #[tokio::test]
    async fn search_files_tool_returns_no_matches() {
        let temp_dir = TempDir::new().unwrap();
        tokio::fs::write(temp_dir.path().join("file.txt"), "Hello world").await.unwrap();

        let tool = SearchFilesTool;
        let args = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "pattern": "xyz123"
        });
        let result = tool.execute(args).await.unwrap();

        assert!(result.success);
        assert!(result.content.contains("No matches found"));
    }

    #[tokio::test]
    async fn search_files_tool_searches_recursively() {
        let temp_dir = TempDir::new().unwrap();
        let subdir = temp_dir.path().join("subdir");
        tokio::fs::create_dir(&subdir).await.unwrap();
        tokio::fs::write(subdir.join("nested.txt"), "target").await.unwrap();

        let tool = SearchFilesTool;
        let args = json!({
            "path": temp_dir.path().to_str().unwrap(),
            "pattern": "target"
        });
        let result = tool.execute(args).await.unwrap();

        assert!(result.success);
        assert!(result.content.contains("nested.txt"));
    }

    #[tokio::test]
    async fn search_files_tool_fails_for_relative_path() {
        let tool = SearchFilesTool;
        let args = json!({
            "path": "relative",
            "pattern": "test"
        });
        let result = tool.execute(args).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ToolError::InvalidPath(_)));
    }

    #[tokio::test]
    async fn search_files_tool_handles_missing_pattern() {
        let temp_dir = TempDir::new().unwrap();
        let tool = SearchFilesTool;
        let args = json!({"path": temp_dir.path().to_str().unwrap()});
        let result = tool.execute(args).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ToolError::InvalidArguments(_)));
    }
}
