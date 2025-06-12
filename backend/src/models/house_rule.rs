use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use super::{GameId, HouseRuleId};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HouseRule {
    pub id: HouseRuleId,
    pub game_id: GameId,
    pub title: String,
    pub description: String,
    pub category: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateHouseRuleRequest {
    pub game_id: GameId,
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

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct HouseRuleSummary {
    pub id: HouseRuleId,
    pub game_id: GameId,
    pub title: String,
    pub category: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

impl HouseRule {
    pub fn to_summary(&self) -> HouseRuleSummary {
        HouseRuleSummary {
            id: self.id,
            game_id: self.game_id,
            title: self.title.clone(),
            category: self.category.clone(),
            is_active: self.is_active,
            created_at: self.created_at,
        }
    }
}

fn default_true() -> bool {
    true
}