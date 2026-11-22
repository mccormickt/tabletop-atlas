use anyhow::{Context, Result, bail};
use rig_agent::AgentBuilder;
use rig_agent::agent::{AgentHook, HookContext, InvalidToolCallAction, InvalidToolCallContext};
use rig_agent::completion::PromptError;
use rig_core::completion::Message;
use rig_core::driver::DynModel;
use rig_core::operation::Completion;
use rig_core::tool::builtin::ThinkTool;

use crate::agents::RulesSearchToolkit;
use crate::models::{EmbeddingSearchResult, EmbeddingSourceType, MessageRole};

/// Maximum model turns per chat request (supports multi-hop search + think + final answer).
const MAX_AGENT_TURNS: usize = 8;
/// Token budget for assistant completions (large enough for reasoning-capable models).
const MAX_OUTPUT_TOKENS: u64 = 2048;
/// Maximum times an unknown/disallowed tool call may be retried with corrective feedback.
const MAX_INVALID_TOOL_CALL_RETRIES: usize = 2;
/// Maximum consecutive malformed tool calls before aborting the run.
const MAX_CONSECUTIVE_MALFORMED_TOOL_CALLS: usize = 2;

fn build_rules_preamble(include_house_rules: bool) -> String {
    if include_house_rules {
        "\
You are a helpful board-game rules expert. \
You have tools to search this game's rulebook and house rules, plus a `think` tool for reasoning.\n\n\
Workflow:\n\
1. Before answering any rules question, search the game's rules with `search_rules` (semantic search) \
and/or `keyword_search_rules` (exact card names, numbers, tiebreakers, or specific terms).\n\
2. Check active house rules with `get_house_rules` when they might modify or override the official rules.\n\
3. For complex or multi-part questions, use the `think` tool between searches to check whether you have \
found every relevant rule, exception, or house-rule override, and issue follow-up searches if anything is missing.\n\
4. Ground your final answer strictly in the retrieved rules. Clearly distinguish Official Rules from House Rules. \
If the rules do not contain enough information to answer the question, say so honestly.\n\
5. Be concise, practical, and use concrete examples when helpful."
            .to_string()
    } else {
        "\
You are a helpful board-game rules expert. \
You have tools to search this game's official rulebook, plus a `think` tool for reasoning.\n\n\
Workflow:\n\
1. Before answering any rules question, search the game's rules with `search_rules` (semantic search) \
and/or `keyword_search_rules` (exact card names, numbers, tiebreakers, or specific terms).\n\
2. For complex or multi-part questions, use the `think` tool between searches to check whether you have \
found every relevant rule or exception, and issue follow-up searches if anything is missing.\n\
3. Ground your final answer strictly in the retrieved official rules. \
If the rules do not contain enough information to answer the question, say so honestly.\n\
4. Be concise, practical, and use concrete examples when helpful."
            .to_string()
    }
}

const SYNTHESIS_PREAMBLE: &str = "\
You are a helpful board-game rules expert. \
You have finished searching the rules and now must provide a concise, grounded final answer \
using only the provided rule excerpts (do not request additional tools). \
Clearly distinguish Official Rules from House Rules when both are present. \
If the provided excerpts do not contain enough information to answer the question, say so honestly.";

/// Recovery hook that steers local/smaller models back onto valid tool names when they
/// hallucinate a tool name or emit malformed arguments.
struct RulesAgentHook;

impl AgentHook for RulesAgentHook {
    async fn on_invalid_tool_call(
        &self,
        _ctx: &HookContext,
        event: &InvalidToolCallContext,
    ) -> Option<InvalidToolCallAction> {
        let valid_tools = if event.allowed_tools.is_empty() {
            &event.available_tools
        } else {
            &event.allowed_tools
        };
        Some(InvalidToolCallAction::retry(format!(
            "Tool call `{}` was invalid ({:?}). Retry using only one of the available tools: {}",
            event.tool_name,
            event.reason,
            valid_tools.join(", ")
        )))
    }
}

/// Agentic rules assistant powered by Rig's multi-turn [`Agent`](rig_agent::Agent) runtime.
///
/// Instead of single-shot automatic RAG injection, equips the LLM with read-only search
/// tools (`search_rules`, `keyword_search_rules`, and optionally `get_house_rules`) alongside
/// Rig's built-in `think` scratchpad tool so it can decompose multi-hop questions, run
/// targeted follow-up lookups, and verify rule interactions before answering.
pub struct RulesChatAgent {
    model: DynModel<Completion>,
}

