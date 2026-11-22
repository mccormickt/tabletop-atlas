use std::sync::{Arc, Mutex};

use rig_core::completion::ToolAnnotations;
use rig_core::driver::DynModel;
use rig_core::operation::Embedding;
use rig_core::tool::{Tool, ToolContext};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::db::Database;
use crate::models::{EmbeddingSearchResult, EmbeddingSourceType, SimilaritySearchRequest};

const DEFAULT_SEARCH_LIMIT: u32 = 6;
const MAX_SEARCH_LIMIT: u32 = 15;
/// Dampening weight for keyword match ratios so a single-word keyword hit does not
/// outrank stronger semantic vector matches in [`RetrievedSources`].
const KEYWORD_SCORE_WEIGHT: f32 = 0.75;
/// Maximum deduplicated rule excerpts returned to the caller / persisted per turn.
const MAX_CONTEXT_SOURCES: usize = 10;

/// Shared collector recording every rule chunk retrieved by agentic search tools during
/// a single chat turn, deduplicated by embedding ID (keeping the highest score).
#[derive(Clone, Default)]
pub struct RetrievedSources {
    inner: Arc<Mutex<Vec<EmbeddingSearchResult>>>,
}

impl RetrievedSources {
    pub fn record(&self, incoming: Vec<EmbeddingSearchResult>) {
        let mut guard = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        for item in incoming {
            if let Some(existing) = guard.iter_mut().find(|r| r.id == item.id) {
                if item.similarity_score > existing.similarity_score {
                    existing.similarity_score = item.similarity_score;
                }
            } else {
                guard.push(item);
            }
        }
    }

    pub fn take(&self) -> Vec<EmbeddingSearchResult> {
        let mut guard = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut results = std::mem::take(&mut *guard);
        results.sort_by(|a, b| {
            b.similarity_score
                .partial_cmp(&a.similarity_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.id.cmp(&b.id))
        });
        results.truncate(MAX_CONTEXT_SOURCES);
        results
    }
}

/// Per-request bundle of read-only rules search tools and their shared source collector.
pub struct RulesSearchToolkit {
    pub search_rules: SearchRulesTool,
    pub keyword_search_rules: KeywordSearchRulesTool,
    pub get_house_rules: Option<GetHouseRulesTool>,
    pub sources: RetrievedSources,
}

