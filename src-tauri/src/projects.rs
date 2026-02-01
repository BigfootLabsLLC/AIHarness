//! Project registry and per-project storage.

use crate::{context::ContextStore, error::ContextError, todos::TodoStore};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Project metadata stored in the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub id: String,
    pub name: String,
    pub root_path: String,
    pub db_path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Registry of all projects.
pub struct ProjectRegistry {
    db_path: String,
}

impl ProjectRegistry {
    pub async fn new(db_path: &str) -> Result<Self, ContextError> {
        let registry = Self {
            db_path: db_path.to_string(),
        };
        registry.init_schema().await?;
        Ok(registry)
    }

    fn get_db(&self) -> Result<rusqlite::Connection, ContextError> {
        Ok(rusqlite::Connection::open(&self.db_path)?)
    }

    async fn init_schema(&self) -> Result<(), ContextError> {
        let db = self.get_db()?;
        db.execute(
            "CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                root_path TEXT NOT NULL,
                db_path TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        db.execute(
            "CREATE INDEX IF NOT EXISTS idx_projects_root ON projects(root_path)",
            [],
        )?;

        Ok(())
    }

    pub async fn create_project(&self, name: &str, root_path: &str) -> Result<ProjectInfo, ContextError> {
        self.create_project_with_id(uuid::Uuid::new_v4().to_string(), name, root_path)
            .await
    }

    pub async fn create_project_with_id(
        &self,
        id: String,
        name: &str,
        root_path: &str,
    ) -> Result<ProjectInfo, ContextError> {
        let root = std::fs::canonicalize(Path::new(root_path))
            .map_err(|_| ContextError::InvalidPath(root_path.to_string()))?;
        let root_path = root.to_string_lossy().to_string();

        let db_path = ensure_project_db_path(&root)?;
        let now = Utc::now();

        let db = self.get_db()?;
        db.execute(
            "INSERT INTO projects (id, name, root_path, db_path, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            [
                &id,
                &name.to_string(),
                &root_path,
                &db_path,
                &now.to_rfc3339(),
                &now.to_rfc3339(),
            ],
        )?;

        Ok(ProjectInfo {
            id,
            name: name.to_string(),
            root_path,
            db_path,
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn list_projects(&self) -> Result<Vec<ProjectInfo>, ContextError> {
        let db = self.get_db()?;
        let mut stmt = db.prepare(
            "SELECT id, name, root_path, db_path, created_at, updated_at
             FROM projects
             ORDER BY updated_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(ProjectInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                root_path: row.get(2)?,
                db_path: row.get(3)?,
                created_at: row.get::<_, String>(4)?.parse().unwrap_or_else(|_| Utc::now()),
                updated_at: row.get::<_, String>(5)?.parse().unwrap_or_else(|_| Utc::now()),
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| ContextError::Database(e.to_string()))
    }

    pub async fn get_project(&self, project_id: &str) -> Result<Option<ProjectInfo>, ContextError> {
        let db = self.get_db()?;
        let result = db.query_row(
            "SELECT id, name, root_path, db_path, created_at, updated_at
             FROM projects WHERE id = ?1",
            [project_id],
            |row| {
                Ok(ProjectInfo {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    root_path: row.get(2)?,
                    db_path: row.get(3)?,
                    created_at: row.get::<_, String>(4)?.parse().unwrap_or_else(|_| Utc::now()),
                    updated_at: row.get::<_, String>(5)?.parse().unwrap_or_else(|_| Utc::now()),
                })
            },
        );

        match result {
            Ok(project) => Ok(Some(project)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(ContextError::Database(e.to_string())),
        }
    }
}

/// Project-scoped stores.
#[derive(Clone)]
pub struct ProjectStore {
    pub info: ProjectInfo,
    pub context_store: Arc<RwLock<ContextStore>>,
    pub todo_store: Arc<RwLock<TodoStore>>,
}

impl ProjectStore {
    pub async fn new(info: ProjectInfo) -> Result<Self, ContextError> {
        let context_store = ContextStore::new(&info.db_path).await?;
        let todo_store = TodoStore::new(&info.db_path).await?;
        Ok(Self {
            info,
            context_store: Arc::new(RwLock::new(context_store)),
            todo_store: Arc::new(RwLock::new(todo_store)),
        })
    }
}

/// Cache project stores in memory.
pub struct ProjectStoreCache {
    stores: RwLock<HashMap<String, Arc<ProjectStore>>>,
}

impl ProjectStoreCache {
    pub fn new() -> Self {
        Self {
            stores: RwLock::new(HashMap::new()),
        }
    }

    pub async fn get(&self, project_id: &str) -> Option<Arc<ProjectStore>> {
        self.stores.read().await.get(project_id).cloned()
    }

    pub async fn insert(&self, store: Arc<ProjectStore>) {
        self.stores
            .write()
            .await
            .insert(store.info.id.clone(), store);
    }
}

fn ensure_project_db_path(root: &Path) -> Result<String, ContextError> {
    let dir = root.join(".aiharness");
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| ContextError::Database(e.to_string()))?;
    }
    let db_path = dir.join("project.db");
    Ok(db_path.to_string_lossy().to_string())
}

pub fn default_project_root(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("projects").join("default")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn registry_creates_project() {
        let temp_dir = TempDir::new().unwrap();
        let registry_path = temp_dir.path().join("registry.db");
        let registry = ProjectRegistry::new(registry_path.to_str().unwrap()).await.unwrap();

        let project_root = temp_dir.path().join("proj");
        fs::create_dir_all(&project_root).unwrap();

        let project = registry
            .create_project("Test Project", project_root.to_str().unwrap())
            .await
            .unwrap();

        assert_eq!(project.name, "Test Project");
        assert!(project.db_path.ends_with(".aiharness/project.db"));
    }

    #[tokio::test]
    async fn registry_lists_projects() {
        let temp_dir = TempDir::new().unwrap();
        let registry_path = temp_dir.path().join("registry.db");
        let registry = ProjectRegistry::new(registry_path.to_str().unwrap()).await.unwrap();

        let project_root = temp_dir.path().join("proj");
        fs::create_dir_all(&project_root).unwrap();

        registry
            .create_project("Test Project", project_root.to_str().unwrap())
            .await
            .unwrap();

        let projects = registry.list_projects().await.unwrap();
        assert_eq!(projects.len(), 1);
    }
}
