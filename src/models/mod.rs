use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod state;
pub use state::AppState;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Game {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateGameRequest {
    pub name: String,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct PdfDocument {
    pub id: Uuid,
    pub game_id: Uuid,
    pub filename: String,
    pub content: String,
    pub file_size: u64,
    pub uploaded_at: String,
    pub processed: bool,
    pub chunk_count: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreatePdfDocumentRequest {
    pub game_id: Uuid,
    pub filename: String,
    pub content: String,
    pub file_size: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct HouseRule {
    pub id: Uuid,
    pub game_id: Uuid,
    pub title: String,
    pub content: String,
    pub created_by: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateHouseRuleRequest {
    pub game_id: Uuid,
    pub title: String,
    pub content: String,
    pub created_by: String,
}

// Path parameter structs
#[derive(Deserialize, JsonSchema)]
pub struct GamePathParams {
    pub game_id: Uuid,
}
