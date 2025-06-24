use rusqlite::{params, Result as SqliteResult};
use chrono::Utc;
use crate::models::{
    ChatSession, ChatSessionId, ChatMessage, GameId, 
    CreateChatSessionRequest, ChatHistory, ChatSessionSummary, PaginatedResponse
};
use super::{Database, parse_datetime, PaginationInfo};

pub async fn list_chat_sessions(db: &Database, game_id: GameId, page: u32, limit: u32) -> SqliteResult<PaginatedResponse<ChatSessionSummary>> {
    let pagination = PaginationInfo::new(page, limit);
    
    db.with_connection(|conn| {
        // Get total count for the specific game
        let total: u32 = conn.query_row(
            "SELECT COUNT(*) FROM chat_sessions WHERE game_id = ?",
            params![game_id],
            |row| row.get(0)
        )?;

        // Get chat sessions with message counts and last message times
        let mut stmt = conn.prepare(
            r#"
            SELECT 
                cs.id, cs.game_id, cs.title, cs.created_at,
                COUNT(cm.id) as message_count,
                MAX(cm.created_at) as last_message_at
            FROM chat_sessions cs
            LEFT JOIN chat_messages cm ON cs.id = cm.session_id
            WHERE cs.game_id = ?
            GROUP BY cs.id, cs.game_id, cs.title, cs.created_at
            ORDER BY COALESCE(MAX(cm.created_at), cs.created_at) DESC
            LIMIT ? OFFSET ?
            "#
        )?;

        let session_iter = stmt.query_map(params![game_id, pagination.limit, pagination.offset], |row| {
            let message_count: i32 = row.get(4)?;
            let last_message_at: Option<String> = row.get(5)?;
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
                message_count,
                last_message_at,
                created_at: parse_datetime(row, "created_at")?,
            })
        })?;

        let sessions: Result<Vec<ChatSessionSummary>, _> = session_iter.collect();
        let sessions = sessions?;

        Ok(PaginatedResponse::new(sessions, total, page, limit))
    })
}

pub async fn get_chat_history(db: &Database, session_id: ChatSessionId) -> SqliteResult<Option<ChatHistory>> {
    db.with_connection(|conn| {
        // First get the session
        let mut session_stmt = conn.prepare(
            "SELECT id, game_id, title, created_at, updated_at FROM chat_sessions WHERE id = ?"
        )?;

        let session_result = session_stmt.query_row(params![session_id], |row| {
            Ok(ChatSession {
                id: row.get(0)?,
                game_id: row.get(1)?,
                title: row.get(2)?,
                created_at: parse_datetime(row, "created_at")?,
                updated_at: parse_datetime(row, "updated_at")?,
            })
        });

        let session = match session_result {
            Ok(session) => session,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
            Err(e) => return Err(e),
        };

        // Get messages for the session
        let mut messages_stmt = conn.prepare(
            r#"
            SELECT id, session_id, role, content, context_chunks, created_at
            FROM chat_messages 
            WHERE session_id = ?
            ORDER BY created_at ASC
            "#
        )?;

        let message_iter = messages_stmt.query_map(params![session_id], |row| {
            let role_str: String = row.get(2)?;
            let role = crate::models::MessageRole::from_str(&role_str)
                .unwrap_or(crate::models::MessageRole::User);
            
            let context_chunks: Option<String> = row.get(4)?;
            let context_chunks = context_chunks.and_then(|s| {
                serde_json::from_str::<Vec<i64>>(&s).ok()
            });

            Ok(ChatMessage {
                id: row.get(0)?,
                session_id: row.get(1)?,
                role,
                content: row.get(3)?,
                context_chunks,
                created_at: parse_datetime(row, "created_at")?,
            })
        })?;

        let messages: Result<Vec<ChatMessage>, _> = message_iter.collect();
        let messages = messages?;

        Ok(Some(ChatHistory { session, messages }))
    })
}

