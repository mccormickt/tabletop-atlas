use anyhow::{Context, Result};
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestAssistantMessage, ChatCompletionRequestMessage,
        ChatCompletionRequestSystemMessage, ChatCompletionRequestUserMessage,
        CreateChatCompletionRequestArgs,
    },
};
use serde::{Deserialize, Serialize};

const DEFAULT_MODEL: &str = "mistral-small3.2:24b";

/// Service for generating chat completions using OpenAI-compatible APIs (like Ollama)
pub struct LLMClient {
    client: Client<OpenAIConfig>,
    model: String,
}

/// Initialize a new LLM client configured for Ollama
impl Default for LLMClient {
    fn default() -> Self {
        // Configure for local Ollama instance
        let api_base = "http://localhost:11434/v1";
        let api_key = "ollama"; // Required but ignored by Ollama

        let config = OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base(api_base);

        let client = Client::with_config(config);

        Self {
            client,
            model: DEFAULT_MODEL.to_string(),
        }
    }
}

impl LLMClient {
    /// Create a new LLM client configured for Ollama by default
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new LLM client with custom configuration
    pub fn with_config(api_base: &str, api_key: &str, model: &str) -> Self {
        let config = OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base(api_base);

        let client = Client::with_config(config);

        Self {
            client,
            model: model.to_string(),
        }
    }

    /// Get the current model name
    pub fn get_model(&self) -> &str {
        &self.model
    }

    /// Test connection to the LLM service
    pub async fn test_connection(&self) -> Result<()> {
        let request = CreateChatCompletionRequestArgs::default()
            .model(self.model.clone())
            .messages(vec![ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessage {
                    content: "Hello".into(),
                    name: None,
                },
            )])
            .build()
            .context("Failed to build test chat completion request")?;

        self.client
            .chat()
            .create(request)
            .await
            .context("Failed to connect to LLM service")?;

        Ok(())
    }

    /// Generate a chat completion with context
    pub async fn chat_completion(
        &self,
        messages: Vec<ChatMessage>,
        system_prompt: Option<String>,
        max_tokens: Option<u16>,
        temperature: Option<f32>,
    ) -> Result<String> {
        let mut request_messages = Vec::new();

        // Add system message if provided
        if let Some(system_content) = system_prompt {
            request_messages.push(ChatCompletionRequestMessage::System(
                ChatCompletionRequestSystemMessage {
                    content: system_content,
                    name: None,
                },
            ));
        }

        // Convert our messages to OpenAI format
        for message in messages {
            let request_message = match message.role.as_str() {
                "user" => ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                    content: message.content.into(),
                    name: None,
                }),
                "assistant" => {
                    ChatCompletionRequestMessage::Assistant(ChatCompletionRequestAssistantMessage {
                        content: Some(message.content),
                        name: None,
                        tool_calls: None,
                        ..Default::default()
                    })
                }
                "system" => {
                    ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                        content: message.content,
                        name: None,
                    })
                }
                _ => {
                    return Err(anyhow::anyhow!(
                        "Unsupported message role: {}",
                        message.role
                    ));
                }
            };
            request_messages.push(request_message);
        }

        let request = CreateChatCompletionRequestArgs::default()
            .model(self.model.clone())
            .messages(request_messages)
            .build()
            .context("Failed to build chat completion request")?;

        let response = self
            .client
            .chat()
            .create(request)
            .await
            .context("Failed to generate chat completion")?;

        let content = response
            .choices
            .first()
            .and_then(|choice| choice.message.content.as_ref())
            .context("No content in chat completion response")?;

        Ok(content.clone())
    }

    /// Generate a simple completion for a single prompt
    pub async fn simple_completion(&self, prompt: &str, max_tokens: Option<u16>) -> Result<String> {
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        }];

        self.chat_completion(messages, None, max_tokens, None).await
    }

    /// Generate a completion with context and system prompt
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

/// Simple message structure for LLM interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
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
        assert_eq!(client.get_model(), "mistral-small3.2:24b");
    }

    #[test]
    fn test_custom_config() {
        let client =
            LLMClient::with_config("http://localhost:11434/v1", "test-key", "custom-model");
        assert_eq!(client.get_model(), "custom-model");
    }

    // Note: These tests require a running Ollama instance with mistral-small3.2:24b
    // They will be skipped if Ollama is not available
    #[tokio::test]
    async fn test_simple_completion() {
        let client = LLMClient::new();

        // Test connection first
        if client.test_connection().await.is_err() {
            println!("Skipping LLM test - Ollama with mistral-small3.2:24b not available");
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
            println!("Skipping LLM test - Ollama with mistral-small3.2:24b not available");
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
            println!("Skipping LLM test - Ollama with mistral-small3.2:24b not available");
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
