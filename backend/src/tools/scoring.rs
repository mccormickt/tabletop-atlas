use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ToolType {
    ScoreCalculator,
    Timer,
    DiceRoller,
    Randomizer,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ScoringSchema {
    pub categories: Vec<ScoringCategory>,
    pub expansions: Vec<Expansion>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ScoringCategory {
    pub id: String,
    pub display_name: String,
    pub input_type: InputType,
    pub rule: ScoringRule,
    pub min: Option<i32>,
    pub max: Option<i32>,
    pub step: Option<i32>,
    /// If set, this category only appears when this expansion is enabled
    pub requires_expansion: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum InputType {
    Integer,
    Counter,
    Checkbox,
    Select { options: Vec<SelectOption> },
    ScienceSymbols,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SelectOption {
    pub value: i32,
    pub label: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ScoringRule {
    Direct,
    Multiplier { factor: f32 },
    Threshold { thresholds: Vec<ThresholdEntry> },
    Custom { formula: String },
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ThresholdEntry {
    pub min: i32,
    pub max: i32,
    pub score: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Expansion {
    pub id: String,
    pub display_name: String,
}

#[derive(Clone, Debug)]
pub struct ToolError(pub String);

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ToolError {}
