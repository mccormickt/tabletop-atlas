//! BoardGameGeek XML API2 client for fetching game data.
//!
//! This module provides functionality to fetch and parse game data from the
//! BoardGameGeek XML API2 endpoint.

use dropshot_schemars::JsonSchema;
use quick_xml::Reader;
use quick_xml::events::Event;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// BGG API2 base URL for fetching game details
const BGG_API_BASE: &str = "https://boardgamegeek.com/xmlapi2/thing";

/// Maximum number of game IDs per request (BGG API limit)
const MAX_IDS_PER_REQUEST: usize = 20;

/// Minimum interval between API requests to avoid rate limiting
const MIN_REQUEST_INTERVAL: Duration = Duration::from_millis(500);

/// Parsed game data from BGG API
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(crate = "dropshot_schemars")]
pub struct BggGameData {
    pub bgg_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub year_published: Option<i32>,
    pub min_players: Option<i32>,
    pub max_players: Option<i32>,
    pub play_time_minutes: Option<i32>,
    pub complexity_rating: Option<f64>,
    pub thumbnail_url: Option<String>,
    pub image_url: Option<String>,
}

/// Error type for BGG API operations
#[derive(Debug)]
pub enum BggApiError {
    NetworkError(String),
    RateLimited,
    ParseError(String),
    GameNotFound(i32),
    InvalidResponse(String),
}

impl std::fmt::Display for BggApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BggApiError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            BggApiError::RateLimited => write!(f, "BGG API rate limited"),
            BggApiError::ParseError(msg) => write!(f, "XML parse error: {}", msg),
            BggApiError::GameNotFound(id) => write!(f, "Game {} not found on BGG", id),
            BggApiError::InvalidResponse(msg) => write!(f, "Invalid API response: {}", msg),
        }
    }
}

impl std::error::Error for BggApiError {}

/// BGG API client with rate limiting
pub struct BggClient {
    client: reqwest::Client,
    last_request: Mutex<Instant>,
    api_token: Option<String>,
}

impl BggClient {
    /// Create a new BGG API client
    /// Reads BGG_API_TOKEN from environment for authentication
    pub fn new() -> Self {
        let api_token = std::env::var("BGG_API_TOKEN").ok();
        Self {
            client: reqwest::Client::builder()
                .user_agent("TabletopAtlas/1.0")
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            last_request: Mutex::new(Instant::now() - MIN_REQUEST_INTERVAL),
            api_token,
        }
    }

    /// Fetch a single game from BGG API
    pub async fn fetch_game(&self, bgg_id: i32) -> Result<BggGameData, BggApiError> {
        let games = self.fetch_games(&[bgg_id]).await?;
        games
            .into_iter()
            .next()
            .ok_or(BggApiError::GameNotFound(bgg_id))
    }

    /// Fetch multiple games from BGG API (handles batching)
    pub async fn fetch_games(&self, bgg_ids: &[i32]) -> Result<Vec<BggGameData>, BggApiError> {
        if bgg_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut all_results = Vec::new();

        // Process in batches of MAX_IDS_PER_REQUEST
        for chunk in bgg_ids.chunks(MAX_IDS_PER_REQUEST) {
            let ids_str = chunk
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(",");

            let url = format!("{}?id={}&stats=1", BGG_API_BASE, ids_str);
            let xml = self.rate_limited_fetch(&url).await?;
            let games = parse_bgg_xml(&xml)?;
            all_results.extend(games);
        }

        Ok(all_results)
    }

    /// Perform a rate-limited HTTP fetch
    async fn rate_limited_fetch(&self, url: &str) -> Result<String, BggApiError> {
        // Enforce rate limiting
        {
            let mut last = self.last_request.lock().await;
            let elapsed = last.elapsed();
            if elapsed < MIN_REQUEST_INTERVAL {
                tokio::time::sleep(MIN_REQUEST_INTERVAL - elapsed).await;
            }
            *last = Instant::now();
        }

        // Build request with optional authorization
        let mut request = self.client.get(url);
        if let Some(ref token) = self.api_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response: reqwest::Response = request
            .send()
            .await
            .map_err(|e| BggApiError::NetworkError(e.to_string()))?;

        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(BggApiError::RateLimited);
        }

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            if self.api_token.is_none() {
                return Err(BggApiError::InvalidResponse(
                    "BGG API requires authentication. Set BGG_API_TOKEN environment variable. Register at https://boardgamegeek.com/applications".to_string()
                ));
            }
            return Err(BggApiError::InvalidResponse(
                "BGG API authentication failed. Check your BGG_API_TOKEN.".to_string(),
            ));
        }

        if !response.status().is_success() {
            return Err(BggApiError::InvalidResponse(format!(
                "HTTP {}",
                response.status()
            )));
        }

        response
            .text()
            .await
            .map_err(|e| BggApiError::NetworkError(e.to_string()))
    }
}

