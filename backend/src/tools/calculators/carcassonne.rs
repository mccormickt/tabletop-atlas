use crate::models::tool::{PlayerScoreResult, ScoreInput, ScoreOutput};
use crate::tools::scoring::{
    Expansion, InputType, ScoringCategory, ScoringRule, ScoringSchema, ToolError, ToolType,
};
use crate::tools::GameTool;
use std::collections::HashMap;

pub struct CarcassonneScorer;

impl GameTool for CarcassonneScorer {
    fn tool_id(&self) -> &'static str {
        "carcassonne-scorer"
    }

    fn display_name(&self) -> &'static str {
        "Carcassonne Score Calculator"
    }

    fn tool_type(&self) -> ToolType {
        ToolType::ScoreCalculator
    }

    fn player_range(&self) -> (u8, u8) {
        (2, 6)
    }

    fn scoring_schema(&self) -> ScoringSchema {
        ScoringSchema {
            categories: vec![
                ScoringCategory {
                    id: "cities".to_string(),
                    display_name: "Cities".to_string(),
                    input_type: InputType::Integer,
                    rule: ScoringRule::Direct,
                    min: Some(0),
                    max: None,
                    step: Some(1),
                    requires_expansion: None,
                },
                ScoringCategory {
                    id: "roads".to_string(),
                    display_name: "Roads".to_string(),
                    input_type: InputType::Integer,
                    rule: ScoringRule::Direct,
                    min: Some(0),
                    max: None,
                    step: Some(1),
                    requires_expansion: None,
                },
                ScoringCategory {
                    id: "monasteries".to_string(),
                    display_name: "Monasteries".to_string(),
                    input_type: InputType::Integer,
                    rule: ScoringRule::Direct,
                    min: Some(0),
                    max: None,
                    step: Some(1),
                    requires_expansion: None,
                },
                ScoringCategory {
                    id: "fields".to_string(),
                    display_name: "Fields (Farmers)".to_string(),
                    input_type: InputType::Integer,
                    rule: ScoringRule::Direct,
                    min: Some(0),
                    max: None,
                    step: Some(1),
                    requires_expansion: None,
                },
                // Inns & Cathedrals expansion
                ScoringCategory {
                    id: "innsCathedrals".to_string(),
                    display_name: "Inns & Cathedrals Bonus".to_string(),
                    input_type: InputType::Integer,
                    rule: ScoringRule::Direct,
                    min: Some(0),
                    max: None,
                    step: Some(1),
                    requires_expansion: Some("inns_cathedrals".to_string()),
                },
                // Traders & Builders expansion
                ScoringCategory {
                    id: "goods".to_string(),
                    display_name: "Trade Goods".to_string(),
                    input_type: InputType::Integer,
                    rule: ScoringRule::Direct,
                    min: Some(0),
                    max: None,
                    step: Some(1),
                    requires_expansion: Some("traders_builders".to_string()),
                },
            ],
            expansions: vec![
                Expansion {
                    id: "inns_cathedrals".to_string(),
                    display_name: "Inns & Cathedrals".to_string(),
                },
                Expansion {
                    id: "traders_builders".to_string(),
                    display_name: "Traders & Builders".to_string(),
                },
            ],
        }
    }

    fn calculate_scores(&self, input: &ScoreInput) -> Result<ScoreOutput, ToolError> {
        let mut results = Vec::new();

        for player in &input.players {
            let mut category_scores = HashMap::new();
            let mut total = 0i32;

            // All categories in Carcassonne use direct scoring
            // Using saturating arithmetic to prevent overflow
            for cat in [
                "cities",
                "roads",
                "monasteries",
                "fields",
                "innsCathedrals",
                "goods",
            ] {
                if let Some(&v) = player.scores.get(cat) {
                    category_scores.insert(cat.to_string(), v);
                    total = total.saturating_add(v);
                }
            }

            results.push(PlayerScoreResult {
                name: player.name.clone(),
                category_scores,
                total,
            });
        }

        // Determine winner (highest score)
        let winner_index = results
            .iter()
            .enumerate()
            .max_by_key(|(_, r)| r.total)
            .map(|(i, _)| i);

        Ok(ScoreOutput {
            players: results,
            winner_index,
        })
    }
}

// Self-registration - happens at program startup
inventory::submit! {
    &CarcassonneScorer as &'static dyn GameTool
}
