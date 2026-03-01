use anyhow::{Context, Result};
use rig::client::Nothing;
use rig::completion::{Chat, Prompt};
use rig::message::Message;
use rig::prelude::CompletionClient;
use rig::providers::ollama;
use serde::{Deserialize, Serialize};

const DEFAULT_API_BASE: &str = "http://localhost:11434";
const DEFAULT_MODEL: &str = "gpt-oss:latest";

/// Service for generating chat completions using Ollama via the Rig framework.
pub struct LLMClient {
    client: ollama::Client,
    model: String,
}

impl Default for LLMClient {
    fn default() -> Self {
        Self::with_config(DEFAULT_API_BASE, DEFAULT_MODEL)
    }
}

impl LLMClient {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new LLM client with custom Ollama URL and model.
    pub fn with_config(api_base: &str, model: &str) -> Self {
        let client = ollama::Client::builder()
            .api_key(Nothing)
            .base_url(api_base)
            .build()
            .expect("failed to build Ollama client");

        Self {
            client,
            model: model.to_string(),
        }
    }

    /// Get the current model name.
    pub fn get_model(&self) -> &str {
        &self.model
    }

    /// Get a reference to the underlying Ollama client for sharing with agents.
    pub fn ollama_client(&self) -> &ollama::Client {
        &self.client
    }

    /// Test connection to the LLM service.
    pub async fn test_connection(&self) -> Result<()> {
        let agent = self.client.agent(&self.model).build();
        let _: String = agent
            .prompt("Hello")
            .await
            .context("Failed to connect to LLM service")?;
        Ok(())
    }

    /// Generate a chat completion with context.
    pub async fn chat_completion(
        &self,
        messages: Vec<ChatMessage>,
        system_prompt: Option<String>,
        _max_tokens: Option<u16>,
        _temperature: Option<f32>,
    ) -> Result<String> {
        let mut builder = self.client.agent(&self.model);
        if let Some(ref preamble) = system_prompt {
            builder = builder.preamble(preamble);
        }
        let agent = builder.build();

        // Separate the last user message as the prompt; everything before is history.
        let (history, prompt) = split_history_and_prompt(messages)?;

        if history.is_empty() {
            let response: String = agent
                .prompt(prompt)
                .await
                .context("Failed to generate chat completion")?;
            Ok(response)
        } else {
            let response: String = agent
                .chat(prompt, history)
                .await
                .context("Failed to generate chat completion")?;
            Ok(response)
        }
    }

    /// Generate a simple completion for a single prompt.
    pub async fn simple_completion(&self, prompt: &str, max_tokens: Option<u16>) -> Result<String> {
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        }];

        self.chat_completion(messages, None, max_tokens, None).await
    }

    /// Generate a completion with context and system prompt.
    pub async fn completion_with_context(
        &self,
        user_message: &str,
        context: &str,
        system_prompt: Option<&str>,
        max_tokens: Option<u16>,
    ) -> Result<String> {
        let system_content = if let Some(system_prompt) = system_prompt {
            format!("{}\n\nContext:\n{}", system_prompt, context)
        } else {
            format!(
                "Use the following context to answer the user's question:\n\n{}",
                context
            )
        };

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: user_message.to_string(),
        }];

        self.chat_completion(messages, Some(system_content), max_tokens, Some(0.7))
            .await
    }
}

/// Split a list of ChatMessages into Rig Message history and a final prompt Message.
fn split_history_and_prompt(messages: Vec<ChatMessage>) -> Result<(Vec<Message>, Message)> {
    if messages.is_empty() {
        return Err(anyhow::anyhow!("No messages provided"));
    }

    let mut rig_messages: Vec<Message> = messages.iter().map(|m| m.into()).collect();
    let prompt = rig_messages.pop().unwrap();

    Ok((rig_messages, prompt))
}

/// Simple message structure for LLM interactions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl From<&ChatMessage> for Message {
    fn from(msg: &ChatMessage) -> Self {
        match msg.role.as_str() {
            "assistant" => Message::assistant(&msg.content),
            // Map both "user" and "system" to user messages since Rig handles
            // system content via the agent preamble, not as chat history.
            _ => Message::user(&msg.content),
        }
    }
}

impl From<&crate::models::ChatMessage> for ChatMessage {
    fn from(message: &crate::models::ChatMessage) -> Self {
        ChatMessage {
            role: message.role.as_str().to_string(),
            content: message.content.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_client_creation() {
        let client = LLMClient::new();
        assert_eq!(client.get_model(), "gpt-oss:latest");
    }

    #[test]
    fn test_custom_config() {
        let client = LLMClient::with_config("http://localhost:11434", "custom-model");
        assert_eq!(client.get_model(), "custom-model");
    }

    #[test]
    fn test_split_history_single_message() {
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        }];
        let (history, _prompt) = split_history_and_prompt(messages).unwrap();
        assert!(history.is_empty());
    }

    #[test]
    fn test_split_history_multiple_messages() {
        let messages = vec![
            ChatMessage {
                role: "user".to_string(),
                content: "First".to_string(),
            },
            ChatMessage {
                role: "assistant".to_string(),
                content: "Response".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "Follow up".to_string(),
            },
        ];
        let (history, _prompt) = split_history_and_prompt(messages).unwrap();
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_split_history_empty() {
        let messages: Vec<ChatMessage> = vec![];
        assert!(split_history_and_prompt(messages).is_err());
    }

    // Note: These tests require a running Ollama instance with gpt-oss:latest
    // They will be skipped if Ollama is not available
    #[tokio::test]
    async fn test_simple_completion() {
        let client = LLMClient::new();

        // Test connection first
        if client.test_connection().await.is_err() {
            println!("Skipping LLM test - Ollama with gpt-oss:latest not available");
            return;
        }

        let result = client
            .simple_completion("Say hello in exactly 3 words.", Some(10))
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.is_empty());
        println!("LLM Response: {}", response);
    }

    #[tokio::test]
    async fn test_completion_with_context() {
        let client = LLMClient::new();

        if client.test_connection().await.is_err() {
            println!("Skipping LLM test - Ollama with gpt-oss:latest not available");
            return;
        }

        let context = "The game ends when all players have collected 5 victory points. Victory points are earned by completing quests or defeating monsters.";
        let user_question = "How do I win the game?";

        let result = client
            .completion_with_context(
                user_question,
                context,
                Some("You are a helpful assistant that explains board game rules."),
                Some(100),
            )
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.is_empty());
        assert!(
            response.to_lowercase().contains("victory")
                || response.to_lowercase().contains("points")
        );
        println!("Contextual LLM Response: {}", response);
    }

    #[tokio::test]
    async fn test_chat_completion_with_history() {
        let client = LLMClient::new();

        if client.test_connection().await.is_err() {
            println!("Skipping LLM test - Ollama with gpt-oss:latest not available");
            return;
        }

        let messages = vec![
            ChatMessage {
                role: "user".to_string(),
                content: "What's 2 + 2?".to_string(),
            },
            ChatMessage {
                role: "assistant".to_string(),
                content: "2 + 2 equals 4.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "What about that number times 3?".to_string(),
            },
        ];

        let result = client
            .chat_completion(messages, None, Some(50), Some(0.1))
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.is_empty());
        println!("Chat completion response: {}", response);
    }
}