impl RulesSearchToolkit {
    pub fn new(
        model: DynModel<Embedding>,
        db: Database,
        game_id: i64,
        include_house_rules: bool,
    ) -> Self {
        let sources = RetrievedSources::default();
        let search_rules = SearchRulesTool {
            model,
            db: db.clone(),
            game_id,
            include_house_rules,
            sources: sources.clone(),
        };
        let keyword_search_rules = KeywordSearchRulesTool {
            db: db.clone(),
            game_id,
            include_house_rules,
            sources: sources.clone(),
        };
        let get_house_rules = include_house_rules.then(|| GetHouseRulesTool { db, game_id });

        Self {
            search_rules,
            keyword_search_rules,
            get_house_rules,
            sources,
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct RulesToolError(String);

/// Serializable rule excerpt returned to the LLM by search tools.
#[derive(Debug, Serialize)]
pub struct RuleExcerpt {
    pub id: i64,
    pub source: &'static str,
    pub score: f32,
    pub text: String,
}

fn to_excerpts(results: &[EmbeddingSearchResult]) -> Vec<RuleExcerpt> {
    results
        .iter()
        .map(|r| RuleExcerpt {
            id: r.id,
            source: match r.source_type {
                EmbeddingSourceType::HouseRule => "House Rule",
                EmbeddingSourceType::RulesPdf => "Official Rule",
            },
            score: r.similarity_score,
            text: r.chunk_text.clone(),
        })
        .collect()
}

fn clamp_limit(limit: Option<u32>) -> u32 {
    limit
        .unwrap_or(DEFAULT_SEARCH_LIMIT)
        .clamp(1, MAX_SEARCH_LIMIT)
}

fn read_only_annotations() -> ToolAnnotations {
    ToolAnnotations::default()
        .with_read_only(true)
        .with_open_world(false)
}

// ---------------------------------------------------------------------------
// 1. Semantic vector search tool (`search_rules`)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct SearchRulesArgs {
    pub query: String,
    #[serde(default)]
    pub limit: Option<u32>,
}

pub struct SearchRulesTool {
    model: DynModel<Embedding>,
    db: Database,
    game_id: i64,
    include_house_rules: bool,
    sources: RetrievedSources,
}

impl Tool for SearchRulesTool {
    const NAME: &'static str = "search_rules";
    type Args = SearchRulesArgs;
    type Output = Vec<RuleExcerpt>;
    type Error = RulesToolError;

    fn description(&self) -> String {
        "Semantic search over this game's rulebook excerpts (and active house rules when enabled). \
         Use focused natural-language queries for mechanics, turn order, scoring, or concepts. \
         You may call this multiple times with different phrasings for multi-part questions."
            .to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Focused natural-language search query describing the rule or concept to look up."
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of excerpts to return (1-15, default 6).",
                    "minimum": 1,
                    "maximum": 15
                }
            },
            "required": ["query"]
        })
    }

    fn annotations(&self) -> ToolAnnotations {
        read_only_annotations()
    }

    async fn call(
        &self,
        _: &mut ToolContext,
        args: Self::Args,
    ) -> Result<Self::Output, Self::Error> {
        if args.query.trim().is_empty() {
            return Ok(Vec::new());
        }

        let query_embedding = self
            .model
            .embed_text(&args.query)
            .await
            .map_err(|e| RulesToolError(format!("Failed to embed search query: {e}")))?;

        let embedding_vec: Vec<f32> = query_embedding.vec.into_iter().map(|v| v as f32).collect();

        let similarity_request = SimilaritySearchRequest {
            game_id: self.game_id,
            query_embedding: embedding_vec,
            similarity_threshold: 0.0,
            limit: clamp_limit(args.limit),
        };

        let results = crate::db::embeddings::similarity_search_filtered(
            &self.db,
            similarity_request,
            self.include_house_rules,
        )
        .await
        .map_err(|e| RulesToolError(format!("Vector search failed: {e}")))?;

        let excerpts = to_excerpts(&results);
        self.sources.record(results);
        Ok(excerpts)
    }
}

// ---------------------------------------------------------------------------
// 2. Exact / keyword search tool (`keyword_search_rules`)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct KeywordSearchArgs {
    pub keywords: String,
    #[serde(default)]
    pub limit: Option<u32>,
}

pub struct KeywordSearchRulesTool {
    db: Database,
    game_id: i64,
    include_house_rules: bool,
    sources: RetrievedSources,
}

impl Tool for KeywordSearchRulesTool {
    const NAME: &'static str = "keyword_search_rules";
    type Args = KeywordSearchArgs;
    type Output = Vec<RuleExcerpt>;
    type Error = RulesToolError;

    fn description(&self) -> String {
        "Case-insensitive keyword search across this game's rulebook text. \
         Best for exact card names, numbers, special symbols, tiebreaker terms, or specific rule headings \
         that semantic search might rank lower."
            .to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "keywords": {
                    "type": "string",
                    "description": "Exact keywords or terms to match in the rulebook text (e.g. 'robber knight 7' or 'tiebreaker')."
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of matching excerpts to return (1-15, default 6).",
                    "minimum": 1,
                    "maximum": 15
                }
            },
            "required": ["keywords"]
        })
    }

    fn annotations(&self) -> ToolAnnotations {
        read_only_annotations()
    }

    async fn call(
        &self,
        _: &mut ToolContext,
        args: Self::Args,
    ) -> Result<Self::Output, Self::Error> {
        if args.keywords.trim().is_empty() {
            return Ok(Vec::new());
        }

        let mut results = crate::db::embeddings::keyword_search_filtered(
            &self.db,
            self.game_id,
            &args.keywords,
            self.include_house_rules,
            clamp_limit(args.limit),
        )
        .await
        .map_err(|e| RulesToolError(format!("Keyword search failed: {e}")))?;

        for result in &mut results {
            result.similarity_score *= KEYWORD_SCORE_WEIGHT;
        }

        let excerpts = to_excerpts(&results);
        self.sources.record(results);
        Ok(excerpts)
    }
}

