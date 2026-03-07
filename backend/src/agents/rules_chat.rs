use anyhow::{Context, Result};
use rig::agent::AgentBuilder;
use rig::completion::{Chat, CompletionModel, Prompt};
use rig::message::Message;
use rig::vector_store::VectorStoreIndexDyn;

use crate::models::MessageRole;

const RULES_PREAMBLE: &str = "\
You are a helpful assistant that explains board game rules. \
The context below contains relevant game rules retrieved for this question. \
Use it to answer accurately and clearly. \
If the rules don't contain enough information to answer the question, say so honestly.\n\n\
Instructions:\n\
- Answer based on the provided rules context\n\
- Be concise but thorough\n\
- If rules are unclear or missing, acknowledge this\n\
- Use examples when helpful\n\
- Focus on practical gameplay guidance\n\
- Context documents have a \"source\" field (\"Official Rule\" or \"House Rule\") and a \"text\" field";

/// Agent for answering board-game rules questions with automatic RAG retrieval.
///
/// Builds a fresh Rig Agent per request. The `dynamic_context()` builder method
/// wires in a `GameRulesIndex` so the agent embeds the query, searches for
/// relevant chunks, and injects them as context automatically.
///
/// Generic over `M: CompletionModel` to allow testing with alternate providers.
pub struct RulesChatAgent<M: CompletionModel> {
    model: M,
}

impl<M: CompletionModel> RulesChatAgent<M> {
    pub fn new(model: M) -> Self {
        Self { model }
    }

    /// Answer a user question. The `index` performs automatic context retrieval.
    ///
    /// `chat_history` — prior messages, oldest-first, pre-truncated by caller.
    pub async fn answer(
        &self,
        question: &str,
        index: impl VectorStoreIndexDyn + Send + Sync + 'static,
        chat_history: &[(MessageRole, String)],
    ) -> Result<String> {
        let agent = AgentBuilder::new(self.model.clone())
            .preamble(RULES_PREAMBLE)
            .dynamic_context(10, index)
            .temperature(0.7)
            .max_tokens(512)
            .build();

        if chat_history.is_empty() {
            agent
                .prompt(question)
                .await
                .context("Failed to prompt agent")
        } else {
            let history: Vec<Message> = chat_history
                .iter()
                .map(|(role, content)| match role {
                    MessageRole::Assistant => Message::assistant(content),
                    // Rig only has User/Assistant variants; system-role messages from
                    // the DB are mapped to user since system content is in the preamble.
                    MessageRole::User | MessageRole::System => Message::user(content),
                })
                .collect();

            agent
                .chat(question, history)
                .await
                .context("Failed to chat with agent")
        }
    }
}
