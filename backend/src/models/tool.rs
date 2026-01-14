use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::tools::scoring::{ScoringSchema, ToolType};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlayerRange {
    pub min: u8,
    pub max: u8,
}

impl PlayerRange {
    /// Creates a new PlayerRange with validation.
    /// Panics if min > max or min < 1.
    pub fn new(min: u8, max: u8) -> Self {
        assert!(min >= 1, "PlayerRange min must be at least 1");
        assert!(
            min <= max,
            "PlayerRange min ({}) must be <= max ({})",
            min,
            max
        );
        PlayerRange { min, max }
    }
}

impl From<(u8, u8)> for PlayerRange {
    fn from((min, max): (u8, u8)) -> Self {
        PlayerRange::new(min, max)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ToolSummary {
    pub id: String,
    pub display_name: String,
    pub tool_type: ToolType,
    pub player_range: PlayerRange,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ToolDetails {
    pub id: String,
    pub display_name: String,
    pub tool_type: ToolType,
    pub player_range: PlayerRange,
    pub schema: ScoringSchema,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ScoreInput {
    pub players: Vec<PlayerScoreInput>,
    pub enabled_expansions: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlayerScoreInput {
    pub name: String,
    pub scores: HashMap<String, i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ScoreOutput {
    pub players: Vec<PlayerScoreResult>,
    pub winner_index: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlayerScoreResult {
    pub name: String,
    pub category_scores: HashMap<String, i32>,
    pub total: i32,
}
