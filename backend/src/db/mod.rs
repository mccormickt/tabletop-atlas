use rusqlite::{Connection, Result as SqliteResult, Row};
use std::sync::{Arc, Mutex};

pub mod chat;
pub mod embeddings;
pub mod games;
pub mod house_rules;

// Re-exports are available but not used globally to avoid namespace pollution

/// Database connection wrapper with utility methods
#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new(conn: Connection) -> Self {
        Self {
            conn: Arc::new(Mutex::new(conn)),
        }
    }

    pub fn with_connection<F, R>(&self, f: F) -> SqliteResult<R>
    where
        F: FnOnce(&Connection) -> SqliteResult<R>,
    {
        let conn = self.conn.lock().unwrap();
        f(&*conn)
    }

    pub fn with_transaction<F, R>(&self, f: F) -> SqliteResult<R>
    where
        F: FnOnce(&Connection) -> SqliteResult<R>,
    {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        let result = f(&tx)?;
        tx.commit()?;
        Ok(result)
    }
}

/// Helper function to parse datetime from SQLite
pub fn parse_datetime(row: &Row, column: &str) -> SqliteResult<chrono::DateTime<chrono::Utc>> {
    let datetime_str: String = row.get(column)?;
    chrono::DateTime::parse_from_rfc3339(&datetime_str)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .or_else(|_| {
            // Try parsing as SQLite CURRENT_TIMESTAMP format
            chrono::NaiveDateTime::parse_from_str(&datetime_str, "%Y-%m-%d %H:%M:%S")
                .map(|dt| dt.and_utc())
        })
        .map_err(|_| {
            rusqlite::Error::InvalidColumnType(
                row.as_ref().column_index(column).unwrap_or(0),
                column.to_string(),
                rusqlite::types::Type::Text,
            )
        })
}

/// Helper function to format datetime for SQLite
pub fn format_datetime(dt: chrono::DateTime<chrono::Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Common pagination helper
pub struct PaginationInfo {
    pub offset: i64,
    pub limit: i64,
}

impl PaginationInfo {
    pub fn new(page: u32, limit: u32) -> Self {
        let page = page.max(1); // Ensure page is at least 1
        let offset = (page - 1) as i64 * limit as i64;
        Self {
            offset,
            limit: limit as i64,
        }
    }
}
