use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use super::GameId;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Game {
    pub id: GameId,
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
    pub id: GameId,
    pub name: String,
    pub publisher: Option<String>,
    pub year_published: Option<i32>,
    pub min_players: Option<i32>,
    pub max_players: Option<i32>,
    pub complexity_rating: Option<f64>,
    pub has_rules_pdf: bool,
    pub house_rules_count: i32,
}

impl Game {
    pub fn to_summary(&self, house_rules_count: i32) -> GameSummary {
        GameSummary {
            id: self.id,
            name: self.name.clone(),
            publisher: self.publisher.clone(),
            year_published: self.year_published,
            min_players: self.min_players,
            max_players: self.max_players,
            complexity_rating: self.complexity_rating,
            has_rules_pdf: self.rules_pdf_path.is_some(),
            house_rules_count,
        }
    }
}