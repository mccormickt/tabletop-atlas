use crate::models::tool::{PlayerScoreResult, ScoreInput, ScoreOutput};
use crate::tools::scoring::{
    Expansion, InputType, ScoringCategory, ScoringRule, ScoringSchema, ToolError, ToolType,
};
use crate::tools::GameTool;
use std::collections::HashMap;

pub struct SevenWondersScorer;

impl GameTool for SevenWondersScorer {
    fn tool_id(&self) -> &'static str {
        "7-wonders-scorer"
    }

    fn display_name(&self) -> &'static str {
        "7 Wonders Score Calculator"
    }

    fn tool_type(&self) -> ToolType {
        ToolType::ScoreCalculator
    }

    fn player_range(&self) -> (u8, u8) {
        (2, 7)
    }

    fn scoring_schema(&self) -> ScoringSchema {
        ScoringSchema {
            categories: vec![
                ScoringCategory {
                    id: "military".to_string(),
                    display_name: "Military".to_string(),
                    input_type: InputType::Integer,
                    rule: ScoringRule::Direct,
                    min: Some(-6),
                    max: Some(18),
                    step: Some(1),
                    requires_expansion: None,
                },
                ScoringCategory {
                    id: "treasury".to_string(),
                    display_name: "Treasury (coins)".to_string(),
                    input_type: InputType::Counter,
                    rule: ScoringRule::Multiplier { factor: 1.0 / 3.0 },
                    min: Some(0),
                    max: None,
                    step: Some(1),
                    requires_expansion: None,
                },
                ScoringCategory {
                    id: "wonders".to_string(),
                    display_name: "Wonders".to_string(),
                    input_type: InputType::Integer,
                    rule: ScoringRule::Direct,
                    min: Some(0),
                    max: Some(20),
                    step: Some(1),
                    requires_expansion: None,
                },
                ScoringCategory {
                    id: "civilian".to_string(),
                    display_name: "Civilian (Blue)".to_string(),
                    input_type: InputType::Integer,
                    rule: ScoringRule::Direct,
                    min: Some(0),
                    max: None,
                    step: Some(1),
                    requires_expansion: None,
                },
                ScoringCategory {
                    id: "science".to_string(),
                    display_name: "Science".to_string(),
                    input_type: InputType::ScienceSymbols,
                    rule: ScoringRule::Custom {
                        formula: "sets*7 + tablets^2 + compasses^2 + gears^2".to_string(),
                    },
                    min: None,
                    max: None,
                    step: None,
                    requires_expansion: None,
                },
                ScoringCategory {
                    id: "commercial".to_string(),
                    display_name: "Commercial (Yellow)".to_string(),
                    input_type: InputType::Integer,
                    rule: ScoringRule::Direct,
                    min: Some(0),
                    max: None,
                    step: Some(1),
                    requires_expansion: None,
                },
                ScoringCategory {
                    id: "guilds".to_string(),
                    display_name: "Guilds (Purple)".to_string(),
                    input_type: InputType::Integer,
                    rule: ScoringRule::Direct,
                    min: Some(0),
                    max: None,
                    step: Some(1),
                    requires_expansion: None,
                },
                // Leaders expansion category
                ScoringCategory {
                    id: "leaders".to_string(),
                    display_name: "Leaders".to_string(),
                    input_type: InputType::Integer,
                    rule: ScoringRule::Direct,
                    min: Some(0),
                    max: None,
                    step: Some(1),
                    requires_expansion: Some("leaders".to_string()),
                },
                // Cities expansion category
                ScoringCategory {
                    id: "cities".to_string(),
                    display_name: "Cities".to_string(),
                    input_type: InputType::Integer,
                    rule: ScoringRule::Direct,
                    min: Some(0),
                    max: None,
                    step: Some(1),
                    requires_expansion: Some("cities".to_string()),
                },
            ],
            expansions: vec![
                Expansion {
                    id: "leaders".to_string(),
                    display_name: "Leaders".to_string(),
                },
                Expansion {
                    id: "cities".to_string(),
                    display_name: "Cities".to_string(),
                },
            ],
        }
    }

    fn calculate_scores(&self, input: &ScoreInput) -> Result<ScoreOutput, ToolError> {
        let mut results = Vec::new();

        for player in &input.players {
            let mut category_scores = HashMap::new();
            let mut total = 0i32;

            // Treasury (coins / 3) - special multiplier rule
            if let Some(&v) = player.scores.get("treasury") {
                let score = v / 3;
                category_scores.insert("treasury".to_string(), score);
                total = total.saturating_add(score);
            }

            // Direct scoring categories (including military)
            for cat in [
                "military", "wonders", "civilian", "commercial", "guilds", "leaders", "cities",
            ] {
                if let Some(&v) = player.scores.get(cat) {
                    category_scores.insert(cat.to_string(), v);
                    total = total.saturating_add(v);
                }
            }

            // Science (special calculation: sets*7 + sum of squares)
            // Using saturating arithmetic to prevent overflow
            let tablets = player.scores.get("scienceTablets").copied().unwrap_or(0);
            let compasses = player.scores.get("scienceCompasses").copied().unwrap_or(0);
            let gears = player.scores.get("scienceGears").copied().unwrap_or(0);

            let sets = tablets.min(compasses).min(gears);
            let tablets_squared = tablets.saturating_mul(tablets);
            let compasses_squared = compasses.saturating_mul(compasses);
            let gears_squared = gears.saturating_mul(gears);
            let sets_score = sets.saturating_mul(7);

            let science_score = sets_score
                .saturating_add(tablets_squared)
                .saturating_add(compasses_squared)
                .saturating_add(gears_squared);
            category_scores.insert("science".to_string(), science_score);
            total = total.saturating_add(science_score);

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
    &SevenWondersScorer as &'static dyn GameTool
}
