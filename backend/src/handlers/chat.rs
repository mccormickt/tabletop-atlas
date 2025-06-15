use dropshot::{Path, Query, RequestContext, TypedBody, endpoint};
use schemars::JsonSchema;
use serde::Deserialize;

use super::success_response;
use crate::{
    AppState,
    db::{Database, chat},
    handlers::{HttpCreated, HttpError, HttpOk, created_response, internal_error, not_found_error},
    models::{
        ChatHistory, ChatRequest, ChatResponse, ChatSession, ChatSessionId, ChatSessionSummary,
        CreateChatSessionRequest, GameId, PaginatedResponse, PaginationParams,
    },
};

#[derive(Deserialize, JsonSchema)]
pub struct ChatSessionPathParam {
    pub id: ChatSessionId,
}

#[derive(Deserialize, JsonSchema)]
pub struct ChatSessionsByGameQuery {
    pub game_id: GameId,
    #[serde(flatten)]
    pub pagination: PaginationParams,
}

/// List chat sessions for a specific game
#[endpoint {
    method = GET,
    path = "/api/chat/sessions"
}]
pub async fn list_chat_sessions(
    rqctx: RequestContext<AppState>,
    query: Query<ChatSessionsByGameQuery>,
) -> Result<HttpOk<PaginatedResponse<ChatSessionSummary>>, HttpError> {
    let app_state = rqctx.context();
    let query = query.into_inner();
    let db = Database::new(app_state.db());

    match chat::list_chat_sessions(
        &db,
        query.game_id,
        query.pagination.page,
        query.pagination.limit,
    )
    .await
    {
        Ok(result) => success_response(result),
        Err(e) => {
            tracing::error!("Failed to list chat sessions: {}", e);
            Err(internal_error("Failed to list chat sessions".to_string()))
        }
    }
}

/// Get a specific chat session with its message history
#[endpoint {
    method = GET,
    path = "/api/chat/sessions/{id}"
}]
pub async fn get_chat_session(
    rqctx: RequestContext<AppState>,
    path: Path<ChatSessionPathParam>,
) -> Result<HttpOk<ChatHistory>, HttpError> {
    let app_state = rqctx.context();
    let session_id = path.into_inner().id;
    let db = Database::new(app_state.db());

    match chat::get_chat_history(&db, session_id).await {
        Ok(Some(history)) => success_response(history),
        Ok(None) => Err(not_found_error(format!(
            "Chat session with id {} not found",
            session_id
        ))),
        Err(e) => {
            tracing::error!("Failed to get chat session {}: {}", session_id, e);
            Err(internal_error("Failed to get chat session".to_string()))
        }
    }
}

/// Create a new chat session
#[endpoint {
    method = POST,
    path = "/api/chat/sessions"
}]
pub async fn create_chat_session(
    rqctx: RequestContext<AppState>,
    body: TypedBody<CreateChatSessionRequest>,
) -> Result<HttpCreated<ChatSession>, HttpError> {
    let app_state = rqctx.context();
    let create_request = body.into_inner();
    let db = Database::new(app_state.db());

    match chat::create_chat_session(&db, create_request).await {
        Ok(session) => created_response(session),
        Err(e) => {
            tracing::error!("Failed to create chat session: {}", e);
            Err(internal_error("Failed to create chat session".to_string()))
        }
    }
}

/// Send a message and get AI response
#[endpoint {
    method = POST,
    path = "/api/chat/message"
}]
pub async fn chat_with_rules(
    rqctx: RequestContext<AppState>,
    body: TypedBody<ChatRequest>,
) -> Result<HttpOk<ChatResponse>, HttpError> {
    let _app_state = rqctx.context();
    let _chat_request = body.into_inner();

    // TODO: Implement chat functionality
    // 1. Save user message to database
    // 2. Generate embedding for user's question
    // 3. Search for relevant rule chunks using similarity search
    // 4. Prepare context with relevant rules and house rules
    // 5. Send to LLM API with context
    // 6. Save assistant response to database
    // 7. Return response with context sources

    Err(internal_error(
        "Chat functionality not yet implemented".to_string(),
    ))
}
