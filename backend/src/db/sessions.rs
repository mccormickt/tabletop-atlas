use super::{parse_datetime, Database};
use crate::models::{Session, SessionId, UserId};
use chrono::{DateTime, Utc};
use rusqlite::{params, Result as SqliteResult};
use sha2::{Digest, Sha256};

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

/// Generate a new session ID
pub fn generate_session_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub async fn create_session(
    db: &Database,
    session_id: &str,
    user_id: UserId,
    refresh_token: &str,
    expires_at: DateTime<Utc>,
) -> SqliteResult<Session> {
    db.with_transaction(|conn| {
        let token_hash = hash_token(refresh_token);
        let now = Utc::now();
        let now_str = now.format("%Y-%m-%d %H:%M:%S").to_string();
        let expires_str = expires_at.format("%Y-%m-%d %H:%M:%S").to_string();

        conn.execute(
            r#"
            INSERT INTO sessions (id, user_id, refresh_token_hash, expires_at, created_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
            params![session_id, user_id, token_hash, expires_str, now_str],
        )?;

        Ok(Session {
            id: session_id.to_string(),
            user_id,
            refresh_token_hash: token_hash,
            expires_at,
            created_at: now,
        })
    })
}

pub async fn find_valid_session(
    db: &Database,
    session_id: &SessionId,
    refresh_token: &str,
) -> SqliteResult<Option<Session>> {
    db.with_connection(|conn| {
        let token_hash = hash_token(refresh_token);
        let now_str = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let mut stmt = conn.prepare(
            r#"
            SELECT id, user_id, refresh_token_hash, expires_at, created_at
            FROM sessions
            WHERE id = ? AND refresh_token_hash = ? AND expires_at > ?
            "#,
        )?;

        let result = stmt.query_row(params![session_id, token_hash, now_str], |row| {
            Ok(Session {
                id: row.get(0)?,
                user_id: row.get(1)?,
                refresh_token_hash: row.get(2)?,
                expires_at: parse_datetime(row, "expires_at")?,
                created_at: parse_datetime(row, "created_at")?,
            })
        });

        match result {
            Ok(session) => Ok(Some(session)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    })
}

#[allow(dead_code)]
pub async fn delete_session(db: &Database, session_id: &SessionId) -> SqliteResult<bool> {
    db.with_connection(|conn| {
        let rows_affected = conn.execute("DELETE FROM sessions WHERE id = ?", params![session_id])?;
        Ok(rows_affected > 0)
    })
}

pub async fn delete_user_sessions(db: &Database, user_id: UserId) -> SqliteResult<u64> {
    db.with_connection(|conn| {
        let rows_affected =
            conn.execute("DELETE FROM sessions WHERE user_id = ?", params![user_id])?;
        Ok(rows_affected as u64)
    })
}

#[allow(dead_code)]
pub async fn cleanup_expired_sessions(db: &Database) -> SqliteResult<u64> {
    db.with_connection(|conn| {
        let now_str = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let rows_affected =
            conn.execute("DELETE FROM sessions WHERE expires_at <= ?", params![now_str])?;
        Ok(rows_affected as u64)
    })
}