pub async fn create_chat_session(db: &Database, request: CreateChatSessionRequest) -> SqliteResult<ChatSession> {
    db.with_transaction(|conn| {
        let now = Utc::now();
        let now_str = now.format("%Y-%m-%d %H:%M:%S").to_string();

        // First verify the game exists
        let game_exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM games WHERE id = ?)",
            params![request.game_id],
            |row| row.get(0)
        )?;

        if !game_exists {
            return Err(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CONSTRAINT),
                Some("Game does not exist".to_string())
            ));
        }

        conn.execute(
            r#"
            INSERT INTO chat_sessions (game_id, title, created_at, updated_at)
            VALUES (?, ?, ?, ?)
            "#,
            params![request.game_id, request.title, now_str, now_str]
        )?;

        let session_id = conn.last_insert_rowid();

        // Fetch the created session
        let mut stmt = conn.prepare(
            "SELECT id, game_id, title, created_at, updated_at FROM chat_sessions WHERE id = ?"
        )?;

        stmt.query_row(params![session_id], |row| {
            Ok(ChatSession {
                id: row.get(0)?,
                game_id: row.get(1)?,
                title: row.get(2)?,
                created_at: parse_datetime(row, "created_at")?,
                updated_at: parse_datetime(row, "updated_at")?,
            })
        })
    })
}

pub async fn add_message_to_session(
    db: &Database, 
    session_id: ChatSessionId, 
    role: crate::models::MessageRole, 
    content: String,
    context_chunks: Option<Vec<i64>>
) -> SqliteResult<ChatMessage> {
    db.with_transaction(|conn| {
        let now = Utc::now();
        let now_str = now.format("%Y-%m-%d %H:%M:%S").to_string();

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

        stmt.query_row(params![message_id], |row| {
            let role_str: String = row.get(2)?;
            let role = crate::models::MessageRole::from_str(&role_str)
                .unwrap_or(crate::models::MessageRole::User);
            
            let context_chunks: Option<String> = row.get(4)?;
            let context_chunks = context_chunks.and_then(|s| {
                serde_json::from_str::<Vec<i64>>(&s).ok()
            });

            Ok(ChatMessage {
                id: row.get(0)?,
                session_id: row.get(1)?,
                role,
                content: row.get(3)?,
                context_chunks,
                created_at: parse_datetime(row, "created_at")?,
            })
        })
    })
}

pub async fn delete_chat_session(db: &Database, session_id: ChatSessionId) -> SqliteResult<bool> {
    db.with_connection(|conn| {
        let rows_affected = conn.execute(
            "DELETE FROM chat_sessions WHERE id = ?",
            params![session_id]
        )?;
        Ok(rows_affected > 0)
    })
}

pub async fn get_session_messages(db: &Database, session_id: ChatSessionId, limit: Option<u32>) -> SqliteResult<Vec<ChatMessage>> {
    db.with_connection(|conn| {
        let query = if let Some(limit) = limit {
            format!(
                r#"
                SELECT id, session_id, role, content, context_chunks, created_at
                FROM chat_messages 
                WHERE session_id = ?
                ORDER BY created_at DESC
                LIMIT {}
                "#,
                limit
            )
        } else {
            r#"
            SELECT id, session_id, role, content, context_chunks, created_at
            FROM chat_messages 
            WHERE session_id = ?
            ORDER BY created_at ASC
            "#.to_string()
        };

        let mut stmt = conn.prepare(&query)?;

        let message_iter = stmt.query_map(params![session_id], |row| {
            let role_str: String = row.get(2)?;
            let role = crate::models::MessageRole::from_str(&role_str)
                .unwrap_or(crate::models::MessageRole::User);
            
            let context_chunks: Option<String> = row.get(4)?;
            let context_chunks = context_chunks.and_then(|s| {
                serde_json::from_str::<Vec<i64>>(&s).ok()
            });

            Ok(ChatMessage {
                id: row.get(0)?,
                session_id: row.get(1)?,
                role,
                content: row.get(3)?,
                context_chunks,
                created_at: parse_datetime(row, "created_at")?,
            })
        })?;

        let messages: Result<Vec<ChatMessage>, _> = message_iter.collect();
        messages
    })
}