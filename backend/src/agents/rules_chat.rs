use anyhow::{Context, Result};
use rig::completion::{Chat, Prompt};
use rig::message::Message;
use rig::prelude::CompletionClient;
use rig::providers::ollama;

/// A per-request context chunk retrieved from the vector store.
pub struct ContextChunk {
    pub text: String,
    pub source_label: &'static str,
}

/// Agent for answering board-game rules questions using retrieved context.
///
/// Builds a fresh Rig Agent per request so the preamble and static context
/// documents match the specific game / session being queried.
pub struct RulesChatAgent {
    client: ollama::Client,
    model: String,
}

impl RulesChatAgent {
    pub fn new(client: ollama::Client, model: String) -> Self {
        Self { client, model }
    }

    /// Answer a user question given retrieved context chunks and conversation history.
    ///
    /// `context_chunks` — rule chunks returned by similarity search (may be empty).
    /// `chat_history`   — prior (role, content) pairs from the session, oldest-first.
    pub async fn answer(
        &self,
        question: &str,
        context_chunks: &[ContextChunk],
        chat_history: &[(String, String)],
    ) -> Result<String> {
        let preamble = Self::build_preamble(context_chunks, chat_history);

        let mut builder = self
            .client
            .agent(&self.model)
            .preamble(&preamble)
            .temperature(0.7)
            .max_tokens(512);

        // Add each context chunk as a Rig context document so the framework
        // includes them in the completion request alongside the preamble.
        for chunk in context_chunks {
            builder = builder.context(&chunk.text);
        }

        let agent = builder.build();

        // If there is conversation history, use `chat()` so the model sees it.
        if chat_history.is_empty() {
            let response: String = agent
                .prompt(question)
                .await
                .context("Failed to generate chat completion")?;
            Ok(response)
        } else {
            let history: Vec<Message> = chat_history
                .iter()
                .map(|(role, content)| match role.as_str() {
                    "assistant" => Message::assistant(content),
                    _ => Message::user(content),
                })
                .collect();

            let response: String = agent
                .chat(question, history)
                .await
                .context("Failed to generate chat completion")?;
            Ok(response)
        }
    }

    /// Build the system preamble incorporating context and history summaries.
    fn build_preamble(
        context_chunks: &[ContextChunk],
        chat_history: &[(String, String)],
    ) -> String {
        let context_text = if context_chunks.is_empty() {
            "No specific rules found for this question.".to_string()
        } else {
            context_chunks
                .iter()
                .map(|c| format!("[{}]: {}", c.source_label, c.text))
                .collect::<Vec<_>>()
                .join("\n\n")
        };

        let history_text: String = chat_history
            .iter()
            .rev()
            .take(6) // last 3 exchanges
            .rev()
            .map(|(role, content)| format!("{}: {}", role, content))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "You are a helpful assistant that explains board game rules. \
             Use the following game rules to answer questions accurately and clearly. \
             If the rules don't contain enough information to answer the question, say so honestly.\n\n\
             Game Rules Context:\n{}\n\n\
             Conversation History:\n{}\n\n\
             Instructions:\n\
             - Answer based on the provided rules context\n\
             - Be concise but thorough\n\
             - If rules are unclear or missing, acknowledge this\n\
             - Use examples when helpful\n\
             - Focus on practical gameplay guidance",
            context_text, history_text,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_preamble_with_context() {
        let chunks = vec![
            ContextChunk {
                text: "Players take turns clockwise.".into(),
                source_label: "Official Rule",
            },
            ContextChunk {
                text: "House rule: Skip every other turn.".into(),
                source_label: "House Rule",
            },
        ];
        let history: Vec<(String, String)> = vec![];

        let preamble = RulesChatAgent::build_preamble(&chunks, &history);
        assert!(preamble.contains("[Official Rule]: Players take turns clockwise."));
        assert!(preamble.contains("[House Rule]: House rule: Skip every other turn."));
    }

    #[test]
    fn test_build_preamble_empty_context() {
        let preamble = RulesChatAgent::build_preamble(&[], &[]);
        assert!(preamble.contains("No specific rules found"));
    }

    #[test]
    fn test_build_preamble_with_history() {
        let history = vec![
            ("user".to_string(), "How do I win?".to_string()),
            (
                "assistant".to_string(),
                "Collect 5 victory points.".to_string(),
            ),
        ];

        let preamble = RulesChatAgent::build_preamble(&[], &history);
        assert!(preamble.contains("user: How do I win?"));
        assert!(preamble.contains("assistant: Collect 5 victory points."));
    }
}