// ---------------------------------------------------------------------------
// 3. Active house rules tool (`get_house_rules`)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct EmptyArgs {}

#[derive(Debug, Serialize)]
pub struct HouseRuleSummary {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub category: Option<String>,
}

/// Lists active house rules directly from the `house_rules` table.
///
/// Intentionally does not insert into [`RetrievedSources`]: `RetrievedSources` records
/// `embeddings.id` references persisted in `chat_messages.context_chunks`, whereas
/// `house_rules.id` belongs to a separate primary-key space (active house rules are also
/// embedded in `embeddings` and recorded whenever matched via `search_rules` or
/// `keyword_search_rules`).
pub struct GetHouseRulesTool {
    db: Database,
    game_id: i64,
}

impl Tool for GetHouseRulesTool {
    const NAME: &'static str = "get_house_rules";
    type Args = EmptyArgs;
    type Output = Vec<HouseRuleSummary>;
    type Error = RulesToolError;

    fn description(&self) -> String {
        "List all active custom house rules configured for this game. \
         Call this to check whether any house rules modify or override the official rules."
            .to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {}
        })
    }

    fn annotations(&self) -> ToolAnnotations {
        read_only_annotations()
    }

    async fn call(&self, _: &mut ToolContext, _: Self::Args) -> Result<Self::Output, Self::Error> {
        let rules = crate::db::house_rules::list_house_rules_by_game(&self.db, self.game_id, true)
            .await
            .map_err(|e| RulesToolError(format!("Failed to fetch active house rules: {e}")))?;

        Ok(rules
            .into_iter()
            .map(|r| HouseRuleSummary {
                id: r.id,
                title: r.title,
                description: r.description,
                category: r.category,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rig_core::test_utils::MockEmbeddings;
    use rusqlite::Connection;

    fn make_result(id: i64, score: f32) -> EmbeddingSearchResult {
        EmbeddingSearchResult {
            id,
            chunk_text: format!("Chunk {id}"),
            similarity_score: score,
            source_type: EmbeddingSourceType::RulesPdf,
            source_id: None,
            metadata: None,
        }
    }

    #[test]
    fn test_retrieved_sources_deduplicates_sorts_and_caps() {
        let sources = RetrievedSources::default();
        sources.record(vec![
            make_result(1, 0.4),
            make_result(2, 0.8),
            make_result(1, 0.9), // higher score should replace 0.4
            make_result(2, 0.2), // lower score should be ignored
        ]);
        // Add 12 more distinct items to test MAX_CONTEXT_SOURCES cap
        let extra: Vec<_> = (3..=14)
            .map(|id| make_result(id, 0.5 + (id as f32) * 0.01))
            .collect();
        sources.record(extra);

        let taken = sources.take();
        assert_eq!(taken.len(), MAX_CONTEXT_SOURCES);
        assert_eq!(taken[0].id, 1);
        assert!((taken[0].similarity_score - 0.9).abs() < f32::EPSILON);
        assert_eq!(taken[1].id, 2);
        assert!((taken[1].similarity_score - 0.8).abs() < f32::EPSILON);
        // Verify strictly non-increasing scores
        for window in taken.windows(2) {
            assert!(window[0].similarity_score >= window[1].similarity_score);
        }
        // Subsequent take is empty
        assert!(sources.take().is_empty());
    }

    #[tokio::test]
    async fn test_search_rules_empty_query_returns_empty() {
        let conn = Connection::open_in_memory().unwrap();
        let db = Database::new(conn);
        let toolkit = RulesSearchToolkit::new(MockEmbeddings::model().erase(), db, 1, false);
        let mut ctx = ToolContext::new();

        let results = toolkit
            .search_rules
            .call(
                &mut ctx,
                SearchRulesArgs {
                    query: "   ".to_string(),
                    limit: None,
                },
            )
            .await
            .unwrap();
        assert!(results.is_empty());
    }
}
