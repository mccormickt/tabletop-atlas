use super::{EmbeddingId, GameId, HouseRuleId};
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Embedding {
    pub id: EmbeddingId,
    pub game_id: GameId,
    pub chunk_text: String,
    pub embedding: Vec<f32>, // Vector embedding
    pub chunk_index: i32,
    pub source_type: EmbeddingSourceType,
    pub source_id: Option<HouseRuleId>,
    pub metadata: Option<String>, // JSON string
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub enum EmbeddingSourceType {
    #[serde(rename = "rules_pdf")]
    RulesPdf,
    #[serde(rename = "house_rule")]
    HouseRule,
}

impl EmbeddingSourceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EmbeddingSourceType::RulesPdf => "rules_pdf",
            EmbeddingSourceType::HouseRule => "house_rule",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "rules_pdf" => Some(EmbeddingSourceType::RulesPdf),
            "house_rule" => Some(EmbeddingSourceType::HouseRule),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateEmbeddingRequest {
    pub game_id: GameId,
    pub chunk_text: String,
    pub embedding: Vec<f32>,
    pub chunk_index: i32,
    pub source_type: EmbeddingSourceType,
    pub source_id: Option<HouseRuleId>,
    pub metadata: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct EmbeddingSearchResult {
    pub id: EmbeddingId,
    pub chunk_text: String,
    pub similarity_score: f32,
    pub source_type: EmbeddingSourceType,
    pub source_id: Option<HouseRuleId>,
    pub metadata: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SimilaritySearchRequest {
    pub game_id: GameId,
    pub query_embedding: Vec<f32>,
    #[serde(default = "default_search_limit")]
    pub limit: u32,
    #[serde(default = "default_similarity_threshold")]
    pub similarity_threshold: f32,
}

fn default_search_limit() -> u32 {
    10
}

fn default_similarity_threshold() -> f32 {
    0.5
}
