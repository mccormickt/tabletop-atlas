use chrono::Utc;
use rusqlite::{Connection, Result as SqliteResult, Row};
use std::sync::{Arc, Mutex, PoisonError};

pub mod admin;
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
    eprintln!(
        "ERROR: Database mutex poisoned - previous thread panicked. Application may be in inconsistent state."
    );
    eprintln!("DEBUG: PoisonError details: {:?}", err);
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

/// Escape LIKE metacharacters (`%`, `_`) in a search term so they match literally.
/// The caller must add `ESCAPE '\'` to the LIKE clause.
pub fn escape_like(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

/// Build a LIKE search pattern from user input, with metacharacters escaped.
/// Returns a pattern like `%escaped_term%` for use with `LIKE ? ESCAPE '\'`.
pub fn like_pattern(term: &str) -> String {
    format!("%{}%", escape_like(&term.to_lowercase()))
}

/// Convert a query result to Option, treating QueryReturnedNoRows as None
pub fn query_row_optional<T>(result: SqliteResult<T>) -> SqliteResult<Option<T>> {
    match result {
        Ok(item) => Ok(Some(item)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Builder for paginated list queries with dynamic WHERE clauses.
///
/// Handles the common pattern of: build conditions -> COUNT query -> SELECT with LIMIT/OFFSET.
#[derive(Default)]
#[allow(dead_code)]
pub struct PaginatedQuery {
    conditions: Vec<String>,
    params: Vec<Box<dyn rusqlite::ToSql>>,
}

#[allow(dead_code)]
impl PaginatedQuery {
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
            params: Vec::new(),
        }
    }

    /// Add a WHERE condition with a bound parameter (e.g., `"role = ?"`, role_value).
    pub fn filter(&mut self, clause: &str, param: impl rusqlite::ToSql + 'static) {
        self.conditions.push(clause.to_string());
        self.params.push(Box::new(param));
    }

    /// Add a LIKE search on a single column with proper wildcard escaping.
    pub fn filter_like(&mut self, column: &str, term: &str) {
        self.conditions
            .push(format!("{} LIKE ? ESCAPE '\\'", column));
        self.params.push(Box::new(like_pattern(term)));
    }

    /// Add a LIKE search across multiple columns (OR). Each column gets its own param.
    pub fn filter_like_any(&mut self, columns: &[&str], term: &str) {
        let pattern = like_pattern(term);
        let parts: Vec<String> = columns
            .iter()
            .map(|col| format!("{} LIKE ? ESCAPE '\\'", col))
            .collect();
        self.conditions.push(format!("({})", parts.join(" OR ")));
        for _ in columns {
            self.params.push(Box::new(pattern.clone()));
        }
    }

    /// Add a raw condition with no bound parameters (e.g., `"rules_pdf_path IS NOT NULL"`).
    pub fn filter_raw(&mut self, clause: &str) {
        self.conditions.push(clause.to_string());
    }

    /// Execute the query: COUNT for total, then SELECT with pagination.
    ///
    /// - `count_from`: table/join for COUNT (e.g., `"master_games g"`)
    /// - `select_columns`: columns for SELECT (e.g., `"g.id, g.name"`)
    /// - `select_from`: table/join for SELECT (may differ from count_from if JOINs add columns)
    /// - `order_by`: ORDER BY clause (e.g., `"g.name ASC"`)
    /// - `group_by`: optional GROUP BY clause (e.g., `"g.id, g.name"`)
    #[allow(clippy::too_many_arguments)]
    pub fn execute<T>(
        &self,
        conn: &Connection,
        count_from: &str,
        select_columns: &str,
        select_from: &str,
        order_by: &str,
        group_by: Option<&str>,
        page: u32,
        limit: u32,
        mapper: impl Fn(&rusqlite::Row) -> SqliteResult<T>,
    ) -> SqliteResult<crate::models::PaginatedResponse<T>> {
        let pagination = PaginationInfo::new(page, limit);

        let where_clause = if self.conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", self.conditions.join(" AND "))
        };

        let group_clause = group_by
            .map(|g| format!("GROUP BY {}", g))
            .unwrap_or_default();

        // Build param refs for the WHERE clause
        let where_refs: Vec<&dyn rusqlite::ToSql> =
            self.params.iter().map(|p| p.as_ref()).collect();

        // COUNT query
        let count_sql = format!("SELECT COUNT(*) FROM {} {}", count_from, where_clause);
        let total: u32 = conn.query_row(&count_sql, where_refs.as_slice(), |row| row.get(0))?;

        // SELECT query with pagination
        let select_sql = format!(
            "SELECT {} FROM {} {} {} ORDER BY {} LIMIT ? OFFSET ?",
            select_columns, select_from, where_clause, group_clause, order_by
        );

        // Build full param list: WHERE params + LIMIT + OFFSET
        let mut all_refs: Vec<&dyn rusqlite::ToSql> = where_refs;
        let limit_val = pagination.limit;
        let offset_val = pagination.offset;
        all_refs.push(&limit_val);
        all_refs.push(&offset_val);

        let mut stmt = conn.prepare(&select_sql)?;
        let items: Vec<T> = stmt
            .query_map(all_refs.as_slice(), |row| mapper(row))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(crate::models::PaginatedResponse::new(
            items, total, page, limit,
        ))
    }
}
