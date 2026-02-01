//! Project-scoped build commands.

use crate::error::ContextError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildCommand {
    pub id: String,
    pub name: String,
    pub command: String,
    pub working_dir: Option<String>,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
}

pub struct BuildCommandStore {
    db_path: String,
}

impl BuildCommandStore {
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
            "CREATE TABLE IF NOT EXISTS build_commands (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                command TEXT NOT NULL,
                working_dir TEXT,
                is_default INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        db.execute(
            "CREATE INDEX IF NOT EXISTS idx_build_commands_name ON build_commands(name)",
            [],
        )?;

        ensure_column(&db, "build_commands", "working_dir", "TEXT")?;
        ensure_column(&db, "build_commands", "is_default", "INTEGER NOT NULL DEFAULT 0")?;

        Ok(())
    }

    pub async fn list(&self) -> Result<Vec<BuildCommand>, ContextError> {
        let db = self.get_db()?;
        let mut stmt = db.prepare(
            "SELECT id, name, command, working_dir, is_default, created_at
             FROM build_commands
             ORDER BY created_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(BuildCommand {
                id: row.get(0)?,
                name: row.get(1)?,
                command: row.get(2)?,
                working_dir: row.get(3)?,
                is_default: row.get::<_, i64>(4)? != 0,
                created_at: row
                    .get::<_, String>(5)?
                    .parse()
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| ContextError::Database(e.to_string()))
    }

    pub async fn add(
        &self,
        name: &str,
        command: &str,
        working_dir: Option<String>,
    ) -> Result<BuildCommand, ContextError> {
        let db = self.get_db()?;
        let now = Utc::now();
        let id = uuid::Uuid::new_v4().to_string();
        let should_default = self.default_is_missing(&db)?;
        if should_default {
            clear_default(&db)?;
        }

        db.execute(
            "INSERT INTO build_commands (id, name, command, working_dir, is_default, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![id, name, command, working_dir, if should_default { 1 } else { 0 }, now.to_rfc3339()],
        )?;

        Ok(BuildCommand {
            id,
            name: name.to_string(),
            command: command.to_string(),
            working_dir,
            is_default: should_default,
            created_at: now,
        })
    }

    pub async fn remove(&self, id: &str) -> Result<(), ContextError> {
        let db = self.get_db()?;
        let rows = db.execute("DELETE FROM build_commands WHERE id = ?1", [id])?;
        if rows == 0 {
            return Err(ContextError::NotInContext(id.to_string()));
        }
        Ok(())
    }

    pub async fn get(&self, id: &str) -> Result<Option<BuildCommand>, ContextError> {
        let db = self.get_db()?;
        let result = db.query_row(
            "SELECT id, name, command, working_dir, is_default, created_at FROM build_commands WHERE id = ?1",
            [id],
            |row| {
                Ok(BuildCommand {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    command: row.get(2)?,
                    working_dir: row.get(3)?,
                    is_default: row.get::<_, i64>(4)? != 0,
                    created_at: row
                        .get::<_, String>(5)?
                        .parse()
                        .unwrap_or_else(|_| Utc::now()),
                })
            },
        );

        match result {
            Ok(command) => Ok(Some(command)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(ContextError::Database(e.to_string())),
        }
    }

    pub async fn set_default(&self, id: &str) -> Result<(), ContextError> {
        let db = self.get_db()?;
        clear_default(&db)?;
        let rows = db.execute(
            "UPDATE build_commands SET is_default = 1 WHERE id = ?1",
            [id],
        )?;
        if rows == 0 {
            return Err(ContextError::NotInContext(id.to_string()));
        }
        Ok(())
    }

    pub async fn get_default(&self) -> Result<Option<BuildCommand>, ContextError> {
        let db = self.get_db()?;
        let result = db.query_row(
            "SELECT id, name, command, working_dir, is_default, created_at
             FROM build_commands
             WHERE is_default = 1
             ORDER BY created_at DESC
             LIMIT 1",
            [],
            |row| {
                Ok(BuildCommand {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    command: row.get(2)?,
                    working_dir: row.get(3)?,
                    is_default: row.get::<_, i64>(4)? != 0,
                    created_at: row
                        .get::<_, String>(5)?
                        .parse()
                        .unwrap_or_else(|_| Utc::now()),
                })
            },
        );

        match result {
            Ok(command) => Ok(Some(command)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(ContextError::Database(e.to_string())),
        }
    }

    fn default_is_missing(&self, db: &rusqlite::Connection) -> Result<bool, ContextError> {
        let count: i64 = db
            .query_row(
                "SELECT COUNT(*) FROM build_commands WHERE is_default = 1",
                [],
                |row| row.get(0),
            )
            .map_err(|e| ContextError::Database(e.to_string()))?;
        Ok(count == 0)
    }
}

fn clear_default(db: &rusqlite::Connection) -> Result<(), ContextError> {
    db.execute("UPDATE build_commands SET is_default = 0", [])
        .map_err(|e| ContextError::Database(e.to_string()))?;
    Ok(())
}

fn ensure_column(
    db: &rusqlite::Connection,
    table: &str,
    column: &str,
    definition: &str,
) -> Result<(), ContextError> {
    let mut stmt = db
        .prepare(&format!("PRAGMA table_info({})", table))
        .map_err(|e| ContextError::Database(e.to_string()))?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| ContextError::Database(e.to_string()))?;
    let mut existing = Vec::new();
    for row in rows {
        existing.push(row.map_err(|e| ContextError::Database(e.to_string()))?);
    }
    if !existing.iter().any(|name| name == column) {
        db.execute(
            &format!("ALTER TABLE {} ADD COLUMN {} {}", table, column, definition),
            [],
        )
        .map_err(|e| ContextError::Database(e.to_string()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn add_and_list() {
        let temp = tempfile::NamedTempFile::new().unwrap();
        let store = BuildCommandStore::new(temp.path().to_str().unwrap())
            .await
            .unwrap();

        store
            .add("Build", "npm run build:app", None)
            .await
            .unwrap();
        let list = store.list().await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "Build");
    }
}
