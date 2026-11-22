use anyhow::Result;
use rig_core::providers::ollama;

/// Configuration for Ollama LLM and embedding services.
///
/// Resolved by clap with priority: CLI arg > env var > hardcoded default.
pub struct OllamaConfig {
    pub api_base: String,
    pub llm_model: String,
    pub embedding_model: String,
}

impl OllamaConfig {
    /// Extract resolved values from clap matches.
    pub fn from_matches(matches: &clap::ArgMatches) -> Self {
        Self {
            api_base: matches.get_one::<String>("ollama-url").unwrap().clone(),
            llm_model: matches.get_one::<String>("llm-model").unwrap().clone(),
            embedding_model: matches
                .get_one::<String>("embedding-model")
                .unwrap()
                .clone(),
        }
    }

    /// Build a shared Ollama client from this configuration.
    pub fn build_client(&self) -> Result<ollama::Ollama> {
        Ok(ollama::OllamaConfig::new()
            .with_base_url(&self.api_base)
            .client())
    }
}
