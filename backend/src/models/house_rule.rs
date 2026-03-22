use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HouseRule {
    pub id: i64,
    pub game_id: i64,
    pub title: String,
    pub description: String,
    pub category: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateHouseRuleRequest {
    pub game_id: i64,
    pub title: String,
    pub description: String,
    pub category: Option<String>,
    #[serde(default = "default_true")]
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateHouseRuleRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub is_active: Option<bool>,
}

fn default_true() -> bool {
    true
}
