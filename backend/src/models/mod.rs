// DateTime and Utc re-exported from individual modules as needed
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod chat;
pub mod embedding;
pub mod game;
pub mod house_rule;

pub use chat::*;
pub use embedding::*;
pub use game::*;
pub use house_rule::*;

// Common types used across models
pub type GameId = i64;
pub type HouseRuleId = i64;
pub type EmbeddingId = i64;
pub type ChatSessionId = i64;
pub type ChatMessageId = i64;



// Pagination parameters
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_page() -> u32 {
    1
}
fn default_limit() -> u32 {
    20
}

// Paginated response
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: u32,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
}

impl<T> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, total: u32, page: u32, limit: u32) -> Self {
        let total_pages = (total as f64 / limit as f64).ceil() as u32;
        Self {
            items,
            total,
            page,
            limit,
            total_pages,
        }
    }
}
