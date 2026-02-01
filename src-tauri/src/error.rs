//! Error types for AIHarness

use std::fmt;

/// Errors that can occur during tool execution
#[derive(Debug, Clone, PartialEq)]
pub enum ToolError {
    /// File not found
    FileNotFound(String),
    /// Permission denied
    PermissionDenied(String),
    /// Invalid path
    InvalidPath(String),
    /// File too large
    FileTooLarge { path: String, size: u64, max_size: u64 },
    /// IO error
    IoError(String),
    /// Invalid arguments
    InvalidArguments(String),
    /// Tool not found
    NotFound(String),
    /// Execution timeout
    Timeout { tool: String, duration_ms: u64 },
    /// Binary file (cannot read as text)
    BinaryFile(String),
}

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileNotFound(p) => write!(f, "File not found: {}", p),
            Self::PermissionDenied(p) => write!(f, "Permission denied: {}", p),
            Self::InvalidPath(p) => write!(f, "Invalid path: {}", p),
            Self::FileTooLarge { path, size, max_size } => {
                write!(f, "File too large: {} ({} bytes, max {})", path, size, max_size)
            }
            Self::IoError(e) => write!(f, "IO error: {}", e),
            Self::InvalidArguments(e) => write!(f, "Invalid arguments: {}", e),
            Self::NotFound(t) => write!(f, "Tool not found: {}", t),
            Self::Timeout { tool, duration_ms } => {
                write!(f, "Tool '{}' timed out after {}ms", tool, duration_ms)
            }
            Self::BinaryFile(p) => write!(f, "Binary file cannot be read as text: {}", p),
        }
    }
}

impl std::error::Error for ToolError {}

impl From<std::io::Error> for ToolError {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::NotFound => Self::FileNotFound(e.to_string()),
            std::io::ErrorKind::PermissionDenied => Self::PermissionDenied(e.to_string()),
            _ => Self::IoError(e.to_string()),
        }
    }
}

/// Errors that can occur in context management
#[derive(Debug, Clone, PartialEq)]
pub enum ContextError {
    /// Database error
    Database(String),
    /// File already in context
    AlreadyExists(String),
    /// File not in context
    NotInContext(String),
    /// Invalid file path
    InvalidPath(String),
}

impl fmt::Display for ContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(e) => write!(f, "Database error: {}", e),
            Self::AlreadyExists(p) => write!(f, "File already in context: {}", p),
            Self::NotInContext(p) => write!(f, "File not in context: {}", p),
            Self::InvalidPath(p) => write!(f, "Invalid path: {}", p),
        }
    }
}

impl std::error::Error for ContextError {}

impl From<rusqlite::Error> for ContextError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Database(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ToolError tests
    #[test]
    fn tool_error_display_file_not_found() {
        let err = ToolError::FileNotFound("/tmp/missing".to_string());
        assert_eq!(err.to_string(), "File not found: /tmp/missing");
    }

    #[test]
    fn tool_error_display_permission_denied() {
        let err = ToolError::PermissionDenied("/root/file".to_string());
        assert_eq!(err.to_string(), "Permission denied: /root/file");
    }

    #[test]
    fn tool_error_display_file_too_large() {
        let err = ToolError::FileTooLarge {
            path: "/tmp/big".to_string(),
            size: 1_000_000,
            max_size: 100_000,
        };
        assert!(err.to_string().contains("1"));
        assert!(err.to_string().contains("100000"));
    }

    #[test]
    fn tool_error_from_io_error_not_found() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let tool_err: ToolError = io_err.into();
        assert!(matches!(tool_err, ToolError::FileNotFound(_)));
    }

    #[test]
    fn tool_error_from_io_error_permission_denied() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let tool_err: ToolError = io_err.into();
        assert!(matches!(tool_err, ToolError::PermissionDenied(_)));
    }

    // ContextError tests
    #[test]
    fn context_error_display_database() {
        let err = ContextError::Database("locked".to_string());
        assert_eq!(err.to_string(), "Database error: locked");
    }

    #[test]
    fn context_error_display_already_exists() {
        let err = ContextError::AlreadyExists("/tmp/file".to_string());
        assert_eq!(err.to_string(), "File already in context: /tmp/file");
    }

    #[test]
    fn context_error_from_rusqlite() {
        let sqlite_err = rusqlite::Error::InvalidPath("bad".into());
        let ctx_err: ContextError = sqlite_err.into();
        assert!(matches!(ctx_err, ContextError::Database(_)));
    }

    #[test]
    fn tool_error_implements_error() {
        let err: Box<dyn std::error::Error> = Box::new(ToolError::InvalidArguments("bad".to_string()));
        assert!(err.to_string().contains("bad"));
    }
}
