use super::{Database, PaginatedQuery, format_now_for_db, parse_datetime, query_row_optional};
use crate::models::{
    ChatHistory, ChatMessage, ChatSession, ChatSessionSummary, CreateChatSessionRequest,
    PaginatedResponse, UpdateChatSessionRequest,
};
use rusqlite::{Result as SqliteResult, Row, params};

/// Map a database row to a ChatSession struct
fn row_to_chat_session(row: &Row) -> SqliteResult<ChatSession> {
    Ok(ChatSession {
        id: row.get(0)?,
        game_id: row.get(1)?,
        title: row.get(2)?,
        include_house_rules: row.get(3)?,
        created_at: parse_datetime(row, "created_at")?,
        updated_at: parse_datetime(row, "updated_at")?,
    })
}

/// Map a database row to a ChatMessage struct
fn row_to_chat_message(row: &Row) -> SqliteResult<ChatMessage> {
    let role_str: String = row.get(2)?;
    let role =
        crate::models::MessageRole::from_str(&role_str).unwrap_or(crate::models::MessageRole::User);

    let context_chunks: Option<String> = row.get(4)?;
    let context_chunks = context_chunks.and_then(|s| serde_json::from_str::<Vec<i64>>(&s).ok());

    Ok(ChatMessage {
        id: row.get(0)?,
        session_id: row.get(1)?,
        role,
        content: row.get(3)?,
        context_chunks,
        created_at: parse_datetime(row, "created_at")?,
    })
}

pub async fn list_chat_sessions(
    db: &Database,
    game_id: i64,
    page: u32,
    limit: u32,
) -> SqliteResult<PaginatedResponse<ChatSessionSummary>> {
    db.with_connection(|conn| {
        let mut q = PaginatedQuery::new();
        q.filter("cs.game_id = ?", game_id);

        q.execute(
            conn,
            "chat_sessions cs",
            "cs.id, cs.game_id, cs.title, cs.include_house_rules, cs.created_at, COUNT(cm.id) as message_count, MAX(cm.created_at) as last_message_at",
            "chat_sessions cs LEFT JOIN chat_messages cm ON cs.id = cm.session_id",
            "COALESCE(MAX(cm.created_at), cs.created_at) DESC",
            Some("cs.id, cs.game_id, cs.title, cs.include_house_rules, cs.created_at"),
            page,
            limit,
            |row| {
                let include_house_rules: bool = row.get(3)?;
                let message_count: i32 = row.get(5)?;
                let last_message_at: Option<String> = row.get(6)?;
                let last_message_at = last_message_at.map(|s| {
                    chrono::DateTime::parse_from_rfc3339(&s)
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                        .or_else(|_| {
                            chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")
                                .map(|dt| dt.and_utc())
                        })
                        .unwrap_or_else(|_| chrono::Utc::now())
                });

                Ok(ChatSessionSummary {
                    id: row.get(0)?,
                    game_id: row.get(1)?,
                    title: row.get(2)?,
                    include_house_rules,
                    message_count,
                    last_message_at,
                    created_at: parse_datetime(row, "created_at")?,
                })
            },
        )
    })
}

pub async fn get_chat_history(db: &Database, session_id: i64) -> SqliteResult<Option<ChatHistory>> {
    db.with_connection(|conn| {
        // First get the session
        let mut session_stmt = conn.prepare(
            "SELECT id, game_id, title, include_house_rules, created_at, updated_at FROM chat_sessions WHERE id = ?",
        )?;

        let session = match query_row_optional(session_stmt.query_row(params![session_id], row_to_chat_session))? {
            Some(session) => session,
            None => return Ok(None),
        };

        // Get messages for the session
        let mut messages_stmt = conn.prepare(
            r#"
            SELECT id, session_id, role, content, context_chunks, created_at
            FROM chat_messages
            WHERE session_id = ?
            ORDER BY created_at ASC
            "#,
        )?;

        let message_iter = messages_stmt.query_map(params![session_id], row_to_chat_message)?;

        let messages: Result<Vec<ChatMessage>, _> = message_iter.collect();
        let messages = messages?;

        Ok(Some(ChatHistory { session, messages }))
    })
}