impl Default for BggClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse BGG XML API response into BggGameData structs
fn parse_bgg_xml(xml: &str) -> Result<Vec<BggGameData>, BggApiError> {
    let mut reader = Reader::from_str(xml);

    let mut games = Vec::new();
    let mut current_game: Option<BggGameDataBuilder> = None;
    let mut in_statistics = false;
    let mut in_ratings = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let tag_name = e.name();

                match tag_name.as_ref() {
                    "item" => {
                        // Start a new game
                        let mut builder = BggGameDataBuilder::default();
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == "id" {
                                builder.bgg_id = attr.value.parse().ok();
                            }
                        }
                        current_game = Some(builder);
                    }
                    "name" if current_game.is_some() => {
                        let mut is_primary = false;
                        let mut name_value = String::new();
                        for attr in e.attributes().flatten() {
                            let key = attr.key.as_ref();
                            if key == "type" && attr.value.as_ref() == "primary" {
                                is_primary = true;
                            }
                            if key == "value" {
                                name_value = attr
                                    .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                                    .map_err(|err| BggApiError::ParseError(err.to_string()))?
                                    .into_owned();
                            }
                        }
                        if is_primary && let Some(ref mut game) = current_game {
                            game.name = Some(name_value);
                        }
                    }
                    "yearpublished" if current_game.is_some() => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == "value"
                                && let Some(ref mut game) = current_game
                            {
                                game.year_published = attr.value.parse().ok();
                            }
                        }
                    }
                    "minplayers" if current_game.is_some() => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == "value"
                                && let Some(ref mut game) = current_game
                            {
                                game.min_players = attr.value.parse().ok();
                            }
                        }
                    }
                    "maxplayers" if current_game.is_some() => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == "value"
                                && let Some(ref mut game) = current_game
                            {
                                game.max_players = attr.value.parse().ok();
                            }
                        }
                    }
                    "playingtime" if current_game.is_some() => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == "value"
                                && let Some(ref mut game) = current_game
                            {
                                game.play_time_minutes = attr.value.parse().ok();
                            }
                        }
                    }
                    "thumbnail" | "image" | "description" if current_game.is_some() => {
                        if let Some(ref mut game) = current_game {
                            game.pending_text.clear();
                        }
                    }
                    "statistics" => {
                        in_statistics = true;
                    }
                    "ratings" if in_statistics => {
                        in_ratings = true;
                    }
                    "averageweight" if in_ratings && current_game.is_some() => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == "value"
                                && let Some(ref mut game) = current_game
                            {
                                game.complexity_rating = attr.value.parse().ok();
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(e)) => {
                if let Some(ref mut game) = current_game {
                    game.pending_text
                        .push_str(&e.xml_content(quick_xml::XmlVersion::Implicit1_0));
                }
            }
            Ok(Event::CData(e)) => {
                if let Some(ref mut game) = current_game {
                    game.pending_text.push_str(e.as_ref());
                }
            }
            Ok(Event::GeneralRef(e)) => {
                if let Some(ref mut game) = current_game {
                    if let Ok(Some(ch)) = e.resolve_char_ref() {
                        game.pending_text.push(ch);
                    } else if let Some(resolved) = quick_xml::escape::resolve_predefined_entity(&e)
                    {
                        game.pending_text.push_str(resolved);
                    } else {
                        game.pending_text.push('&');
                        game.pending_text.push_str(&e);
                        game.pending_text.push(';');
                    }
                }
            }
            Ok(Event::End(e)) => match e.name().as_ref() {
                "item" => {
                    if let Some(builder) = current_game.take()
                        && let Some(game) = builder.build()
                    {
                        games.push(game);
                    }
                }
                "statistics" => {
                    in_statistics = false;
                }
                "ratings" => {
                    in_ratings = false;
                }
                "thumbnail" => {
                    if let Some(ref mut game) = current_game {
                        let text = std::mem::take(&mut game.pending_text);
                        let trimmed = text.trim();
                        if !trimmed.is_empty() {
                            game.thumbnail_url = Some(trimmed.to_string());
                        }
                    }
                }
                "image" => {
                    if let Some(ref mut game) = current_game {
                        let text = std::mem::take(&mut game.pending_text);
                        let trimmed = text.trim();
                        if !trimmed.is_empty() {
                            game.image_url = Some(trimmed.to_string());
                        }
                    }
                }
                "description" => {
                    if let Some(ref mut game) = current_game {
                        let text = std::mem::take(&mut game.pending_text);
                        let trimmed = text.trim();
                        if !trimmed.is_empty() {
                            // Clean up HTML entities (including double-encoded BGG entities)
                            let cleaned = html_entity_decode(trimmed);
                            game.description = Some(cleaned);
                        }
                    }
                }
                _ => {
                    if let Some(ref mut game) = current_game {
                        game.pending_text.clear();
                    }
                }
            },
            Ok(Event::Eof) => break,
            Err(e) => return Err(BggApiError::ParseError(e.to_string())),
            _ => {}
        }
    }

    Ok(games)
}

