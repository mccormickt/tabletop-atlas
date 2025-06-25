use dropshot::{Path, Query, RequestContext, TypedBody, endpoint};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{created_response, internal_error, not_found_error, success_response};
use crate::{
    AppState,
    db::{Database, chat},
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
    let db = app_state.db();

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
    let db = app_state.db();

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
    let db = app_state.db();

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
    let db = app_state.db();

    // Preprocess and enhance the search query for better embedding matching
    let enhanced_query = enhance_search_query(&search_query.query);

    // Generate embedding for the enhanced search query
    let query_embedding = app_state
        .embedder()
        .generate_embedding(&enhanced_query)
        .await
        .map_err(|e| internal_error(format!("Failed to generate query embedding: {}", e)))?;

    // Search using database layer directly
    let similarity_request = SimilaritySearchRequest {
        game_id: search_query.game_id,
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

/// Enhance search results by grouping related chunks and providing better context
fn enhance_search_results(
    mut results: Vec<crate::models::EmbeddingSearchResult>,
    limit: usize,
) -> Vec<crate::models::EmbeddingSearchResult> {
    // Sort by similarity score (highest first)
    results.sort_by(|a, b| {
        b.similarity_score
            .partial_cmp(&a.similarity_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Deduplicate very similar chunks (same source, similar content)
    let mut deduplicated = Vec::new();
    for result in results {
        let is_duplicate =
            deduplicated
                .iter()
                .any(|existing: &crate::models::EmbeddingSearchResult| {
                    // Check if chunks are very similar (same source type and high text overlap)
                    existing.source_type == result.source_type
                        && text_similarity(&existing.chunk_text, &result.chunk_text) > 0.8
                });

        if !is_duplicate {
            deduplicated.push(result);
        }

        if deduplicated.len() >= limit {
            break;
        }
    }

    deduplicated
}

/// Create a context preview that shows the key information from a chunk
fn create_context_preview(chunk_text: &str) -> String {
    let sentences: Vec<&str> = chunk_text
        .split(|c| ".!?".contains(c))
        .map(|s| s.trim())
        .filter(|s| !s.is_empty() && s.len() > 10)
        .take(2) // Take first 2 complete sentences
        .collect();

    if sentences.is_empty() {
        // Fallback to first 150 characters
        if chunk_text.len() > 150 {
            format!("{}...", &chunk_text[..147])
        } else {
            chunk_text.to_string()
        }
    } else {
        let preview = sentences.join(". ");
        if preview.len() > 200 {
            format!("{}...", &preview[..197])
        } else {
            format!("{}.", preview)
        }
    }
}

/// Calculate text similarity between two chunks (simple word overlap)
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
        final_query.push_str(" ");
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

fn text_similarity(text1: &str, text2: &str) -> f32 {
    let words1: std::collections::HashSet<&str> = text1
        .split_whitespace()
        .filter(|w| w.len() > 3) // Only consider meaningful words
        .collect();
    let words2: std::collections::HashSet<&str> =
        text2.split_whitespace().filter(|w| w.len() > 3).collect();

    if words1.is_empty() || words2.is_empty() {
        return 0.0;
    }

    let intersection = words1.intersection(&words2).count();
    let union = words1.union(&words2).count();

    intersection as f32 / union as f32
}
