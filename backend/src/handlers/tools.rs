use dropshot::{HttpError, Path, RequestContext, TypedBody, endpoint};
use schemars::JsonSchema;
use serde::Deserialize;
use std::collections::HashSet;

use crate::AppState;
use crate::models::tool::{PlayerRange, ScoreInput, ScoreOutput, ToolDetails, ToolSummary};
use crate::tools;
use crate::tools::GameTool;

use super::{HttpOk, bad_request_error, not_found_error, success_response};

/// Maximum allowed length for player names
const MAX_PLAYER_NAME_LENGTH: usize = 100;

/// Validates the score input against the tool's schema and constraints
fn validate_score_input(input: &ScoreInput, tool: &dyn GameTool) -> Result<(), String> {
    let schema = tool.scoring_schema();
    let (min_players, max_players) = tool.player_range();

    // Validate player count
    let player_count = input.players.len();
    if player_count < min_players as usize {
        return Err(format!(
            "Too few players: {} provided, minimum is {}",
            player_count, min_players
        ));
    }
    if player_count > max_players as usize {
        return Err(format!(
            "Too many players: {} provided, maximum is {}",
            player_count, max_players
        ));
    }

    // Build set of valid category IDs
    let valid_categories: HashSet<&str> = schema.categories.iter().map(|c| c.id.as_str()).collect();

    // Build set of valid expansion IDs
    let valid_expansions: HashSet<&str> = schema.expansions.iter().map(|e| e.id.as_str()).collect();

    // Validate expansion IDs
    for expansion_id in &input.enabled_expansions {
        if !valid_expansions.contains(expansion_id.as_str()) {
            return Err(format!(
                "Unknown expansion: '{}'. Valid expansions: {:?}",
                expansion_id,
                schema.expansions.iter().map(|e| &e.id).collect::<Vec<_>>()
            ));
        }
    }

    // Validate each player's input
    for (i, player) in input.players.iter().enumerate() {
        // Validate player name length
        if player.name.len() > MAX_PLAYER_NAME_LENGTH {
            return Err(format!(
                "Player {} name too long: {} characters (maximum {} allowed)",
                i + 1,
                player.name.len(),
                MAX_PLAYER_NAME_LENGTH
            ));
        }

        // Validate score category keys
        for key in player.scores.keys() {
            // Skip science symbol keys which are special
            if key.starts_with("science") && key != "science" {
                continue;
            }
            if !valid_categories.contains(key.as_str()) {
                return Err(format!(
                    "Unknown score category '{}' for player {}. Valid categories: {:?}",
                    key,
                    i + 1,
                    schema.categories.iter().map(|c| &c.id).collect::<Vec<_>>()
                ));
            }
        }
    }

    Ok(())
}

/// List all available tools
#[endpoint {
    method = GET,
    path = "/api/tools",
}]
pub async fn list_tools(
    _rqctx: RequestContext<AppState>,
) -> Result<HttpOk<Vec<ToolSummary>>, HttpError> {
    let tools: Vec<ToolSummary> = tools::all_tools()
        .map(|t| ToolSummary {
            id: t.tool_id().to_string(),
            display_name: t.display_name().to_string(),
            tool_type: t.tool_type(),
            player_range: PlayerRange::from(t.player_range()),
        })
        .collect();

    success_response(tools)
}

#[derive(Deserialize, JsonSchema)]
pub struct ToolIdPath {
    pub tool_id: String,
}

/// Get tool details including scoring schema
#[endpoint {
    method = GET,
    path = "/api/tools/{tool_id}",
}]
pub async fn get_tool(
    _rqctx: RequestContext<AppState>,
    path: Path<ToolIdPath>,
) -> Result<HttpOk<ToolDetails>, HttpError> {
    let tool_id = &path.into_inner().tool_id;

    let tool = tools::get_tool(tool_id)
        .ok_or_else(|| not_found_error(format!("Tool not found: {}", tool_id)))?;

    let details = ToolDetails {
        id: tool.tool_id().to_string(),
        display_name: tool.display_name().to_string(),
        tool_type: tool.tool_type(),
        player_range: PlayerRange::from(tool.player_range()),
        schema: tool.scoring_schema(),
    };

    success_response(details)
}

/// Calculate scores for a tool (stateless)
#[endpoint {
    method = POST,
    path = "/api/tools/{tool_id}/calculate",
}]
pub async fn calculate_scores(
    _rqctx: RequestContext<AppState>,
    path: Path<ToolIdPath>,
    body: TypedBody<ScoreInput>,
) -> Result<HttpOk<ScoreOutput>, HttpError> {
    let tool_id = &path.into_inner().tool_id;
    let input = body.into_inner();

    let tool = tools::get_tool(tool_id)
        .ok_or_else(|| not_found_error(format!("Tool not found: {}", tool_id)))?;

    // Validate input before processing
    validate_score_input(&input, tool).map_err(bad_request_error)?;

    let output = tool
        .calculate_scores(&input)
        .map_err(|e| bad_request_error(e.to_string()))?;

    success_response(output)
}
