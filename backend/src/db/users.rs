use super::{parse_datetime, Database};
use crate::models::{CreateUserRequest, User, UserId};
use chrono::Utc;
use rusqlite::{params, Result as SqliteResult};

pub async fn find_by_google_sub(db: &Database, google_sub: &str) -> SqliteResult<Option<User>> {
    db.with_connection(|conn| {
        let mut stmt = conn.prepare(
            r#"
            SELECT id, google_sub, email, display_name, picture_url, role, created_at, updated_at
            FROM users WHERE google_sub = ?
            "#,
        )?;

        let result = stmt.query_row(params![google_sub], |row| {
            Ok(User {
                id: row.get(0)?,
                google_sub: row.get(1)?,
                email: row.get(2)?,
                display_name: row.get(3)?,
                picture_url: row.get(4)?,
                role: row.get(5)?,
                created_at: parse_datetime(row, "created_at")?,
                updated_at: parse_datetime(row, "updated_at")?,
            })
        });

        match result {
            Ok(user) => Ok(Some(user)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    })
}

pub async fn create_user(db: &Database, request: CreateUserRequest) -> SqliteResult<User> {
    db.with_transaction(|conn| {
        let now = Utc::now();
        let now_str = now.format("%Y-%m-%d %H:%M:%S").to_string();

        conn.execute(
            r#"
            INSERT INTO users (google_sub, email, display_name, picture_url, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            params![
                request.google_sub,
                request.email,
                request.display_name,
                request.picture_url,
                now_str,
                now_str
            ],
        )?;

        let user_id = conn.last_insert_rowid();

        let mut stmt = conn.prepare(
            r#"
            SELECT id, google_sub, email, display_name, picture_url, role, created_at, updated_at
            FROM users WHERE id = ?
            "#,
        )?;

        stmt.query_row(params![user_id], |row| {
            Ok(User {
                id: row.get(0)?,
                google_sub: row.get(1)?,
                email: row.get(2)?,
                display_name: row.get(3)?,
                picture_url: row.get(4)?,
                role: row.get(5)?,
                created_at: parse_datetime(row, "created_at")?,
                updated_at: parse_datetime(row, "updated_at")?,
            })
        })
    })
}

pub async fn get_user_by_id(db: &Database, user_id: UserId) -> SqliteResult<Option<User>> {
    db.with_connection(|conn| {
        let mut stmt = conn.prepare(
            r#"
            SELECT id, google_sub, email, display_name, picture_url, role, created_at, updated_at
            FROM users WHERE id = ?
            "#,
        )?;

        let result = stmt.query_row(params![user_id], |row| {
            Ok(User {
                id: row.get(0)?,
                google_sub: row.get(1)?,
                email: row.get(2)?,
                display_name: row.get(3)?,
                picture_url: row.get(4)?,
                role: row.get(5)?,
                created_at: parse_datetime(row, "created_at")?,
                updated_at: parse_datetime(row, "updated_at")?,
            })
        });

        match result {
            Ok(user) => Ok(Some(user)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    })
}

pub async fn update_user(
    db: &Database,
    user_id: UserId,
    display_name: Option<String>,
    picture_url: Option<String>,
) -> SqliteResult<Option<User>> {
    db.with_transaction(|conn| {
        let exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM users WHERE id = ?)",
            params![user_id],
            |row| row.get(0),
        )?;

        if !exists {
            return Ok(None);
        }

        let now_str = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let mut update_parts = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(name) = display_name {
            update_parts.push("display_name = ?");
            params_vec.push(Box::new(name));
        }
        if let Some(url) = picture_url {
            update_parts.push("picture_url = ?");
            params_vec.push(Box::new(url));
        }

        if !update_parts.is_empty() {
            update_parts.push("updated_at = ?");
            params_vec.push(Box::new(now_str));
            params_vec.push(Box::new(user_id));

            let query = format!("UPDATE users SET {} WHERE id = ?", update_parts.join(", "));
            let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
            conn.execute(&query, params_refs.as_slice())?;
        }

        let mut stmt = conn.prepare(
            r#"
            SELECT id, google_sub, email, display_name, picture_url, role, created_at, updated_at
            FROM users WHERE id = ?
            "#,
        )?;

        let user = stmt.query_row(params![user_id], |row| {
            Ok(User {
                id: row.get(0)?,
                google_sub: row.get(1)?,
                email: row.get(2)?,
                display_name: row.get(3)?,
                picture_url: row.get(4)?,
                role: row.get(5)?,
                created_at: parse_datetime(row, "created_at")?,
                updated_at: parse_datetime(row, "updated_at")?,
            })
        })?;

        Ok(Some(user))
    })
}
