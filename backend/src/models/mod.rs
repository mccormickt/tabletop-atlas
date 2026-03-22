// DateTime and Utc re-exported from individual modules as needed
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod admin;
pub mod challenge;
pub mod chat;
pub mod collection;
pub mod custom_game;
pub mod embedding;
pub mod game;
pub mod house_rule;
pub mod tool;
pub mod user;

pub use challenge::*;
pub use chat::*;
pub use collection::*;
pub use custom_game::*;
pub use embedding::*;
pub use game::*;
pub use house_rule::*;
pub use user::*;

// Note: tool module uses explicit imports (crate::models::tool::*) for clarity

// Pagination parameters
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

pub fn default_page() -> u32 {
    1
}
pub fn default_limit() -> u32 {
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