impl RulesChatAgent {
    pub fn new(model: impl Into<DynModel<Completion>>) -> Self {
        Self {
            model: model.into(),
        }
    }

    /// Answer a user question using agentic search tools and return both the final answer
    /// and the deduplicated rule excerpts consulted during the agent's search turns.
    ///
    /// `chat_history` — prior messages, oldest-first, pre-truncated by caller.
    pub async fn answer(
        &self,
        question: &str,
        toolkit: RulesSearchToolkit,
        chat_history: &[(MessageRole, String)],
    ) -> Result<(String, Vec<EmbeddingSearchResult>)> {
        let sources = toolkit.sources.clone();
        let include_house_rules = toolkit.get_house_rules.is_some();

        let mut builder = AgentBuilder::new(self.model.clone())
            .name("rules-chat")
            .preamble(build_rules_preamble(include_house_rules))
            .tool(ThinkTool)
            .tool(toolkit.search_rules)
            .tool(toolkit.keyword_search_rules)
            .default_max_turns(MAX_AGENT_TURNS)
            .temperature(0.7)
            .max_tokens(MAX_OUTPUT_TOKENS);

        if let Some(house_rules_tool) = toolkit.get_house_rules {
            builder = builder.tool(house_rules_tool);
        }

        let agent = builder.build();

        let history: Vec<Message> = chat_history
            .iter()
            .map(|(role, content)| match role {
                MessageRole::Assistant => Message::assistant(content),
                // Map system-role messages from the DB to user since system content is in the preamble.
                MessageRole::User | MessageRole::System => Message::user(content),
            })
            .collect();

        let mut runner = agent
            .prompt(question)
            .add_hook(RulesAgentHook)
            .max_invalid_tool_call_retries(MAX_INVALID_TOOL_CALL_RETRIES)
            .max_consecutive_malformed_tool_calls(MAX_CONSECUTIVE_MALFORMED_TOOL_CALLS);
        if !history.is_empty() {
            runner = runner.history(history.clone());
        }

        let (output, collected_sources) = match runner.await {
            Ok(response) => (response.output(), sources.take()),
            Err(PromptError::MaxTurns { .. }) => {
                let collected = sources.take();
                let synthesized = self
                    .synthesize_from_sources(question, &collected, history)
                    .await
                    .context("Failed to synthesize answer after reaching max agent turns")?;
                (synthesized, collected)
            }
            Err(err) => return Err(err).context("Failed to run rules chat agent"),
        };

        if output.trim().is_empty() {
            bail!("Rules chat agent returned an empty response");
        }

        Ok((output, collected_sources))
    }

