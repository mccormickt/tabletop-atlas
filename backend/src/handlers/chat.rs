use dropshot::{Path, Query, RequestContext, TypedBody, endpoint};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{IdPath, created_response, internal_error, not_found_error, success_response};
use crate::{
    AppState,
    db::chat,
    error::{DbResultExt, OptionExt},
    handlers::{HttpCreated, HttpError, HttpOk},
    models::{
        ChatHistory, ChatRequest, ChatResponse, ChatSession, ChatSessionSummary, ContextSource,
        CreateChatSessionRequest, MessageRole, PaginatedResponse, SimilaritySearchRequest,
        UpdateChatSessionRequest,
    },
};

#[derive(Deserialize, JsonSchema)]
pub struct ChatSessionsByGameQuery {
    pub game_id: String,
    pub page: u32,
    pub limit: u32,
}

#[derive(Deserialize, JsonSchema)]
pub struct RulesSearchQuery {
    pub game_id: String,
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
    let db = app_state.db();

    let game_id: i64 = query
        .game_id
        .parse()
        .map_err(|_| super::bad_request_error("Invalid game_id parameter".to_string()))?;

    match chat::list_chat_sessions(&db, game_id, query.page, query.limit).await {
        Ok(result) => success_response(result),
        Err(e) => {
            slog::error!(rqctx.log, "Failed to list chat sessions"; "error" => %e);
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
    let db = app_state.db();

    match chat::get_chat_history(&db, session_id).await {
        Ok(Some(history)) => success_response(history),
        Ok(None) => Err(not_found_error(format!(
            "Chat session with id {} not found",
            session_id
        ))),
        Err(e) => {
            slog::error!(rqctx.log, "Failed to get chat session"; "session_id" => session_id, "error" => %e);
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
    let db = app_state.db();

    match chat::create_chat_session(&db, create_request).await {
        Ok(session) => created_response(session),
        Err(e) => {
            slog::error!(rqctx.log, "Failed to create chat session"; "error" => %e);
            Err(internal_error("Failed to create chat session".to_string()))
        }
    }
}

/// Update a chat session (e.g., toggle include_house_rules)
#[endpoint {
    method = PATCH,
    path = "/api/chat/sessions/{id}"
}]
pub async fn update_chat_session(
    rqctx: RequestContext<AppState>,
    path: Path<ChatSessionPathParam>,
    body: TypedBody<UpdateChatSessionRequest>,
) -> Result<HttpOk<ChatSession>, HttpError> {
    let app_state = rqctx.context();
    let session_id = path.into_inner().id;
    let update_request = body.into_inner();
    let db = app_state.db();

    match chat::update_chat_session(&db, session_id, update_request).await {
        Ok(Some(session)) => success_response(session),
        Ok(None) => Err(not_found_error(format!(
            "Chat session with id {} not found",
            session_id
        ))),
        Err(e) => {
            slog::error!(rqctx.log, "Failed to update chat session"; "session_id" => session_id, "error" => %e);
            Err(internal_error("Failed to update chat session".to_string()))
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
    let db = app_state.db();

    // Parse game_id from string
    let game_id: i64 = search_query
        .game_id
        .parse()
        .map_err(|_| super::bad_request_error("Invalid game_id parameter".to_string()))?;

    // Preprocess and enhance the search query for better embedding matching
    let enhanced_query = enhance_search_query(&search_query.query);

    // Generate embedding for the enhanced search query
    let query_embedding = app_state
        .embedder()
        .generate_embedding(&enhanced_query)
        .await
        .map_err(|e| internal_error(format!("Failed to generate query embedding: {}", e)))?;

    let similarity_request = SimilaritySearchRequest {
        game_id,
        query_embedding,
        similarity_threshold: 0.0, // Include all results, let sorting handle ranking
        limit: limit as u32,
    };

    let search_results = crate::db::embeddings::similarity_search(&db, similarity_request)
        .await
        .map_err(|e| internal_error(format!("Search failed: {}", e)))?;

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
        game_id,
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
    let app_state = rqctx.context();
    let chat_request = body.into_inner();
    let db = app_state.db();

    // 1. Get the chat session to verify it exists and get the game_id
    let session_history = chat::get_chat_history(&db, chat_request.session_id)
        .await
        .map_err(|e| {
            slog::error!(rqctx.log, "Failed to get chat session";
                "session_id" => chat_request.session_id, "error" => %e);
            internal_error("Failed to access chat session".to_string())
        })?
        .ok_or_else(|| {
            not_found_error(format!(
                "Chat session with id {} not found",
                chat_request.session_id
            ))
        })?;

    let game_id = session_history.session.game_id;
    let include_house_rules = session_history.session.include_house_rules;

    // 2. Save user message to database
    let user_message = chat::add_message_to_session(
        &db,
        chat_request.session_id,
        crate::models::MessageRole::User,
        chat_request.message.clone(),
        None,
    )
    .await
    .map_err(|e| {
        slog::error!(rqctx.log, "Failed to save user message"; "error" => %e);
        internal_error("Failed to save message".to_string())
    })?;

    // 3. Build GameRulesIndex for dynamic_context retrieval
    let index = crate::agents::GameRulesIndex::new(
        app_state.embedder().model_arc(),
        db.clone(),
        game_id,
        include_house_rules,
    );

    // 4. Build chat history, truncated to last 6 messages
    let messages = &session_history.messages;
    let truncated = if messages.len() > 6 {
        &messages[messages.len() - 6..]
    } else {
        messages
    };
    let chat_history: Vec<(MessageRole, String)> = truncated
        .iter()
        .map(|msg| (msg.role.clone(), msg.content.clone()))
        .collect();

    // 5. Agent handles retrieval + response via dynamic_context
    let assistant_response = app_state
        .rules_agent()
        .answer(&chat_request.message, index, &chat_history)
        .await
        .map_err(|e| {
            slog::error!(rqctx.log, "Failed to generate LLM response"; "error" => %e);
            internal_error("Failed to generate response".to_string())
        })?;

    // 6. Separate search for context_sources (API response metadata)
    let query_embedding = app_state
        .embedder()
        .generate_embedding(&chat_request.message)
        .await
        .map_err(|e| {
            slog::error!(rqctx.log, "Failed to generate query embedding for context"; "error" => %e);
            internal_error("Failed to generate context sources".to_string())
        })?;

    let similarity_request = SimilaritySearchRequest {
        game_id,
        query_embedding,
        similarity_threshold: 0.0,
        limit: 10,
    };

    let search_results = crate::db::embeddings::similarity_search_filtered(
        &db,
        similarity_request,
        include_house_rules,
    )
    .await
    .map_err(|e| {
        slog::error!(rqctx.log, "Failed to search embeddings for context"; "error" => %e);
        internal_error("Failed to search rules".to_string())
    })?;

    let context_sources: Vec<ContextSource> = search_results
        .iter()
        .map(|result| ContextSource {
            embedding_id: result.id,
            chunk_text: result.chunk_text.clone(),
            source_type: result.source_type.as_str().to_string(),
            similarity_score: result.similarity_score,
            metadata: result.metadata.clone(),
        })
        .collect();

    // 7. Save assistant response to database
    let context_chunk_ids: Vec<i64> = search_results.iter().map(|r| r.id).collect();
    let assistant_message = chat::add_message_to_session(
        &db,
        chat_request.session_id,
        MessageRole::Assistant,
        assistant_response,
        Some(context_chunk_ids),
    )
    .await
    .map_err(|e| {
        slog::error!(rqctx.log, "Failed to save assistant message"; "error" => %e);
        internal_error("Failed to save response".to_string())
    })?;

    // 8. Return response with both messages and context sources
    let chat_response = ChatResponse {
        user_message,
        assistant_message,
        context_sources,
    };

    success_response(chat_response)
}

/// Enhance search queries to better match rule document content
fn enhance_search_query(query: &str) -> String {
    let query_lower = query.to_lowercase();
    let mut enhanced_parts = Vec::new();

    // Convert questions to statement form for better embedding matching
    if query_lower.starts_with("how do i") || query_lower.starts_with("how to") {
        let without_prefix = query_lower
            .strip_prefix("how do i ")
            .or_else(|| query_lower.strip_prefix("how to "))
            .unwrap_or(&query_lower);
        enhanced_parts.push(without_prefix.to_string());
        enhanced_parts.push(format!("rules for {}", without_prefix));
        enhanced_parts.push(format!("instructions {}", without_prefix));
    } else if query_lower.starts_with("what") {
        if query_lower.contains("happens") {
            enhanced_parts.push(query_lower.replace("what happens", "when"));
            enhanced_parts.push(query_lower.replace("what happens", "rules"));
        } else if query_lower.contains("can i") || query_lower.contains("may i") {
            enhanced_parts.push(query_lower.replace("what can i", "player may"));
            enhanced_parts.push(query_lower.replace("what may i", "player may"));
            enhanced_parts.push("allowed actions".to_string());
        } else {
            enhanced_parts.push(query_lower.clone());
        }
    } else if query_lower.starts_with("when") {
        enhanced_parts.push(query_lower.clone());
        enhanced_parts.push(query_lower.replace("when", "if"));
        enhanced_parts.push("timing rules".to_string());
    } else if query_lower.starts_with("can i") || query_lower.starts_with("may i") {
        let without_prefix = query_lower
            .strip_prefix("can i ")
            .or_else(|| query_lower.strip_prefix("may i "))
            .unwrap_or(&query_lower);
        enhanced_parts.push(format!("player may {}", without_prefix));
        enhanced_parts.push(format!("allowed to {}", without_prefix));
        enhanced_parts.push(without_prefix.to_string());
    } else {
        enhanced_parts.push(query_lower.clone());
    }

    // Add domain-specific game terms
    let game_terms = extract_game_terms(&query_lower);
    enhanced_parts.extend(game_terms);

    // Join with the original query for comprehensive matching
    let mut final_query = query.to_string();
    if !enhanced_parts.is_empty() {
        final_query.push(' ');
        final_query.push_str(&enhanced_parts.join(" "));
    }

    final_query
}

/// Extract and enhance game-specific terms from the query
fn extract_game_terms(query: &str) -> Vec<String> {
    let mut terms = Vec::new();

    // Common game concepts and their rule document equivalents
    let concept_mappings = [
        ("win", vec!["victory", "winning condition", "game end"]),
        ("lose", vec!["defeat", "elimination", "losing condition"]),
        ("turn", vec!["round", "phase", "player turn"]),
        ("move", vec!["movement", "moving pieces", "relocate"]),
        ("attack", vec!["combat", "battle", "fight"]),
        ("defend", vec!["defense", "block", "protection"]),
        ("points", vec!["score", "scoring", "victory points"]),
        ("cards", vec!["hand", "deck", "draw"]),
        ("dice", vec!["roll", "rolling", "die"]),
        ("setup", vec!["preparation", "initial setup", "game setup"]),
        ("end", vec!["finish", "conclusion", "game over"]),
    ];

    for (concept, equivalents) in &concept_mappings {
        if query.contains(concept) {
            terms.extend(equivalents.iter().map(|s| s.to_string()));
        }
    }

    terms
}
