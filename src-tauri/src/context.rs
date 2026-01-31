//! Context management for AIHarness
//!
//! Manages files that are in the AI's context - files it should know about
//! and can reference without explicit tool calls.

use crate::error::ContextError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A file in the context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextFile {
    /// Unique ID
    pub id: String,
    /// Absolute path to the file
    pub path: String,
    /// Content hash for change detection
    pub content_hash: Option<String>,
    /// When the file was added to context
    pub added_at: DateTime<Utc>,
    /// Last time the file was read
    pub last_read_at: Option<DateTime<Utc>>,
}

/// Store for managing context files
/// 
/// Uses a connection per operation pattern since rusqlite::Connection
/// is not Send + Sync. This is acceptable for the low-concurrency use case.
pub struct ContextStore {
    db_path: String,
}

impl ContextStore {
    /// Create a new context store with the given database path
    /// 
    /// # Errors
    /// 
    /// Returns `ContextError` if the database cannot be opened or initialized
    pub async fn new(db_path: &str) -> Result<Self, ContextError> {
        let store = Self {
            db_path: db_path.to_string(),
        };
        store.init_schema().await?;
        
        Ok(store)
    }

    /// Get a database connection
    fn get_db(&self) -> Result<rusqlite::Connection, ContextError> {
        Ok(rusqlite::Connection::open(&self.db_path)?)
    }

    /// Initialize the database schema
    async fn init_schema(&self) -> Result<(), ContextError> {
        let db = self.get_db()?;
        
        db.execute(
            "CREATE TABLE IF NOT EXISTS context_files (
                id TEXT PRIMARY KEY,
                path TEXT UNIQUE NOT NULL,
                content_hash TEXT,
                added_at TEXT NOT NULL,
                last_read_at TEXT
            )",
            [],
        )?;

        db.execute(
            "CREATE INDEX IF NOT EXISTS idx_context_files_path ON context_files(path)",
            [],
        )?;

