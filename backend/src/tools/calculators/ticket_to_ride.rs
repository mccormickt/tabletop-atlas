use crate::models::tool::{PlayerScoreResult, ScoreInput, ScoreOutput};
use crate::tools::GameTool;
use crate::tools::scoring::{
    Expansion, InputType, ScoringCategory, ScoringRule, ScoringSchema, SelectOption, ToolError,
    ToolType,
};
use std::collections::HashMap;

pub struct TicketToRideScorer;

impl GameTool for TicketToRideScorer {
    fn tool_id(&self) -> &'static str {
        "ticket-to-ride-scorer"
    }

    fn display_name(&self) -> &'static str {
        "Ticket to Ride Score Calculator"
    }

    fn tool_type(&self) -> ToolType {
        ToolType::ScoreCalculator
    }

    fn player_range(&self) -> (u8, u8) {
        (2, 5)
    }

    fn scoring_schema(&self) -> ScoringSchema {
        ScoringSchema {
            categories: vec![
                ScoringCategory {
                    id: "routes".to_string(),
                    display_name: "Route Points".to_string(),
                    input_type: InputType::Integer,
                    rule: ScoringRule::Direct,
                    min: Some(0),
                    max: Some(200),
                    step: Some(1),
                    requires_expansion: None,
                },
                ScoringCategory {
                    id: "longestRoute".to_string(),
                    display_name: "Longest Route Bonus".to_string(),
                    input_type: InputType::Select {
                        options: vec![
                            SelectOption {
                                value: 0,
                                label: "No".to_string(),
                            },
                            SelectOption {
                                value: 10,
                                label: "Yes".to_string(),
                            },
                        ],
                    },
                    rule: ScoringRule::Direct,
                    min: None,
                    max: None,
                    step: None,
                    requires_expansion: None,
                },
                ScoringCategory {
                    id: "ticketsCompleted".to_string(),
                    display_name: "Completed Tickets".to_string(),
                    input_type: InputType::Integer,
                    rule: ScoringRule::Direct,
                    min: Some(0),
                    max: Some(100),
                    step: Some(1),
                    requires_expansion: None,
                },
                ScoringCategory {
                    id: "ticketsFailed".to_string(),
                    display_name: "Failed Tickets".to_string(),
                    input_type: InputType::Integer,
                    rule: ScoringRule::Direct,
                    min: Some(0),
                    max: Some(100),
                    step: Some(1),
                    requires_expansion: None,
                },
                // USA 1910 expansion
                ScoringCategory {
                    id: "globeTrotter".to_string(),
                    display_name: "Globe Trotter Bonus".to_string(),
                    input_type: InputType::Integer,
                    rule: ScoringRule::Direct,
                    min: Some(0),
                    max: Some(15),
                    step: Some(1),
                    requires_expansion: Some("usa_1910".to_string()),
                },
            ],
            expansions: vec![Expansion {
                id: "usa_1910".to_string(),
                display_name: "USA 1910".to_string(),
            }],
        }
    }

    fn calculate_scores(&self, input: &ScoreInput) -> Result<ScoreOutput, ToolError> {
        let mut results = Vec::new();

        for player in &input.players {
            let mut category_scores = HashMap::new();
            let mut total = 0i32;

            // Direct scoring categories
            for cat in ["routes", "longestRoute", "ticketsCompleted", "globeTrotter"] {
                if let Some(&v) = player.scores.get(cat) {
                    category_scores.insert(cat.to_string(), v);
                    total = total.saturating_add(v);
                }
            }

            // Failed tickets are subtracted (multiplier: -1.0)
            if let Some(&v) = player.scores.get("ticketsFailed") {
                let penalty = v.saturating_neg();
                category_scores.insert("ticketsFailed".to_string(), penalty);
                total = total.saturating_add(penalty);
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
    &TicketToRideScorer as &'static dyn GameTool
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
    fn basic_scoring() {
        let input = make_input(vec![
            (
                "Alice",
                vec![
                    ("routes", 50),
                    ("ticketsCompleted", 20),
                    ("longestRoute", 10),
                ],
            ),
            ("Bob", vec![("routes", 40), ("ticketsCompleted", 30)]),
        ]);
        let result = TicketToRideScorer.calculate_scores(&input).unwrap();
        assert_eq!(result.players[0].total, 80); // 50 + 20 + 10
        assert_eq!(result.players[1].total, 70); // 40 + 30
        assert_eq!(result.winner_index, Some(0));
    }

    #[test]
    fn failed_tickets_subtract() {
        let input = make_input(vec![("Alice", vec![("routes", 50), ("ticketsFailed", 20)])]);
        let result = TicketToRideScorer.calculate_scores(&input).unwrap();
        assert_eq!(result.players[0].total, 30); // 50 - 20
        assert_eq!(
            *result.players[0]
                .category_scores
                .get("ticketsFailed")
                .unwrap(),
            -20
        );
    }

    #[test]
    fn net_negative_score() {
        let input = make_input(vec![("Alice", vec![("routes", 10), ("ticketsFailed", 30)])]);
        let result = TicketToRideScorer.calculate_scores(&input).unwrap();
        assert_eq!(result.players[0].total, -20);
    }

    #[test]
    fn globe_trotter_expansion() {
        let input = make_input(vec![("Alice", vec![("routes", 50), ("globeTrotter", 15)])]);
        let result = TicketToRideScorer.calculate_scores(&input).unwrap();
        assert_eq!(result.players[0].total, 65);
    }

    #[test]
    fn missing_categories_skipped() {
        let input = make_input(vec![("Alice", vec![("routes", 50)])]);
        let result = TicketToRideScorer.calculate_scores(&input).unwrap();
        assert_eq!(result.players[0].total, 50);
        assert!(
            !result.players[0]
                .category_scores
                .contains_key("ticketsFailed")
        );
    }
}
