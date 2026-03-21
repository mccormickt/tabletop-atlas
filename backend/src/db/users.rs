use super::{
    Database, PaginationInfo, format_now_for_db, like_pattern, parse_datetime, query_row_optional,
};
use crate::models::{CreateUserRequest, PaginatedResponse, User, UserId, UserListItem};
use rusqlite::{Result as SqliteResult, Row, params};

/// Map a database row to a User struct
fn row_to_user(row: &Row) -> SqliteResult<User> {
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
}

pub async fn find_by_google_sub(db: &Database, google_sub: &str) -> SqliteResult<Option<User>> {
    db.with_connection(|conn| {
        let mut stmt = conn.prepare(
            r#"
            SELECT id, google_sub, email, display_name, picture_url, role, created_at, updated_at
            FROM users WHERE google_sub = ?
            "#,
        )?;

        query_row_optional(stmt.query_row(params![google_sub], row_to_user))
    })
}

pub async fn create_user(db: &Database, request: CreateUserRequest) -> SqliteResult<User> {
    db.with_transaction(|conn| {
        let now_str = format_now_for_db();

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

        stmt.query_row(params![user_id], row_to_user)
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

        query_row_optional(stmt.query_row(params![user_id], row_to_user))
    })
}

fn row_to_user_list_item(row: &Row) -> SqliteResult<UserListItem> {
    Ok(UserListItem {
        id: row.get(0)?,
        email: row.get(1)?,
        display_name: row.get(2)?,
        role: row.get(3)?,
        created_at: parse_datetime(row, "created_at")?,
    })
}

pub async fn list_users(
    db: &Database,
    page: u32,
    limit: u32,
    search: Option<&str>,
    role: Option<&str>,
) -> SqliteResult<PaginatedResponse<UserListItem>> {
    let pagination = PaginationInfo::new(page, limit);

    db.with_connection(|conn| {
        let search_pattern = search.map(like_pattern);
        let mut where_conditions: Vec<String> = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if search_pattern.is_some() {
            where_conditions.push(
                "(LOWER(email) LIKE ? ESCAPE '\\' OR LOWER(display_name) LIKE ? ESCAPE '\\')"
                    .to_string(),
            );
        }

        if role.is_some() {
            where_conditions.push("role = ?".to_string());
        }

        let where_clause = if where_conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_conditions.join(" AND "))
        };

        // Build params for count query
        if let Some(ref pattern) = search_pattern {
            params_vec.push(Box::new(pattern.clone()));
            params_vec.push(Box::new(pattern.clone()));
        }
        if let Some(r) = role {
            params_vec.push(Box::new(r.to_string()));
        }

        let count_query = format!("SELECT COUNT(*) FROM users {}", where_clause);
        let count_refs: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|p| p.as_ref()).collect();
        let total: u32 = conn.query_row(&count_query, count_refs.as_slice(), |row| row.get(0))?;

        // Add pagination params
        params_vec.push(Box::new(pagination.limit));
        params_vec.push(Box::new(pagination.offset));

        let query = format!(
            "SELECT id, email, display_name, role, created_at FROM users {} ORDER BY created_at DESC LIMIT ? OFFSET ?",
            where_clause
        );
        let all_refs: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|p| p.as_ref()).collect();

        let mut stmt = conn.prepare(&query)?;
        let items: Vec<UserListItem> = stmt
            .query_map(all_refs.as_slice(), row_to_user_list_item)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(PaginatedResponse::new(items, total, page, limit))
    })
}

/// Update a user's role. If `check_last_admin` is true and the update would
/// remove the last admin, returns a custom error instead of proceeding.
pub async fn update_user_role(
    db: &Database,
    user_id: UserId,
    new_role: &str,
    check_last_admin: bool,
) -> SqliteResult<Option<UserListItem>> {
    db.with_transaction(|conn| {
        let exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM users WHERE id = ?)",
            params![user_id],
            |row| row.get(0),
        )?;

        if !exists {
            return Ok(None);
        }

        // Atomically check last-admin constraint inside the transaction
        if check_last_admin && new_role == "user" {
            let admin_count: u32 = conn.query_row(
                "SELECT COUNT(*) FROM users WHERE role = 'admin'",
                [],
                |row| row.get(0),
            )?;
            if admin_count <= 1 {
                return Err(rusqlite::Error::SqliteFailure(
                    rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CONSTRAINT),
                    Some("Cannot demote the last remaining admin".to_string()),
                ));
            }
        }

        let now_str = format_now_for_db();
        conn.execute(
            "UPDATE users SET role = ?, updated_at = ? WHERE id = ?",
            params![new_role, now_str, user_id],
        )?;

        let mut stmt = conn
            .prepare("SELECT id, email, display_name, role, created_at FROM users WHERE id = ?")?;
        Ok(Some(
            stmt.query_row(params![user_id], row_to_user_list_item)?,
        ))
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

        let now_str = format_now_for_db();

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
            let params_refs: Vec<&dyn rusqlite::ToSql> =
                params_vec.iter().map(|p| p.as_ref()).collect();
            conn.execute(&query, params_refs.as_slice())?;
        }

        let mut stmt = conn.prepare(
            r#"
            SELECT id, google_sub, email, display_name, picture_url, role, created_at, updated_at
            FROM users WHERE id = ?
            "#,
        )?;

        Ok(Some(stmt.query_row(params![user_id], row_to_user)?))
    })
}
