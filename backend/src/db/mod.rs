use chrono::Utc;
use rusqlite::{Connection, Result as SqliteResult, Row};
use std::sync::{Arc, Mutex, PoisonError};

pub mod challenges;
pub mod chat;
pub mod collections;
pub mod custom_games;
pub mod embeddings;
pub mod games;
pub mod house_rules;
pub mod sessions;
pub mod users;

// Re-exports are available but not used globally to avoid namespace pollution

/// Helper function to handle mutex poisoning errors with proper logging
fn handle_mutex_poison<T>(err: PoisonError<T>) -> rusqlite::Error {
    // This is a critical error - a thread panicked while holding the database lock
    tracing::error!(
        "Database mutex poisoned - previous thread panicked. Application may be in inconsistent state."
    );
    tracing::debug!("PoisonError details: {:?}", err);
    rusqlite::Error::SqliteFailure(
        rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_LOCKED),
        Some("Database connection mutex poisoned".to_string()),
    )
}

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
        let conn = self.conn.lock().map_err(handle_mutex_poison)?;
        f(&conn)
    }

    pub fn with_transaction<F, R>(&self, f: F) -> SqliteResult<R>
    where
        F: FnOnce(&Connection) -> SqliteResult<R>,
    {
        let mut conn = self.conn.lock().map_err(handle_mutex_poison)?;
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

/// Format the current UTC time for SQLite storage
/// Returns a string in "%Y-%m-%d %H:%M:%S" format
pub fn format_now_for_db() -> String {
    Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Format a specific DateTime for SQLite storage
pub fn format_datetime_for_db(dt: chrono::DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Convert a query result to Option, treating QueryReturnedNoRows as None
pub fn query_row_optional<T>(result: SqliteResult<T>) -> SqliteResult<Option<T>> {
    match result {
        Ok(item) => Ok(Some(item)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}
