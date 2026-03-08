use crate::models::tool::{PlayerScoreResult, ScoreInput, ScoreOutput};
use crate::tools::GameTool;
use crate::tools::scoring::{
    Expansion, InputType, ScoringCategory, ScoringRule, ScoringSchema, ToolError, ToolType,
};
use std::collections::HashMap;

pub struct WingspanScorer;

impl GameTool for WingspanScorer {
    fn tool_id(&self) -> &'static str {
        "wingspan-scorer"
    }

    fn display_name(&self) -> &'static str {
        "Wingspan Score Calculator"
    }

    fn tool_type(&self) -> ToolType {
        ToolType::ScoreCalculator
    }

    fn player_range(&self) -> (u8, u8) {
        (1, 5)
    }

    fn scoring_schema(&self) -> ScoringSchema {
        ScoringSchema {
            categories: vec![
                ScoringCategory {
                    id: "birds".to_string(),
                    display_name: "Bird Points".to_string(),
                    input_type: InputType::Integer,
                    rule: ScoringRule::Direct,
                    min: Some(0),
                    max: Some(150),
                    step: Some(1),
                    requires_expansion: None,
                },
                ScoringCategory {
                    id: "bonusCards".to_string(),
                    display_name: "Bonus Cards".to_string(),
                    input_type: InputType::Integer,
                    rule: ScoringRule::Direct,
                    min: Some(0),
                    max: Some(50),
                    step: Some(1),
                    requires_expansion: None,
                },
                ScoringCategory {
                    id: "endOfRound".to_string(),
                    display_name: "End-of-Round Goals".to_string(),
                    input_type: InputType::Integer,
                    rule: ScoringRule::Direct,
                    min: Some(0),
                    max: Some(36),
                    step: Some(1),
                    requires_expansion: None,
                },
                ScoringCategory {
                    id: "eggs".to_string(),
                    display_name: "Eggs on Cards".to_string(),
                    input_type: InputType::Counter,
                    rule: ScoringRule::Direct,
                    min: Some(0),
                    max: Some(50),
                    step: Some(1),
                    requires_expansion: None,
                },
                ScoringCategory {
                    id: "foodOnCards".to_string(),
                    display_name: "Cached Food".to_string(),
                    input_type: InputType::Counter,
                    rule: ScoringRule::Direct,
                    min: Some(0),
                    max: Some(30),
                    step: Some(1),
                    requires_expansion: None,
                },
                ScoringCategory {
                    id: "tuckedCards".to_string(),
                    display_name: "Tucked Cards".to_string(),
                    input_type: InputType::Counter,
                    rule: ScoringRule::Direct,
                    min: Some(0),
                    max: Some(40),
                    step: Some(1),
                    requires_expansion: None,
                },
                // Oceania expansion
                ScoringCategory {
                    id: "nectar".to_string(),
                    display_name: "Nectar Points".to_string(),
                    input_type: InputType::Integer,
                    rule: ScoringRule::Direct,
                    min: Some(0),
                    max: Some(25),
                    step: Some(1),
                    requires_expansion: Some("oceania".to_string()),
                },
            ],
            expansions: vec![Expansion {
                id: "oceania".to_string(),
                display_name: "Oceania Expansion".to_string(),
            }],
        }
    }

    fn calculate_scores(&self, input: &ScoreInput) -> Result<ScoreOutput, ToolError> {
        let mut results = Vec::new();

        for player in &input.players {
            let mut category_scores = HashMap::new();
            let mut total = 0i32;

            for cat in [
                "birds",
                "bonusCards",
                "endOfRound",
                "eggs",
                "foodOnCards",
                "tuckedCards",
                "nectar",
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

inventory::submit! {
    &WingspanScorer as &'static dyn GameTool
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::tool::PlayerScoreInput;

    fn make_input(players: Vec<(&str, Vec<(&str, i32)>)>) -> ScoreInput {
        ScoreInput {
            players: players
                .into_iter()
                .map(|(name, scores)| PlayerScoreInput {
                    name: name.to_string(),
                    scores: scores
                        .into_iter()
                        .map(|(k, v)| (k.to_string(), v))
                        .collect(),
                })
                .collect(),
            enabled_expansions: vec![],
        }
    }

    #[test]
    fn all_categories_sum() {
        let input = make_input(vec![(
            "Alice",
            vec![
                ("birds", 40),
                ("bonusCards", 10),
                ("endOfRound", 15),
                ("eggs", 8),
                ("foodOnCards", 5),
                ("tuckedCards", 3),
            ],
        )]);
        let result = WingspanScorer.calculate_scores(&input).unwrap();
        assert_eq!(result.players[0].total, 81);
    }

    #[test]
    fn nectar_expansion_included() {
        let input = make_input(vec![("Alice", vec![("birds", 30), ("nectar", 15)])]);
        let result = WingspanScorer.calculate_scores(&input).unwrap();
        assert_eq!(result.players[0].total, 45);
    }

    #[test]
    fn winner_detection() {
        let input = make_input(vec![
            ("Alice", vec![("birds", 40), ("eggs", 10)]),
            ("Bob", vec![("birds", 60), ("eggs", 5)]),
        ]);
        let result = WingspanScorer.calculate_scores(&input).unwrap();
        assert_eq!(result.players[0].total, 50);
        assert_eq!(result.players[1].total, 65);
        assert_eq!(result.winner_index, Some(1));
    }
}
