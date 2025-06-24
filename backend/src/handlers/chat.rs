use dropshot::{Path, Query, RequestContext, TypedBody, endpoint};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{created_response, internal_error, not_found_error, success_response};
use crate::{
    AppState,
    db::{Database, chat, embeddings},
    handlers::{HttpCreated, HttpError, HttpOk},
    models::{
        ChatHistory, ChatRequest, ChatResponse, ChatSession, ChatSessionId, ChatSessionSummary,
        CreateChatSessionRequest, GameId, PaginatedResponse, PaginationParams,
        SimilaritySearchRequest,
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

#[derive(Deserialize, JsonSchema)]
pub struct RulesSearchQuery {
    pub game_id: GameId,
    pub query: String,
    pub limit: Option<usize>,
}

#[derive(Serialize, JsonSchema)]
pub struct RulesSearchResponse {
    pub game_id: i64,
    pub query: String,
    pub results: Vec<SearchResult>,
    pub total_results: usize,
}

#[derive(Serialize, JsonSchema)]
pub struct SearchResult {
    pub chunk_id: i64,
    pub chunk_text: String,
    pub chunk_index: i32,
    pub similarity_score: f32,
    pub metadata: String,
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

/// Search rules text for a specific game using embedding similarity
#[endpoint {
    method = GET,
    path = "/api/chat/search-rules"
}]
pub async fn search_rules(
    rqctx: RequestContext<AppState>,
    query: Query<RulesSearchQuery>,
) -> Result<HttpOk<RulesSearchResponse>, HttpError> {
    let app_state = rqctx.context();
    let search_query = query.into_inner();
    let limit = search_query.limit.unwrap_or(5);

    // Generate embedding for the search query using shared service
    let query_embedding = app_state
        .embedding_service()
        .generate_embedding(&search_query.query)
        .await
        .map_err(|e| internal_error(format!("Failed to generate query embedding: {}", e)))?;

    // Use vector similarity search
    let db = Database::new(app_state.db());
    let similarity_request = SimilaritySearchRequest {
        game_id: search_query.game_id,
        query_embedding,
        similarity_threshold: 0.0, // Include all results, let sorting handle ranking
        limit: limit as u32,
    };

    let search_results = embeddings::similarity_search(&db, similarity_request)
        .await
        .map_err(|e| internal_error(format!("Vector similarity search failed: {}", e)))?;

    let results: Vec<SearchResult> = search_results
        .into_iter()
        .map(|result| SearchResult {
            chunk_id: result.id,
            chunk_text: result.chunk_text,
            chunk_index: 0, // We don't have chunk_index in the similarity search result
            similarity_score: result.similarity_score,
            metadata: result.metadata.unwrap_or_default(),
        })
        .collect();

    let response = RulesSearchResponse {
        game_id: search_query.game_id,
        query: search_query.query,
        total_results: results.len(),
        results,
    };

    success_response(response)
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
