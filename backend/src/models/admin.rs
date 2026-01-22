use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Response for BGG import preview
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct BggImportPreviewResponse {
    /// Games that will be inserted (new)
    pub games_to_insert: Vec<BggGamePreview>,
    /// Games that will be updated (existing by bgg_id)
    pub games_to_update: Vec<BggGameUpdatePreview>,
    /// Parsing errors encountered
    pub errors: Vec<BggParseError>,
    /// Total rows in the CSV
    pub total_rows: u32,
}

/// Response for BGG import execution
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct BggImportResponse {
    /// Number of games inserted
    pub inserted_count: u32,
    /// Number of games updated
    pub updated_count: u32,
    /// Errors that occurred during import
    pub errors: Vec<BggParseError>,
}

/// Preview of a game to be inserted from BGG CSV
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BggGamePreview {
    /// Row number in CSV (1-indexed)
    pub row: u32,
    /// Game name from BGG
    pub name: String,
    /// BGG object ID
    pub bgg_id: i32,
    /// Year published
    pub year_published: Option<i32>,
    /// Minimum players
    pub min_players: Option<i32>,
    /// Maximum players
    pub max_players: Option<i32>,
    /// Playing time in minutes
    pub play_time_minutes: Option<i32>,
    /// Average weight/complexity (1.0-5.0)
    pub complexity_rating: Option<f64>,
}

/// Preview of a game that will be updated
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BggGameUpdatePreview {
    /// Row number in CSV (1-indexed)
    pub row: u32,
    /// Existing database ID
    pub existing_id: i64,
    /// BGG object ID
    pub bgg_id: i32,
    /// Game name
    pub name: String,
    /// Fields that will change
    pub changes: Vec<FieldChange>,
}

/// A field that will be changed during update
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FieldChange {
    /// Field name
    pub field: String,
    /// Current value (as string for display)
    pub old_value: Option<String>,
    /// New value (as string for display)
    pub new_value: Option<String>,
}

/// Error that occurred while parsing a row
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BggParseError {
    /// Row number in CSV (1-indexed)
    pub row: u32,
    /// Error message
    pub message: String,
}

/// Internal struct for parsed BGG game data
#[derive(Debug, Clone)]
pub struct ParsedBggGame {
    pub row: u32,
    pub name: String,
    pub bgg_id: i32,
    pub year_published: Option<i32>,
    pub min_players: Option<i32>,
    pub max_players: Option<i32>,
    pub play_time_minutes: Option<i32>,
    pub complexity_rating: Option<f64>,
}

impl ParsedBggGame {
    pub fn into_preview(self) -> BggGamePreview {
        BggGamePreview {
            row: self.row,
            name: self.name,
            bgg_id: self.bgg_id,
            year_published: self.year_published,
            min_players: self.min_players,
            max_players: self.max_players,
            play_time_minutes: self.play_time_minutes,
            complexity_rating: self.complexity_rating,
        }
    }
}

// ============================================================================
// BGG API Enrichment Models
// ============================================================================

/// Values for comparing current game data vs BGG data
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BggGameValues {
    /// Game name
    pub name: String,
    /// Game description
    pub description: Option<String>,
    /// Year published
    pub year_published: Option<i32>,
    /// Minimum players
    pub min_players: Option<i32>,
    /// Maximum players
    pub max_players: Option<i32>,
    /// Playing time in minutes
    pub play_time_minutes: Option<i32>,
    /// Average weight/complexity (1.0-5.0)
    pub complexity_rating: Option<f64>,
}

/// Response for single game BGG enrichment preview
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct BggEnrichPreviewResponse {
    /// Database game ID
    pub game_id: i64,
    /// BGG ID
    pub bgg_id: i32,
    /// Current values in our database
    pub current_values: BggGameValues,
    /// Values from BGG API
    pub bgg_values: BggGameValues,
    /// List of fields that differ
    pub changes: Vec<FieldChange>,
}

/// Request to execute single game BGG enrichment
#[derive(Debug, Deserialize, JsonSchema)]
pub struct BggEnrichRequest {
    /// Which fields to update from BGG data
    pub fields_to_update: Vec<String>,
}

/// Statistics about games needing enrichment
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct EnrichmentStats {
    /// Total games with a BGG ID
    pub total_with_bgg_id: u32,
    /// Games missing year_published
    pub missing_year: u32,
    /// Games missing player counts (min or max)
    pub missing_players: u32,
    /// Games missing play_time_minutes
    pub missing_play_time: u32,
    /// Games missing complexity_rating
    pub missing_complexity: u32,
    /// Games missing description
    pub missing_description: u32,
    /// Games missing at least one field
    pub missing_any: u32,
}

/// Request for bulk BGG enrichment
#[derive(Debug, Deserialize, JsonSchema)]
pub struct BulkEnrichRequest {
    /// Which fields to enrich (e.g., ["year_published", "min_players"])
    pub fields_to_enrich: Vec<String>,
    /// Maximum number of games to process (default 50)
    pub limit: Option<u32>,
}

/// Preview of a game that will be enriched from BGG
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BggGameEnrichPreview {
    /// Database game ID
    pub game_id: i64,
    /// BGG ID
    pub bgg_id: i32,
    /// Game name
    pub name: String,
    /// Fields that will change
    pub changes: Vec<FieldChange>,
}

/// Error that occurred while enriching a game from BGG
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BggEnrichError {
    /// Database game ID
    pub game_id: i64,
    /// BGG ID
    pub bgg_id: i32,
    /// Error message
    pub message: String,
}

/// Response for bulk BGG enrichment preview
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct BulkEnrichPreviewResponse {
    /// Games that will be updated
    pub games_to_update: Vec<BggGameEnrichPreview>,
    /// Errors encountered while fetching from BGG
    pub errors: Vec<BggEnrichError>,
    /// Total games fetched from BGG
    pub total_fetched: u32,
}

/// Response for bulk BGG enrichment execution
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct BulkEnrichResponse {
    /// Number of games updated
    pub updated_count: u32,
    /// Errors encountered during update
    pub errors: Vec<BggEnrichError>,
}