    /// Fallback single-turn, tool-less synthesis invoked when the multi-turn agent
    /// exhausts `MAX_AGENT_TURNS` while searching.
    async fn synthesize_from_sources(
        &self,
        question: &str,
        collected_sources: &[EmbeddingSearchResult],
        history: Vec<Message>,
    ) -> Result<String> {
        let mut builder = AgentBuilder::new(self.model.clone())
            .name("rules-chat-synthesis")
            .preamble(SYNTHESIS_PREAMBLE)
            .temperature(0.7)
            .max_tokens(MAX_OUTPUT_TOKENS);

        if collected_sources.is_empty() {
            builder = builder.context("No rule excerpts were retrieved during search.");
        } else {
            for source in collected_sources {
                let label = match source.source_type {
                    EmbeddingSourceType::HouseRule => "House Rule",
                    EmbeddingSourceType::RulesPdf => "Official Rule",
                };
                builder = builder.context(format!("[{label}] {}", source.chunk_text));
            }
        }

        let synth_agent = builder.build();
        let mut runner = synth_agent.prompt(question);
        if !history.is_empty() {
            runner = runner.history(history);
        }

        let response = runner.await?;
        Ok(response.output())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::models::{CreateEmbeddingRequest, EmbeddingSourceType};
    use rig_core::test_utils::{MockCompletionModel, MockEmbeddings, MockTurn, sent_documents};
    use rusqlite::{Connection, ffi::sqlite3_auto_extension};
    use serde_json::json;
    use sqlite_vec::sqlite3_vec_init;

    async fn setup_test_db() -> (Database, i64) {
        unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute::<
                *const (),
                unsafe extern "C" fn(
                    *mut rusqlite::ffi::sqlite3,
                    *mut *mut std::os::raw::c_char,
                    *const rusqlite::ffi::sqlite3_api_routines,
                ) -> std::os::raw::c_int,
            >(sqlite3_vec_init as *const ())));
        }

        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE master_games (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL
            );
            CREATE TABLE house_rules (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id INTEGER NOT NULL,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                category TEXT,
                is_active BOOLEAN NOT NULL DEFAULT TRUE,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE embeddings (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id INTEGER NOT NULL,
                chunk_text TEXT NOT NULL,
                chunk_index INTEGER NOT NULL,
                source_type TEXT NOT NULL DEFAULT 'rules_pdf',
                source_id INTEGER,
                metadata TEXT,
                created_at TEXT NOT NULL
            );
            CREATE VIRTUAL TABLE vec_embeddings USING vec0(
                embedding_vector float[10]
            );
            INSERT INTO master_games (id, name) VALUES (1, 'Catan');
            INSERT INTO house_rules (game_id, title, description, category, is_active, created_at, updated_at)
            VALUES (1, 'Friendly Robber', 'Do not move the robber onto players with 2 or fewer victory points.', 'Robber', 1, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z');
            "#,
        )
        .unwrap();

        let db = Database::new(conn);
        let mock_vec = vec![0.0_f32, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9];

        crate::db::embeddings::create_embeddings_batch(
            &db,
            vec![
                CreateEmbeddingRequest {
                    game_id: 1,
                    chunk_text: "On your turn, roll two dice to produce resources for adjacent settlements.".to_string(),
                    embedding: mock_vec.clone(),
                    chunk_index: 0,
                    source_type: EmbeddingSourceType::RulesPdf,
                    source_id: None,
                    metadata: None,
                },
                CreateEmbeddingRequest {
                    game_id: 1,
                    chunk_text: "Rolling a 7 activates the Robber: players with more than 7 cards discard half.".to_string(),
                    embedding: mock_vec,
                    chunk_index: 1,
                    source_type: EmbeddingSourceType::RulesPdf,
                    source_id: None,
                    metadata: None,
                },
            ],
        )
        .await
        .unwrap();

        (db, 1)
    }

    #[tokio::test]
    async fn test_agentic_multi_turn_search_think_and_house_rules() {
        let (db, game_id) = setup_test_db().await;
        let mock_model = MockCompletionModel::from_turns([
            MockTurn::tool_call(
                "call_1",
                "search_rules",
                json!({ "query": "rolling a 7 robber", "limit": 5 }),
            ),
            MockTurn::tool_call(
                "call_2",
                "think",
                json!({ "thought": "Official rules explain discarding on a 7; let me check exact keyword matches and active house rules." }),
            ),
            MockTurn::tool_call(
                "call_3",
                "keyword_search_rules",
                json!({ "keywords": "Robber discard" }),
            ),
            MockTurn::tool_call("call_4", "get_house_rules", json!({})),
            MockTurn::text(
                "Official rule: rolling a 7 activates the Robber and players with >7 cards discard half. \
                 House rule (Friendly Robber): do not move the robber onto players with 2 or fewer VP.",
            ),
        ]);

        let agent = RulesChatAgent::new(mock_model.clone());
        let toolkit = RulesSearchToolkit::new(MockEmbeddings::model().erase(), db, game_id, true);

        let (answer, sources) = agent
            .answer("What happens when I roll a 7?", toolkit, &[])
            .await
            .unwrap();

        assert!(answer.contains("Friendly Robber"));
        assert_eq!(mock_model.request_count(), 5);

        // First request should advertise all 4 tools: think, search_rules, keyword_search_rules, get_house_rules
        let requests = mock_model.requests();
        let tool_names: Vec<&str> = requests[0].tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"think"));
        assert!(tool_names.contains(&"search_rules"));
        assert!(tool_names.contains(&"keyword_search_rules"));
        assert!(tool_names.contains(&"get_house_rules"));
        let preamble_text = format!("{:?}", requests[0].chat_history[0]);
        assert!(preamble_text.contains("get_house_rules"));

        // Retrieved sources are deduplicated across search_rules + keyword_search_rules
        assert_eq!(sources.len(), 2);
    }

    #[tokio::test]
    async fn test_house_rules_tool_omitted_when_disabled_and_history_forwarded() {
        let (db, game_id) = setup_test_db().await;
        let mock_model = MockCompletionModel::text("Roll two dice at the start of your turn.");

        let agent = RulesChatAgent::new(mock_model.clone());
        let toolkit = RulesSearchToolkit::new(MockEmbeddings::model().erase(), db, game_id, false);
        let history = vec![
            (MessageRole::User, "How does a turn start?".to_string()),
            (MessageRole::Assistant, "By rolling two dice.".to_string()),
        ];

        let (answer, sources) = agent
            .answer("Can you confirm?", toolkit, &history)
            .await
            .unwrap();

        assert_eq!(answer, "Roll two dice at the start of your turn.");
        assert!(sources.is_empty());

        let requests = mock_model.requests();
        assert_eq!(requests.len(), 1);
        let tool_names: Vec<&str> = requests[0].tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"think"));
        assert!(tool_names.contains(&"search_rules"));
        assert!(tool_names.contains(&"keyword_search_rules"));
        assert!(!tool_names.contains(&"get_house_rules"));
        // Preamble must not mention get_house_rules when house rules are disabled
        let preamble_text = format!("{:?}", requests[0].chat_history[0]);
        assert!(!preamble_text.contains("get_house_rules"));
        // System preamble + 2 prior messages + current prompt = 4 messages
        assert_eq!(requests[0].chat_history.len(), 4);
    }

    #[tokio::test]
    async fn test_invalid_tool_call_retries_and_recovers() {
        let (db, game_id) = setup_test_db().await;
        let mock_model = MockCompletionModel::from_turns([
            MockTurn::tool_call("bad_call", "hallucinated_tool", json!({})),
            MockTurn::tool_call(
                "good_call",
                "keyword_search_rules",
                json!({ "keywords": "Robber" }),
            ),
            MockTurn::text("Rolling a 7 activates the Robber."),
        ]);

        let agent = RulesChatAgent::new(mock_model.clone());
        let toolkit = RulesSearchToolkit::new(MockEmbeddings::model().erase(), db, game_id, false);

        let (answer, sources) = agent
            .answer("What activates the robber?", toolkit, &[])
            .await
            .unwrap();

        assert!(answer.contains("Robber"));
        assert_eq!(sources.len(), 1);
        assert_eq!(mock_model.request_count(), 3);
    }

    #[tokio::test]
    async fn test_max_turns_falls_back_to_toolless_synthesis() {
        let (db, game_id) = setup_test_db().await;
        // Exhaust MAX_AGENT_TURNS (8) with tool calls, then answer on the tool-less synthesis call.
        let mut turns: Vec<MockTurn> = (0..MAX_AGENT_TURNS)
            .map(|i| {
                MockTurn::tool_call(
                    format!("call_{i}"),
                    "keyword_search_rules",
                    json!({ "keywords": "Robber" }),
                )
            })
            .collect();
        turns.push(MockTurn::text(
            "Synthesized answer: rolling a 7 activates the Robber and triggers discard.",
        ));

        let mock_model = MockCompletionModel::from_turns(turns);
        let agent = RulesChatAgent::new(mock_model.clone());
        let toolkit = RulesSearchToolkit::new(MockEmbeddings::model().erase(), db, game_id, false);

        let (answer, sources) = agent
            .answer("What happens on a 7?", toolkit, &[])
            .await
            .unwrap();

        assert!(answer.contains("Synthesized answer"));
        assert_eq!(sources.len(), 1);

        let requests = mock_model.requests();
        assert_eq!(requests.len(), MAX_AGENT_TURNS + 1);
        let synthesis_req = requests.last().unwrap();
        assert!(synthesis_req.tools.is_empty());
        let docs = sent_documents(synthesis_req);
        assert_eq!(docs.len(), 1);
        assert!(docs[0].1.contains("Robber"));
    }

    #[tokio::test]
    async fn test_empty_response_is_rejected() {
        let (db, game_id) = setup_test_db().await;
        let mock_model = MockCompletionModel::text("   \n\t  ");

        let agent = RulesChatAgent::new(mock_model);
        let toolkit = RulesSearchToolkit::new(MockEmbeddings::model().erase(), db, game_id, false);

        let err = agent
            .answer("How do I win?", toolkit, &[])
            .await
            .unwrap_err();
        assert!(err.to_string().contains("empty response"));
    }
}
