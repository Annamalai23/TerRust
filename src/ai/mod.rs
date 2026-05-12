//! AI assistance integration for TerRust
//!
//! Provides AI-powered command suggestions, completions, and intelligent assistance
//!
//! This module is only available when the "ai" feature is enabled.

pub mod provider;
pub use provider::{AI, Completion, ProviderFactory};

use crate::config::AIConfig;
use anyhow::{Context, Result};
use std::time::Duration;

/// AI client for communicating with AI services
#[derive(Debug, Clone)]
pub struct AIClient {
    /// AI configuration
    config: AIConfig,
    /// HTTP client for API requests
    #[cfg(feature = "ai")]
    http_client: reqwest::Client,
}

#[cfg(feature = "ai")]
impl AIClient {
    /// Create a new AIClient with the given configuration
    pub fn new(config: AIConfig) -> Self {
        let timeout = Duration::from_secs(config.timeout);
        let http_client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to create HTTP client");

        Self { config, http_client }
    }

    /// Send a prompt to the AI service and get a response
    ///
    /// # Arguments
    /// * `prompt` - The user's prompt or question
    ///
    /// # Returns
    /// The AI's response as a string
    pub async fn send_prompt(&self, prompt: &str) -> Result<String> {
        // This is a placeholder implementation
        // In a real implementation, this would call an actual AI API
        // like OpenAI, Anthropic, or a local LLM

        // Check if AI is enabled in config
        if !self.config.enabled {
            return Err(anyhow::anyhow!("AI is disabled in configuration"));
        }

        // For demo purposes, return a mock response
        // In production, this would make an actual HTTP request
        Ok(format!("AI response to: {}", prompt))
    }

    /// Get command suggestions based on partial input
    pub async fn get_suggestions(&self, partial: &str) -> Result<Vec<String>> {
        // Placeholder: return mock suggestions
        Ok(vec![
            format!("suggestion for: {}", partial),
            format!("another suggestion for: {}", partial),
        ])
    }

    /// Explain a command or concept
    pub async fn explain(&self, topic: &str) -> Result<String> {
        // Placeholder: return mock explanation
        Ok(format!("Explanation of {}: This is a detailed explanation.", topic))
    }

    /// Get the current AI configuration
    pub fn config(&self) -> &AIConfig {
        &self.config
    }

    /// Update the AI configuration
    pub fn update_config(&mut self, config: AIConfig) {
        self.config = config;
    }
}

#[cfg(not(feature = "ai"))]
impl AIClient {
    /// Create a new AIClient (stub when ai feature is disabled)
    pub fn new(config: AIConfig) -> Self {
        Self { config }
    }

    /// Stub implementation that returns an error when ai feature is disabled
    pub async fn send_prompt(&self, _prompt: &str) -> Result<String> {
        Err(anyhow::anyhow!(
            "AI feature is not enabled. Compile with --features ai"
        ))
    }

    /// Stub implementation
    pub async fn get_suggestions(&self, _partial: &str) -> Result<Vec<String>> {
        Ok(Vec::new())
    }

    /// Stub implementation
    pub async fn explain(&self, _topic: &str) -> Result<String> {
        Err(anyhow::anyhow!("AI feature is not enabled"))
    }

    /// Get configuration
    pub fn config(&self) -> &AIConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: AIConfig) {
        self.config = config;
    }
}

/// Result of an AI completion request
#[derive(Debug, Clone)]
pub struct CompletionResult {
    /// The completed text
    pub text: String,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
    /// Source or model used
    pub source: String,
}

/// Provider for AI completions
#[cfg(feature = "ai")]
#[derive(Debug, Clone)]
pub struct CompletionProvider {
    client: AIClient,
}

#[cfg(feature = "ai")]
impl CompletionProvider {
    pub fn new(config: AIConfig) -> Self {
        Self {
            client: AIClient::new(config),
        }
    }

    pub async fn complete(&self, prompt: &str) -> Result<CompletionResult> {
        let response = self.client.send_prompt(prompt).await?;
        Ok(CompletionResult {
            text: response,
            confidence: 0.95, // Placeholder
            source: self.client.config().default_provider.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[cfg(feature = "ai")]
    async fn test_send_prompt() {
        let config = AIConfig {
            default_provider: "mock".to_string(),
            timeout: 30,
            max_context_tokens: 1000,
            show_typing_indicator: false,
            enabled: true,
            providers: serde_json::from_str("{}").unwrap(),
        };

        let client = AIClient::new(config);
        let response = client.send_prompt("test prompt").await.unwrap();
        assert!(response.contains("test prompt"));
    }

    #[test]
    fn test_ai_disabled() {
        let config = AIConfig {
            default_provider: "mock".to_string(),
            timeout: 30,
            max_context_tokens: 1000,
            show_typing_indicator: false,
            enabled: false,
            providers: serde_json::from_str("{}").unwrap(),
        };

        let client = AIClient::new(config);
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(client.send_prompt("test"));

        assert!(result.is_err());
    }
}
