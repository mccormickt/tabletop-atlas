use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use super::{GameId, ChatSessionId, ChatMessageId, EmbeddingId};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChatSession {
    pub id: ChatSessionId,
    pub game_id: GameId,
    pub title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChatMessage {
    pub id: ChatMessageId,
    pub session_id: ChatSessionId,
    pub role: MessageRole,
    pub content: String,
    pub context_chunks: Option<Vec<EmbeddingId>>, // IDs of embeddings used for context
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

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateChatSessionRequest {
    pub game_id: GameId,
    pub title: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ChatRequest {
    pub session_id: ChatSessionId,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ChatResponse {
    pub message: ChatMessage,
    pub context_sources: Vec<ContextSource>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ContextSource {
    pub embedding_id: EmbeddingId,
    pub chunk_text: String,
    pub source_type: String,
    pub similarity_score: f32,
    pub metadata: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ChatSessionSummary {
    pub id: ChatSessionId,
    pub game_id: GameId,
    pub title: Option<String>,
    pub message_count: i32,
    pub last_message_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ChatHistory {
    pub session: ChatSession,
    pub messages: Vec<ChatMessage>,
}

impl ChatSession {
    pub fn to_summary(&self, message_count: i32, last_message_at: Option<DateTime<Utc>>) -> ChatSessionSummary {
        ChatSessionSummary {
            id: self.id,
            game_id: self.game_id,
            title: self.title.clone(),
            message_count,
            last_message_at,
            created_at: self.created_at,
        }
    }
}

// Helper struct for LLM API integration
#[derive(Debug, Serialize, Deserialize)]
pub struct LLMMessage {
    pub role: String,
    pub content: String,
}

impl From<&ChatMessage> for LLMMessage {
    fn from(message: &ChatMessage) -> Self {
        LLMMessage {
            role: message.role.as_str().to_string(),
            content: message.content.clone(),
        }
    }
}