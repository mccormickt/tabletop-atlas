pub mod calculators;
pub mod scoring;

pub use scoring::*;

use crate::models::tool::{ScoreInput, ScoreOutput};

/// Trait all game tools must implement
pub trait GameTool: Send + Sync {
    fn tool_id(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    fn tool_type(&self) -> ToolType;
    fn player_range(&self) -> (u8, u8);
    fn scoring_schema(&self) -> ScoringSchema;
    fn calculate_scores(&self, input: &ScoreInput) -> Result<ScoreOutput, ToolError>;
}

// Register the collection - enables distributed registration
inventory::collect!(&'static dyn GameTool);

/// Helper to iterate all registered tools
pub fn all_tools() -> impl Iterator<Item = &'static dyn GameTool> {
    inventory::iter::<&'static dyn GameTool>
        .into_iter()
        .copied()
}

/// Get a tool by its ID
pub fn get_tool(tool_id: &str) -> Option<&'static dyn GameTool> {
    all_tools().find(|t| t.tool_id() == tool_id)
}
