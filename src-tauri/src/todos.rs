//! Ordered todo list storage for projects.

use crate::error::ContextError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub position: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct TodoStore {
    db_path: String,
}

impl TodoStore {
    pub async fn new(db_path: &str) -> Result<Self, ContextError> {
        tracing::info!("TodoStore::new() db_path={} self_ptr={:?}", db_path, &db_path as *const _);
        let store = Self {
            db_path: db_path.to_string(),
        };
        store.init_schema().await?;
        tracing::info!("TodoStore::new() DONE db_path={} self_ptr={:?}", store.db_path, &store as *const _);
        Ok(store)
    }

    fn get_db(&self) -> Result<rusqlite::Connection, ContextError> {
        Ok(rusqlite::Connection::open(&self.db_path)?)
    }

    async fn init_schema(&self) -> Result<(), ContextError> {
        let db = self.get_db()?;
        db.execute(
            "CREATE TABLE IF NOT EXISTS todos (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT,
                completed INTEGER NOT NULL,
                position INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        db.execute(
            "CREATE INDEX IF NOT EXISTS idx_todos_position ON todos(position)",
            [],
        )?;

        Ok(())
    }

    pub async fn list(&self) -> Result<Vec<TodoItem>, ContextError> {
        let db_path = &self.db_path;
        tracing::info!("TodoStore::list() db_path={} self_ptr={:?}", db_path, self as *const Self);
        
        // Verify the DB file exists
        let path_exists = std::path::Path::new(db_path).exists();
        tracing::info!("TodoStore::list() db_path={} exists={}", db_path, path_exists);
        
        let db = self.get_db()?;
        let mut stmt = db.prepare(
            "SELECT id, title, description, completed, position, created_at, updated_at
             FROM todos
             ORDER BY position ASC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(TodoItem {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                completed: row.get::<_, i64>(3)? != 0,
                position: row.get(4)?,
                created_at: row
                    .get::<_, String>(5)?
                    .parse()
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: row
                    .get::<_, String>(6)?
                    .parse()
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| ContextError::Database(e.to_string()))
    }

    pub async fn add(
        &self,
        title: &str,
        description: Option<String>,
        position: Option<i64>,
    ) -> Result<TodoItem, ContextError> {
        tracing::info!("TodoStore::add() title={} db_path={}", title, self.db_path);
        let db = self.get_db()?;
        let now = Utc::now();
        let id = uuid::Uuid::new_v4().to_string();
        let position = position.unwrap_or_else(|| self.next_position(&db).unwrap_or(0));

        shift_positions(&db, position, 1)?;

        db.execute(
            "INSERT INTO todos (id, title, description, completed, position, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                id,
                title.to_string(),
                description,
                0i64,
                position,
                now.to_rfc3339(),
                now.to_rfc3339(),
            ],
        )?;

        Ok(TodoItem {
            id,
            title: title.to_string(),
            description,
            completed: false,
            position,
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn remove(&self, id: &str) -> Result<(), ContextError> {
        let db = self.get_db()?;
        let position = find_position(&db, id)?;

        let rows = db.execute("DELETE FROM todos WHERE id = ?1", [id])?;
        if rows == 0 {
            return Err(ContextError::NotInContext(id.to_string()));
        }

        if let Some(position) = position {
            shift_positions(&db, position + 1, -1)?;
        }

        Ok(())
    }

    pub async fn set_completed(&self, id: &str, completed: bool) -> Result<(), ContextError> {
        let db = self.get_db()?;
        let now = Utc::now().to_rfc3339();
        let rows = db.execute(
            "UPDATE todos SET completed = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![if completed { 1 } else { 0 }, now, id],
        )?;

        if rows == 0 {
            return Err(ContextError::NotInContext(id.to_string()));
        }

        Ok(())
    }

    pub async fn get_next(&self) -> Result<Option<TodoItem>, ContextError> {
        let db = self.get_db()?;
        let result = db.query_row(
            "SELECT id, title, description, completed, position, created_at, updated_at
             FROM todos WHERE completed = 0 ORDER BY position ASC LIMIT 1",
            [],
            |row| {
                Ok(TodoItem {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    completed: row.get::<_, i64>(3)? != 0,
                    position: row.get(4)?,
                    created_at: row
                        .get::<_, String>(5)?
                        .parse()
                        .unwrap_or_else(|_| Utc::now()),
                    updated_at: row
                        .get::<_, String>(6)?
                        .parse()
                        .unwrap_or_else(|_| Utc::now()),
                })
            },
        );

        match result {
            Ok(todo) => Ok(Some(todo)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(ContextError::Database(e.to_string())),
        }
    }

    pub async fn move_to(&self, id: &str, new_position: i64) -> Result<(), ContextError> {
        let db = self.get_db()?;
        let current_position = find_position(&db, id)?
            .ok_or_else(|| ContextError::NotInContext(id.to_string()))?;

        if current_position == new_position {
            return Ok(());
        }

        if new_position > current_position {
            db.execute(
                "UPDATE todos SET position = position - 1 WHERE position > ?1 AND position <= ?2",
                [current_position, new_position],
            )
            .map_err(|e| ContextError::Database(e.to_string()))?;
        } else {
            db.execute(
                "UPDATE todos SET position = position + 1 WHERE position >= ?1 AND position < ?2",
                [new_position, current_position],
            )
            .map_err(|e| ContextError::Database(e.to_string()))?;
        }

        let now = Utc::now().to_rfc3339();
        db.execute(
            "UPDATE todos SET position = ?1, updated_at = ?2 WHERE id = ?3",
            (&new_position, &now, &id.to_string()),
        )
        .map_err(|e| ContextError::Database(e.to_string()))?;

        Ok(())
    }

    fn next_position(&self, db: &rusqlite::Connection) -> Result<i64, ContextError> {
        let max: Option<i64> = db
            .query_row("SELECT MAX(position) FROM todos", [], |row| row.get(0))
            .map_err(|e| ContextError::Database(e.to_string()))?;
        Ok(max.unwrap_or(-1) + 1)
    }
}

fn shift_positions(db: &rusqlite::Connection, start: i64, delta: i64) -> Result<(), ContextError> {
    db.execute(
        "UPDATE todos SET position = position + ?1 WHERE position >= ?2",
        [delta, start],
    )
    .map_err(|e| ContextError::Database(e.to_string()))?;
    Ok(())
}

fn find_position(db: &rusqlite::Connection, id: &str) -> Result<Option<i64>, ContextError> {
    let result = db.query_row(
        "SELECT position FROM todos WHERE id = ?1",
        [id],
        |row| row.get(0),
    );

    match result {
        Ok(pos) => Ok(Some(pos)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(ContextError::Database(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_store() -> (TodoStore, TempDir) {
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join("todo.db");
        let store = TodoStore::new(db_path.to_str().unwrap()).await.unwrap();
        (store, temp)
    }

    #[tokio::test]
    async fn add_and_list() {
        let (store, _temp) = create_store().await;
        store.add("Task", None, None).await.unwrap();
        let todos = store.list().await.unwrap();
        assert_eq!(todos.len(), 1);
    }

    #[tokio::test]
    async fn add_with_position_inserts() {
        let (store, _temp) = create_store().await;
        store.add("First", None, None).await.unwrap();
        store.add("Second", None, None).await.unwrap();
        store.add("Insert", None, Some(0)).await.unwrap();
        let todos = store.list().await.unwrap();
        assert_eq!(todos[0].title, "Insert");
    }

    #[tokio::test]
    async fn set_completed_updates() {
        let (store, _temp) = create_store().await;
        let item = store.add("Task", None, None).await.unwrap();
        store.set_completed(&item.id, true).await.unwrap();
        let todos = store.list().await.unwrap();
        assert!(todos[0].completed);
    }

    #[tokio::test]
    async fn get_next_returns_first_incomplete() {
        let (store, _temp) = create_store().await;
        let first = store.add("A", None, None).await.unwrap();
        let second = store.add("B", None, None).await.unwrap();
        store.set_completed(&first.id, true).await.unwrap();
        let next = store.get_next().await.unwrap().unwrap();
        assert_eq!(next.id, second.id);
    }

    #[tokio::test]
    async fn remove_shifts_positions() {
        let (store, _temp) = create_store().await;
        let first = store.add("A", None, None).await.unwrap();
        let second = store.add("B", None, None).await.unwrap();
        store.remove(&first.id).await.unwrap();
        let todos = store.list().await.unwrap();
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].id, second.id);
        assert_eq!(todos[0].position, 0);
    }

    #[tokio::test]
    async fn get_next_returns_none_when_all_complete() {
        let (store, _temp) = create_store().await;
        let item = store.add("A", None, None).await.unwrap();
        store.set_completed(&item.id, true).await.unwrap();
        let next = store.get_next().await.unwrap();
        assert!(next.is_none());
    }

    #[tokio::test]
    async fn move_to_higher_position() {
        let (store, _temp) = create_store().await;
        let a = store.add("A", None, None).await.unwrap();
        let b = store.add("B", None, None).await.unwrap();
        store.move_to(&a.id, 1).await.unwrap();
        let todos = store.list().await.unwrap();
        assert_eq!(todos[0].id, b.id);
        assert_eq!(todos[1].id, a.id);
    }

    #[tokio::test]
    async fn move_to_lower_position() {
        let (store, _temp) = create_store().await;
        let a = store.add("A", None, None).await.unwrap();
        let b = store.add("B", None, None).await.unwrap();
        store.move_to(&b.id, 0).await.unwrap();
        let todos = store.list().await.unwrap();
        assert_eq!(todos[0].id, b.id);
        assert_eq!(todos[1].id, a.id);
    }

    #[tokio::test]
    async fn remove_missing_returns_error() {
        let (store, _temp) = create_store().await;
        let err = store.remove("missing").await.unwrap_err();
        assert!(matches!(err, ContextError::NotInContext(_)));
    }

    #[tokio::test]
    async fn separate_databases_are_isolated() {
        // Create two separate stores with different DB paths
        let temp1 = TempDir::new().unwrap();
        let temp2 = TempDir::new().unwrap();
        let db_path1 = temp1.path().join("todo1.db");
        let db_path2 = temp2.path().join("todo2.db");
        
        let store1 = TodoStore::new(db_path1.to_str().unwrap()).await.unwrap();
        let store2 = TodoStore::new(db_path2.to_str().unwrap()).await.unwrap();
        
        // Add a todo to store1
        store1.add("Store1 Task", None, None).await.unwrap();
        
        // Verify store1 has the task
        let todos1 = store1.list().await.unwrap();
        assert_eq!(todos1.len(), 1);
        assert_eq!(todos1[0].title, "Store1 Task");
        
        // Verify store2 does NOT have the task
        let todos2 = store2.list().await.unwrap();
        assert_eq!(todos2.len(), 0, "Store2 should not see Store1's todos");
        
        // Add a different task to store2
        store2.add("Store2 Task", None, None).await.unwrap();
        
        // Verify isolation is maintained
        let todos1 = store1.list().await.unwrap();
        let todos2 = store2.list().await.unwrap();
        assert_eq!(todos1.len(), 1, "Store1 should still have only 1 task");
        assert_eq!(todos2.len(), 1, "Store2 should have 1 task");
        assert_eq!(todos1[0].title, "Store1 Task");
        assert_eq!(todos2[0].title, "Store2 Task");
    }
}
