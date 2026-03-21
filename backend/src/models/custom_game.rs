use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CustomGame {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub publisher: Option<String>,
    pub year_published: Option<i32>,
    pub min_players: Option<i32>,
    pub max_players: Option<i32>,
    pub play_time_minutes: Option<i32>,
    pub complexity_rating: Option<f64>,
    pub rules_pdf_path: Option<String>,
    pub rules_text: Option<String>,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CustomGameSummary {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub publisher: Option<String>,
    pub year_published: Option<i32>,
    pub min_players: Option<i32>,
    pub max_players: Option<i32>,
    pub complexity_rating: Option<f64>,
    pub is_public: bool,
    pub has_rules_pdf: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateCustomGameRequest {
    pub name: String,
    pub description: Option<String>,
    pub publisher: Option<String>,
    pub year_published: Option<i32>,
    pub min_players: Option<i32>,
    pub max_players: Option<i32>,
    pub play_time_minutes: Option<i32>,
    pub complexity_rating: Option<f64>,
    pub is_public: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateCustomGameRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub publisher: Option<String>,
    pub year_published: Option<i32>,
    pub min_players: Option<i32>,
    pub max_players: Option<i32>,
    pub play_time_minutes: Option<i32>,
    pub complexity_rating: Option<f64>,
    pub is_public: Option<bool>,
}