/// Builder for constructing BggGameData during parsing
#[derive(Default)]
struct BggGameDataBuilder {
    bgg_id: Option<i32>,
    name: Option<String>,
    description: Option<String>,
    year_published: Option<i32>,
    min_players: Option<i32>,
    max_players: Option<i32>,
    play_time_minutes: Option<i32>,
    complexity_rating: Option<f64>,
    thumbnail_url: Option<String>,
    image_url: Option<String>,
    pending_text: String,
}

impl BggGameDataBuilder {
    fn build(self) -> Option<BggGameData> {
        Some(BggGameData {
            bgg_id: self.bgg_id?,
            name: self.name?,
            description: self.description,
            year_published: self.year_published,
            min_players: self.min_players,
            max_players: self.max_players,
            play_time_minutes: self.play_time_minutes,
            complexity_rating: self.complexity_rating,
            thumbnail_url: self.thumbnail_url,
            image_url: self.image_url,
        })
    }
}

/// Decode common HTML entities in BGG descriptions
fn html_entity_decode(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#10;", "\n")
        .replace("&mdash;", "\u{2014}")
        .replace("&ndash;", "\u{2013}")
        .replace("&rsquo;", "\u{2019}")
        .replace("&lsquo;", "\u{2018}")
        .replace("&rdquo;", "\u{201D}")
        .replace("&ldquo;", "\u{201C}")
        .replace("&nbsp;", " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_XML: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<items termsofuse="https://boardgamegeek.com/xmlapi/termsofuse">
    <item type="boardgame" id="224517">
        <thumbnail>https://cf.geekdo-images.com/thumb.jpg</thumbnail>
        <image>https://cf.geekdo-images.com/image.jpg</image>
        <name type="primary" sortindex="1" value="Brass: Birmingham"/>
        <name type="alternate" sortindex="1" value="Brass Birmingham"/>
        <description>Brass: Birmingham is an economic strategy game.</description>
        <yearpublished value="2018"/>
        <minplayers value="2"/>
        <maxplayers value="4"/>
        <playingtime value="120"/>
        <minplaytime value="60"/>
        <maxplaytime value="120"/>
        <minage value="14"/>
        <statistics page="1">
            <ratings>
                <usersrated value="50000"/>
                <average value="8.5"/>
                <bayesaverage value="8.4"/>
                <stddev value="1.2"/>
                <median value="0"/>
                <owned value="60000"/>
                <trading value="500"/>
                <wanting value="1000"/>
                <wishing value="5000"/>
                <numcomments value="5000"/>
                <numweights value="3000"/>
                <averageweight value="3.89"/>
            </ratings>
        </statistics>
    </item>
</items>"#;

    #[test]
    fn test_parse_bgg_xml() {
        let games = parse_bgg_xml(SAMPLE_XML).unwrap();
        assert_eq!(games.len(), 1);

        let game = &games[0];
        assert_eq!(game.bgg_id, 224517);
        assert_eq!(game.name, "Brass: Birmingham");
        assert_eq!(game.year_published, Some(2018));
        assert_eq!(game.min_players, Some(2));
        assert_eq!(game.max_players, Some(4));
        assert_eq!(game.play_time_minutes, Some(120));
        assert!(game.complexity_rating.is_some());
        assert!((game.complexity_rating.unwrap() - 3.89).abs() < 0.01);
        assert!(game.description.is_some());
        assert!(game.thumbnail_url.is_some());
        assert!(game.image_url.is_some());
    }

    #[test]
    fn test_html_entity_decode() {
        assert_eq!(html_entity_decode("&amp;"), "&");
        assert_eq!(html_entity_decode("&lt;tag&gt;"), "<tag>");
        assert_eq!(html_entity_decode("line1&#10;line2"), "line1\nline2");
    }

    #[test]
    fn test_parse_bgg_xml_preserves_and_decodes_entities() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<items>
    <item type="boardgame" id="1234">
        <thumbnail>https://example.com/thumb.jpg?a=1&amp;b=2</thumbnail>
        <name type="primary" value="Dungeons &amp; Dragons: Player&#039;s &quot;Guide&quot;"/>
        <description>  Economic &amp; strategy game &#10; Second line &amp;mdash; with &quot;quotes&quot; &amp; &apos;apos&apos;.  </description>
    </item>
</items>"#;

        let games = parse_bgg_xml(xml).unwrap();
        assert_eq!(games.len(), 1);
        let game = &games[0];
        assert_eq!(game.name, "Dungeons & Dragons: Player's \"Guide\"");
        assert_eq!(
            game.thumbnail_url.as_deref(),
            Some("https://example.com/thumb.jpg?a=1&b=2")
        );
        assert_eq!(
            game.description.as_deref(),
            Some("Economic & strategy game \n Second line \u{2014} with \"quotes\" & 'apos'.")
        );
    }
}
