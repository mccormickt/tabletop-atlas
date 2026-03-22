use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChatSession {
    pub id: i64,
    pub game_id: i64,
    pub title: Option<String>,
    pub include_house_rules: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChatMessage {
    pub id: i64,
    pub session_id: i64,
    pub role: MessageRole,
    pub content: String,
    pub context_chunks: Option<Vec<i64>>, // IDs of embeddings used for context
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum MessageRole {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
    #[serde(rename = "system")]
    System,
}

impl MessageRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
            MessageRole::System => "system",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "user" => Some(MessageRole::User),
            "assistant" => Some(MessageRole::Assistant),
            "system" => Some(MessageRole::System),
            _ => None,
        }
    }
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateChatSessionRequest {
    pub game_id: i64,
    pub title: Option<String>,
    #[serde(default = "default_true")]
    pub include_house_rules: bool,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateChatSessionRequest {
    pub title: Option<String>,
    pub include_house_rules: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ChatRequest {
    pub session_id: i64,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ChatResponse {
    pub user_message: ChatMessage,
    pub assistant_message: ChatMessage,
    pub context_sources: Vec<ContextSource>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ContextSource {
    pub embedding_id: i64,
    pub chunk_text: String,
    pub source_type: String,
    pub similarity_score: f32,
    pub metadata: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ChatSessionSummary {
    pub id: i64,
    pub game_id: i64,
    pub title: Option<String>,
    pub include_house_rules: bool,
    pub message_count: i32,
    pub last_message_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ChatHistory {
    pub session: ChatSession,
    pub messages: Vec<ChatMessage>,
}
