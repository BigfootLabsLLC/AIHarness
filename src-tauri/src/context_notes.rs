//! Project-scoped context notes (manual text lines).

use crate::error::ContextError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextNote {
    pub id: String,
    pub content: String,
    pub position: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct ContextNoteStore {
    db_path: String,
}

impl ContextNoteStore {
    pub async fn new(db_path: &str) -> Result<Self, ContextError> {
        let store = Self {
            db_path: db_path.to_string(),
        };
        store.init_schema().await?;
        Ok(store)
    }

    fn get_db(&self) -> Result<rusqlite::Connection, ContextError> {
        Ok(rusqlite::Connection::open(&self.db_path)?)
    }

    async fn init_schema(&self) -> Result<(), ContextError> {
        let db = self.get_db()?;
        db.execute(
            "CREATE TABLE IF NOT EXISTS context_notes (
                id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                position INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        db.execute(
            "CREATE INDEX IF NOT EXISTS idx_context_notes_position ON context_notes(position)",
            [],
        )?;

        Ok(())
    }

    pub async fn list(&self) -> Result<Vec<ContextNote>, ContextError> {
        let db = self.get_db()?;
        let mut stmt = db.prepare(
            "SELECT id, content, position, created_at, updated_at
             FROM context_notes
             ORDER BY position ASC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(ContextNote {
                id: row.get(0)?,
                content: row.get(1)?,
                position: row.get(2)?,
                created_at: row
                    .get::<_, String>(3)?
                    .parse()
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: row
                    .get::<_, String>(4)?
                    .parse()
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| ContextError::Database(e.to_string()))
    }

    pub async fn add(&self, content: &str, position: Option<i64>) -> Result<ContextNote, ContextError> {
        let db = self.get_db()?;
        let now = Utc::now();
        let id = uuid::Uuid::new_v4().to_string();
        let position = position.unwrap_or_else(|| self.next_position(&db).unwrap_or(0));

        shift_positions(&db, position, 1)?;

        db.execute(
            "INSERT INTO context_notes (id, content, position, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![id, content.to_string(), position, now.to_rfc3339(), now.to_rfc3339()],
        )?;

        Ok(ContextNote {
            id,
            content: content.to_string(),
            position,
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn remove(&self, id: &str) -> Result<(), ContextError> {
        let db = self.get_db()?;
        let position = find_position(&db, id)?;

        let rows = db.execute("DELETE FROM context_notes WHERE id = ?1", [id])?;
        if rows == 0 {
            return Err(ContextError::NotInContext(id.to_string()));
        }

        if let Some(position) = position {
            shift_positions(&db, position + 1, -1)?;
        }

        Ok(())
    }

    pub async fn update(&self, id: &str, content: &str) -> Result<(), ContextError> {
        let db = self.get_db()?;
        let now = Utc::now().to_rfc3339();
        let rows = db.execute(
            "UPDATE context_notes SET content = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![content.to_string(), now, id],
        )?;

        if rows == 0 {
            return Err(ContextError::NotInContext(id.to_string()));
        }

        Ok(())
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
                "UPDATE context_notes SET position = position - 1 WHERE position > ?1 AND position <= ?2",
                [current_position, new_position],
            )
            .map_err(|e| ContextError::Database(e.to_string()))?;
        } else {
            db.execute(
                "UPDATE context_notes SET position = position + 1 WHERE position >= ?1 AND position < ?2",
                [new_position, current_position],
            )
            .map_err(|e| ContextError::Database(e.to_string()))?;
        }

        let now = Utc::now().to_rfc3339();
        db.execute(
            "UPDATE context_notes SET position = ?1, updated_at = ?2 WHERE id = ?3",
            (&new_position, &now, &id.to_string()),
        )
        .map_err(|e| ContextError::Database(e.to_string()))?;

        Ok(())
    }

    fn next_position(&self, db: &rusqlite::Connection) -> Result<i64, ContextError> {
        let max: Option<i64> = db
            .query_row("SELECT MAX(position) FROM context_notes", [], |row| row.get(0))
            .map_err(|e| ContextError::Database(e.to_string()))?;
        Ok(max.unwrap_or(-1) + 1)
    }
}

fn shift_positions(db: &rusqlite::Connection, start: i64, delta: i64) -> Result<(), ContextError> {
    db.execute(
        "UPDATE context_notes SET position = position + ?1 WHERE position >= ?2",
        [delta, start],
    )
    .map_err(|e| ContextError::Database(e.to_string()))?;
    Ok(())
}

fn find_position(db: &rusqlite::Connection, id: &str) -> Result<Option<i64>, ContextError> {
    let result = db.query_row(
        "SELECT position FROM context_notes WHERE id = ?1",
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

    async fn create_store() -> (ContextNoteStore, TempDir) {
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join("notes.db");
        let store = ContextNoteStore::new(db_path.to_str().unwrap()).await.unwrap();
        (store, temp)
    }

    #[tokio::test]
    async fn add_and_list() {
        let (store, _temp) = create_store().await;
        store.add("Line", None).await.unwrap();
        let notes = store.list().await.unwrap();
        assert_eq!(notes.len(), 1);
    }

    #[tokio::test]
    async fn update_changes_content() {
        let (store, _temp) = create_store().await;
        let note = store.add("Line", None).await.unwrap();
        store.update(&note.id, "New").await.unwrap();
        let notes = store.list().await.unwrap();
        assert_eq!(notes[0].content, "New");
    }

    #[tokio::test]
    async fn move_reorders() {
        let (store, _temp) = create_store().await;
        let a = store.add("A", None).await.unwrap();
        let b = store.add("B", None).await.unwrap();
        store.move_to(&b.id, 0).await.unwrap();
        let notes = store.list().await.unwrap();
        assert_eq!(notes[0].id, b.id);
        assert_eq!(notes[1].id, a.id);
    }
}
