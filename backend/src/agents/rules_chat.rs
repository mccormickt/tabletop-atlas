use std::collections::HashMap;

use anyhow::{Context, Result};
use rig_core::DynModel;
use rig_core::completion::{AssistantContent, CompletionRequest, Document, Message};
use rig_core::operation::Completion;
use rig_core::vector_store::{VectorSearchRequest, VectorStoreIndex};

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
pub struct RulesChatAgent {
    model: DynModel<Completion>,
}

impl RulesChatAgent {
    pub fn new(model: impl Into<DynModel<Completion>>) -> Self {
        Self {
            model: model.into(),
        }
    }

    /// Answer a user question. The `index` performs automatic context retrieval.
    ///
    /// `chat_history` — prior messages, oldest-first, pre-truncated by caller.
    pub async fn answer<I: VectorStoreIndex>(
        &self,
        question: &str,
        index: I,
        chat_history: &[(MessageRole, String)],
    ) -> Result<String> {
        let search_req = VectorSearchRequest::<I::Filter>::builder()
            .query(question)
            .samples(10)
            .build();

        let retrieved = index
            .top_n::<serde_json::Value>(search_req)
            .await
            .context("Failed to retrieve rules context")?;

        let documents = retrieved
            .into_iter()
            .map(|r| {
                let text = serde_json::to_string(&r.document)
                    .context("Failed to serialize retrieved context document")?;
                Ok(Document {
                    id: r.id,
                    text,
                    additional_props: HashMap::new(),
                })
            })
            .collect::<Result<Vec<_>>>()?;

        let mut request = CompletionRequest::new(question)
            .preamble(RULES_PREAMBLE)
            .documents(documents)
            .temperature(0.7)
            .max_tokens(512);

        if !chat_history.is_empty() {
            let history: Vec<Message> = chat_history
                .iter()
                .map(|(role, content)| match role {
                    MessageRole::Assistant => Message::assistant(content),
                    // Map system-role messages from the DB to user since system content is in the preamble.
                    MessageRole::User | MessageRole::System => Message::user(content),
                })
                .collect();
            request = request.messages(history);
        }

        let response = self
            .model
            .call(request)
            .await
            .context("Failed to generate completion response")?;

        Ok(response
            .choice
            .iter()
            .filter_map(AssistantContent::as_text)
            .collect::<Vec<_>>()
            .join(""))
    }
}
