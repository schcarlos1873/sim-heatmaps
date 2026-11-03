use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};

use domain::{DomainError, DomainResult};
use rusqlite::Connection;

/// Migrações versionadas via `PRAGMA user_version`. Nunca altere uma migração já publicada.
const MIGRATIONS: &[&str] = &[include_str!("../migrations/001_init.sql")];

#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

pub(crate) fn storage(err: rusqlite::Error) -> DomainError {
    DomainError::Storage(err.to_string())
}

impl Database {
    pub fn open(path: &Path) -> DomainResult<Self> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| DomainError::Storage(e.to_string()))?;
        }
        Self::init(Connection::open(path).map_err(storage)?)
    }

    pub fn open_in_memory() -> DomainResult<Self> {
        Self::init(Connection::open_in_memory().map_err(storage)?)
    }

    fn init(conn: Connection) -> DomainResult<Self> {
        conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;").map_err(storage)?;
        let db = Self { conn: Arc::new(Mutex::new(conn)) };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> DomainResult<()> {
        let mut conn = self.conn();
        let version: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0)).map_err(storage)?;
        for (i, sql) in MIGRATIONS.iter().enumerate().skip(version as usize) {
            let tx = conn.transaction().map_err(storage)?;
            tx.execute_batch(sql).map_err(storage)?;
            tx.pragma_update(None, "user_version", (i + 1) as i64).map_err(storage)?;
            tx.commit().map_err(storage)?;
            tracing::info!(version = i + 1, "database migrated");
        }
        Ok(())
    }

    pub(crate) fn conn(&self) -> MutexGuard<'_, Connection> {
        self.conn.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
