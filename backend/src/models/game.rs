use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Game {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub publisher: Option<String>,
    pub year_published: Option<i32>,
    pub min_players: Option<i32>,
    pub max_players: Option<i32>,
    pub play_time_minutes: Option<i32>,
    pub complexity_rating: Option<f64>,
    pub bgg_id: Option<i32>,
    pub rules_pdf_path: Option<String>,
    pub rules_text: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateGameRequest {
    pub name: String,
    pub description: Option<String>,
    pub publisher: Option<String>,
    pub year_published: Option<i32>,
    pub min_players: Option<i32>,
    pub max_players: Option<i32>,
    pub play_time_minutes: Option<i32>,
    pub complexity_rating: Option<f64>,
    pub bgg_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateGameRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub publisher: Option<String>,
    pub year_published: Option<i32>,
    pub min_players: Option<i32>,
    pub max_players: Option<i32>,
    pub play_time_minutes: Option<i32>,
    pub complexity_rating: Option<f64>,
    pub bgg_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GameSummary {
    pub id: i64,
    pub name: String,
    pub publisher: Option<String>,
    pub year_published: Option<i32>,
    pub min_players: Option<i32>,
    pub max_players: Option<i32>,
    pub complexity_rating: Option<f64>,
    pub has_rules_pdf: bool,
    pub house_rules_count: i32,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RulesInfoResponse {
    pub game_id: i64,
    pub game_name: String,
    pub has_rules_pdf: bool,
    pub rules_pdf_path: Option<String>,
    pub text_length: Option<usize>,
    pub chunk_count: i64,
    pub last_processed: Option<String>,
}
