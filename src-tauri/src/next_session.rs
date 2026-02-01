//! Project-scoped next session briefing storage.

use crate::error::ContextError;
use chrono::{DateTime, Utc};
use rusqlite::params;

#[derive(Debug, Clone)]
pub struct NextSessionBriefing {
    pub content: String,
    pub updated_at: DateTime<Utc>,
}

pub struct NextSessionBriefingStore {
    db_path: String,
}

impl NextSessionBriefingStore {
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
            "CREATE TABLE IF NOT EXISTS next_session_briefing (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                content TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    pub async fn get(&self) -> Result<Option<NextSessionBriefing>, ContextError> {
        let db = self.get_db()?;
        let row = db.query_row(
            "SELECT content, updated_at FROM next_session_briefing WHERE id = 1",
            [],
            |row| {
                let updated_at: String = row.get(1)?;
                Ok(NextSessionBriefing {
                    content: row.get(0)?,
                    updated_at: updated_at.parse().unwrap_or_else(|_| Utc::now()),
                })
            },
        );

        match row {
            Ok(briefing) => Ok(Some(briefing)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(ContextError::Database(e.to_string())),
        }
    }

    pub async fn set(&self, content: &str) -> Result<NextSessionBriefing, ContextError> {
        let db = self.get_db()?;
        let now = Utc::now();
        db.execute(
            "INSERT INTO next_session_briefing (id, content, updated_at)
             VALUES (1, ?1, ?2)
             ON CONFLICT(id) DO UPDATE SET content = excluded.content, updated_at = excluded.updated_at",
            params![content, now.to_rfc3339()],
        )?;
        Ok(NextSessionBriefing {
            content: content.to_string(),
            updated_at: now,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn get_returns_none_when_empty() {
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join("briefing.db");
        let store = NextSessionBriefingStore::new(db_path.to_str().unwrap())
            .await
            .unwrap();
        let briefing = store.get().await.unwrap();
        assert!(briefing.is_none());
    }

    #[tokio::test]
    async fn set_then_get_returns_content() {
        let temp = TempDir::new().unwrap();
        let db_path = temp.path().join("briefing.db");
        let store = NextSessionBriefingStore::new(db_path.to_str().unwrap())
            .await
            .unwrap();
        store.set("hello").await.unwrap();
        let briefing = store.get().await.unwrap().unwrap();
        assert_eq!(briefing.content, "hello");
    }
}