pub async fn create_chat_session(
    db: &Database,
    request: CreateChatSessionRequest,
) -> SqliteResult<ChatSession> {
    db.with_transaction(|conn| {
        let now_str = format_now_for_db();

        // First verify the game exists
        let game_exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM master_games WHERE id = ?)",
            params![request.game_id],
            |row| row.get(0),
        )?;

        if !game_exists {
            return Err(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CONSTRAINT),
                Some("Game does not exist".to_string()),
            ));
        }

        conn.execute(
            r#"
            INSERT INTO chat_sessions (game_id, title, include_house_rules, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
            params![request.game_id, request.title, request.include_house_rules, now_str, now_str],
        )?;

        let session_id = conn.last_insert_rowid();

        // Fetch the created session
        let mut stmt = conn.prepare(
            "SELECT id, game_id, title, include_house_rules, created_at, updated_at FROM chat_sessions WHERE id = ?",
        )?;

        stmt.query_row(params![session_id], row_to_chat_session)
    })
}

pub async fn add_message_to_session(
    db: &Database,
    session_id: i64,
    role: crate::models::MessageRole,
    content: String,
    context_chunks: Option<Vec<i64>>,
) -> SqliteResult<ChatMessage> {
    db.with_transaction(|conn| {
        let now_str = format_now_for_db();

        let context_chunks_json = context_chunks.map(|chunks| {
            serde_json::to_string(&chunks).unwrap_or_else(|_| "[]".to_string())
        });

        conn.execute(
            r#"
            INSERT INTO chat_messages (session_id, role, content, context_chunks, created_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
            params![session_id, role.as_str(), content, context_chunks_json, now_str]
        )?;

        let message_id = conn.last_insert_rowid();

        // Fetch the created message
        let mut stmt = conn.prepare(
            "SELECT id, session_id, role, content, context_chunks, created_at FROM chat_messages WHERE id = ?"
        )?;

        stmt.query_row(params![message_id], row_to_chat_message)
    })
}

pub async fn update_chat_session(
    db: &Database,
    session_id: i64,
    request: UpdateChatSessionRequest,
) -> SqliteResult<Option<ChatSession>> {
    db.with_transaction(|conn| {
        let now_str = format_now_for_db();

        // Build dynamic update query based on provided fields
        let mut updates = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(title) = &request.title {
            updates.push("title = ?");
            params_vec.push(Box::new(title.clone()));
        }
        if let Some(include_house_rules) = request.include_house_rules {
            updates.push("include_house_rules = ?");
            params_vec.push(Box::new(include_house_rules));
        }

        if updates.is_empty() {
            // Nothing to update, just return the existing session
            let mut stmt = conn.prepare(
                "SELECT id, game_id, title, include_house_rules, created_at, updated_at FROM chat_sessions WHERE id = ?",
            )?;
            return query_row_optional(stmt.query_row(params![session_id], row_to_chat_session));
        }

        updates.push("updated_at = ?");
        params_vec.push(Box::new(now_str));
        params_vec.push(Box::new(session_id));

        let query = format!(
            "UPDATE chat_sessions SET {} WHERE id = ?",
            updates.join(", ")
        );

        let rows_affected = conn.execute(
            &query,
            params_vec
                .iter()
                .map(|p| p.as_ref())
                .collect::<Vec<_>>()
                .as_slice(),
        )?;

        if rows_affected == 0 {
            return Ok(None);
        }

        // Fetch the updated session
        let mut stmt = conn.prepare(
            "SELECT id, game_id, title, include_house_rules, created_at, updated_at FROM chat_sessions WHERE id = ?",
        )?;

        query_row_optional(stmt.query_row(params![session_id], row_to_chat_session))
    })
}