        Ok(())
    }

    /// Add a file to the context
    /// 
    /// # Errors
    /// 
    /// Returns `ContextError::AlreadyExists` if the file is already in context
    pub async fn add_file(&self, path: &str) -> Result<ContextFile, ContextError> {
        let path = std::fs::canonicalize(Path::new(path))
            .map_err(|_| ContextError::InvalidPath(path.to_string()))?;
        let path_str = path.to_string_lossy().to_string();

        let db = self.get_db()?;

        // Check if already exists
        let exists: bool = db.query_row(
            "SELECT 1 FROM context_files WHERE path = ?1",
            [&path_str],
            |_| Ok(true),
        ).unwrap_or(false);

        if exists {
            return Err(ContextError::AlreadyExists(path_str));
        }

        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        db.execute(
            "INSERT INTO context_files (id, path, content_hash, added_at, last_read_at)
             VALUES (?1, ?2, NULL, ?3, NULL)",
            [
                &id,
                &path_str,
                &now.to_rfc3339(),
            ],
        )?;

        Ok(ContextFile {
            id,
            path: path_str,
            content_hash: None,
            added_at: now,
            last_read_at: None,
        })
    }

    /// Remove a file from the context
    /// 
    /// # Errors
    /// 
    /// Returns `ContextError::NotInContext` if the file is not in context
    pub async fn remove_file(&self, path: &str) -> Result<(), ContextError> {
        // Try to canonicalize, but if file doesn't exist, use path as-is
        let path_str = match std::fs::canonicalize(Path::new(path)) {
            Ok(p) => p.to_string_lossy().to_string(),
            Err(_) => {
                // File doesn't exist - definitely not in context
                return Err(ContextError::NotInContext(path.to_string()));
            }
        };

        let db = self.get_db()?;
        let rows_affected = db.execute(
            "DELETE FROM context_files WHERE path = ?1",
            [&path_str],
        )?;

        if rows_affected == 0 {
            return Err(ContextError::NotInContext(path_str));
        }

        Ok(())
    }

    /// Get all files in the context
    pub async fn list_files(&self) -> Result<Vec<ContextFile>, ContextError> {
        let db = self.get_db()?;
        let mut stmt = db.prepare(
            "SELECT id, path, content_hash, added_at, last_read_at 
             FROM context_files 
             ORDER BY added_at DESC"
        )?;

        let files = stmt.query_map([], |row| {
            Ok(ContextFile {
                id: row.get(0)?,
                path: row.get(1)?,
                content_hash: row.get(2)?,
                added_at: row.get::<_, String>(3)?.parse().unwrap_or_else(|_| Utc::now()),
                last_read_at: row.get::<_, Option<String>>(4)?.and_then(|s| s.parse().ok()),
            })
        })?;

        files.collect::<Result<Vec<_>, _>>()
            .map_err(|e| ContextError::Database(e.to_string()))
    }

    /// Check if a file is in the context
    pub async fn contains(&self, path: &str) -> Result<bool, ContextError> {
        let path = match std::fs::canonicalize(Path::new(path)) {
            Ok(p) => p.to_string_lossy().to_string(),
            Err(_) => return Ok(false),
        };

        let db = self.get_db()?;
        let exists: bool = db.query_row(
            "SELECT 1 FROM context_files WHERE path = ?1",
            [&path],
            |_| Ok(true),
        ).unwrap_or(false);

        Ok(exists)
    }

    /// Get a single file from context
    pub async fn get_file(&self, path: &str) -> Result<Option<ContextFile>, ContextError> {
        let path = match std::fs::canonicalize(Path::new(path)) {
            Ok(p) => p.to_string_lossy().to_string(),
            Err(_) => return Ok(None),
        };

        let db = self.get_db()?;
        let result = db.query_row(
            "SELECT id, path, content_hash, added_at, last_read_at 
             FROM context_files WHERE path = ?1",
            [&path],
            |row| {
                Ok(ContextFile {
                    id: row.get(0)?,
                    path: row.get(1)?,
                    content_hash: row.get(2)?,
                    added_at: row.get::<_, String>(3)?.parse().unwrap_or_else(|_| Utc::now()),
                    last_read_at: row.get::<_, Option<String>>(4)?.and_then(|s| s.parse().ok()),
                })
            },
        );

        match result {
            Ok(file) => Ok(Some(file)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(ContextError::Database(e.to_string())),
        }
    }

    /// Update the last read timestamp for a file
    pub async fn mark_read(&self, path: &str) -> Result<(), ContextError> {
        let path = std::fs::canonicalize(Path::new(path))
            .map_err(|_| ContextError::InvalidPath(path.to_string()))?;
        let path_str = path.to_string_lossy().to_string();
        
        let now = Utc::now().to_rfc3339();
        
        let db = self.get_db()?;
        db.execute(
            "UPDATE context_files SET last_read_at = ?1 WHERE path = ?2",
            [&now, &path_str],
        )?;

        Ok(())
    }

    /// Clear all files from context
    pub async fn clear(&self) -> Result<(), ContextError> {
        let db = self.get_db()?;
        db.execute("DELETE FROM context_files", [])?;
        Ok(())
    }

    /// Get the number of files in context
    pub async fn count(&self) -> Result<usize, ContextError> {
        let db = self.get_db()?;
        let count: i64 = db.query_row(
            "SELECT COUNT(*) FROM context_files",
            [],
            |row| row.get(0),
        )?;

        Ok(count as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_store() -> (ContextStore, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = ContextStore::new(db_path.to_str().unwrap()).await.unwrap();
        (store, temp_dir)
    }

    #[tokio::test]
    async fn context_store_new_creates_database() {
        let (store, _temp) = create_test_store().await;
        // Should not panic
        drop(store);
    }

    #[tokio::test]
    async fn context_store_add_file_adds_to_context() {
        let (store, temp) = create_test_store().await;
        let file_path = temp.path().join("test.txt");
        tokio::fs::write(&file_path, "content").await.unwrap();

        let file = store.add_file(file_path.to_str().unwrap()).await.unwrap();

        assert!(!file.id.is_empty());
        assert!(file.path.contains("test.txt"));
    }

    #[tokio::test]
    async fn context_store_add_file_fails_for_duplicate() {
        let (store, temp) = create_test_store().await;
        let file_path = temp.path().join("test.txt");
        tokio::fs::write(&file_path, "content").await.unwrap();

        store.add_file(file_path.to_str().unwrap()).await.unwrap();
        let result = store.add_file(file_path.to_str().unwrap()).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ContextError::AlreadyExists(_)));
    }

    #[tokio::test]
    async fn context_store_add_file_fails_for_missing_file() {
        let (store, _temp) = create_test_store().await;

        let result = store.add_file("/tmp/nonexistent/file.txt").await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ContextError::InvalidPath(_)));
    }

    #[tokio::test]
    async fn context_store_remove_file_removes_from_context() {
        let (store, temp) = create_test_store().await;
        let file_path = temp.path().join("test.txt");
        tokio::fs::write(&file_path, "content").await.unwrap();

        store.add_file(file_path.to_str().unwrap()).await.unwrap();
        store.remove_file(file_path.to_str().unwrap()).await.unwrap();

        let count = store.count().await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn context_store_remove_file_fails_for_missing() {
        let (store, _temp) = create_test_store().await;

        let result = store.remove_file("/tmp/nonexistent.txt").await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ContextError::NotInContext(_)));
    }

    #[tokio::test]
    async fn context_store_list_files_returns_all_files() {
        let (store, temp) = create_test_store().await;
        
        for i in 0..3 {
            let file_path = temp.path().join(format!("file{}.txt", i));
            tokio::fs::write(&file_path, "content").await.unwrap();
            store.add_file(file_path.to_str().unwrap()).await.unwrap();
        }

        let files = store.list_files().await.unwrap();
        assert_eq!(files.len(), 3);
    }

    #[tokio::test]
    async fn context_store_contains_returns_true_for_existing() {
        let (store, temp) = create_test_store().await;
        let file_path = temp.path().join("test.txt");
        tokio::fs::write(&file_path, "content").await.unwrap();

        store.add_file(file_path.to_str().unwrap()).await.unwrap();
        let contains = store.contains(file_path.to_str().unwrap()).await.unwrap();

        assert!(contains);
    }

    #[tokio::test]
    async fn context_store_contains_returns_false_for_missing() {
        let (store, _temp) = create_test_store().await;

        let contains = store.contains("/tmp/nonexistent.txt").await.unwrap();

        assert!(!contains);
    }

    #[tokio::test]
    async fn context_store_get_file_returns_file() {
        let (store, temp) = create_test_store().await;
        let file_path = temp.path().join("test.txt");
        tokio::fs::write(&file_path, "content").await.unwrap();

        store.add_file(file_path.to_str().unwrap()).await.unwrap();
        let file = store.get_file(file_path.to_str().unwrap()).await.unwrap();

        assert!(file.is_some());
        assert!(file.unwrap().path.contains("test.txt"));
    }

    #[tokio::test]
    async fn context_store_get_file_returns_none_for_missing() {
        let (store, _temp) = create_test_store().await;

        let file = store.get_file("/tmp/nonexistent.txt").await.unwrap();

        assert!(file.is_none());
    }

    #[tokio::test]
    async fn context_store_clear_removes_all_files() {
        let (store, temp) = create_test_store().await;
        
        for i in 0..3 {
            let file_path = temp.path().join(format!("file{}.txt", i));
            tokio::fs::write(&file_path, "content").await.unwrap();
            store.add_file(file_path.to_str().unwrap()).await.unwrap();
        }

        store.clear().await.unwrap();
        let count = store.count().await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn context_store_count_returns_correct_number() {
        let (store, temp) = create_test_store().await;

        assert_eq!(store.count().await.unwrap(), 0);

        let file_path = temp.path().join("test.txt");
        tokio::fs::write(&file_path, "content").await.unwrap();
        store.add_file(file_path.to_str().unwrap()).await.unwrap();

        assert_eq!(store.count().await.unwrap(), 1);
    }

    #[tokio::test]
    async fn context_store_mark_read_updates_timestamp() {
        let (store, temp) = create_test_store().await;
        let file_path = temp.path().join("test.txt");
        tokio::fs::write(&file_path, "content").await.unwrap();

        store.add_file(file_path.to_str().unwrap()).await.unwrap();
        store.mark_read(file_path.to_str().unwrap()).await.unwrap();

        let file = store.get_file(file_path.to_str().unwrap()).await.unwrap().unwrap();
        assert!(file.last_read_at.is_some());
    }

    #[tokio::test]
    async fn context_store_persists_across_instances() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // First instance
        {
            let store = ContextStore::new(db_path.to_str().unwrap()).await.unwrap();
            let file_path = temp_dir.path().join("test.txt");
            tokio::fs::write(&file_path, "content").await.unwrap();
            store.add_file(file_path.to_str().unwrap()).await.unwrap();
        }

        // Second instance
        {
            let store = ContextStore::new(db_path.to_str().unwrap()).await.unwrap();
            let count = store.count().await.unwrap();
            assert_eq!(count, 1);
        }
    }

    #[tokio::test]
    async fn context_file_serialization_roundtrip() {
        let file = ContextFile {
            id: "test-id".to_string(),
            path: "/tmp/test.txt".to_string(),
            content_hash: Some("abc123".to_string()),
            added_at: Utc::now(),
            last_read_at: Some(Utc::now()),
        };

        let json = serde_json::to_string(&file).unwrap();
        let decoded: ContextFile = serde_json::from_str(&json).unwrap();

        assert_eq!(file.id, decoded.id);
        assert_eq!(file.path, decoded.path);
    }
}
