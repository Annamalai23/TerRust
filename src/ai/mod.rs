//! AI assistance integration for TerRust
//!
//! Provides AI-powered command suggestions, completions, and intelligent assistance
//!
//! This module is only available when the "ai" feature is enabled.

pub mod provider;
pub use provider::{AI, Completion, ProviderFactory};

use crate::config::AIConfig;
use anyhow::Result;
use std::time::Duration;

/// AI client for communicating with AI services
#[derive(Debug, Clone)]
pub struct AIClient {
    /// AI configuration
    config: AIConfig,
    /// HTTP client for API requests
    #[cfg(feature = "ai")]
    #[allow(dead_code)]
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
            .unwrap_or_else(|e| {
                tracing::warn!("Failed to create HTTP client with timeout: {}. Using default.", e);
                reqwest::Client::new()
            });

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

/// Stream a prompt response as chunks of text.
/// Returns a list of text chunks that can be reassembled into the full response.
/// This simulates streaming from any provider (mock implementation).
#[cfg(feature = "ai")]
pub async fn stream_prompt_text(provider: &str, prompt: &str) -> Vec<String> {
    // Simulate streaming by splitting the response into sensible chunks
    let full = format!("[{}] Response to: {}", provider, prompt);
    let words: Vec<&str> = full.split(' ').collect();
    let mut chunks = Vec::new();
    for chunk in words.chunks(2) {
        chunks.push(chunk.join(" "));
    }
    // Ensure at least one chunk
    if chunks.is_empty() {
        chunks.push(full);
    }
    chunks
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

    #[tokio::test]
    #[cfg(feature = "ai")]
    async fn test_stream_prompt_text() {
        let chunks = stream_prompt_text("mock", "hello world").await;
        assert!(!chunks.is_empty(), "Should produce at least one chunk");
        let full: String = chunks.join(" ");
        assert!(full.contains("hello world"), "Response should contain prompt");
    }

    #[tokio::test]
    #[cfg(feature = "ai")]
    async fn test_stream_prompt_text_empty_prompt() {
        let chunks = stream_prompt_text("test", "").await;
        assert!(!chunks.is_empty(), "Should produce at least one chunk for empty prompt");
    }

    #[test]
    #[cfg(feature = "ai")]
    fn test_chunk_reassembly() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let chunks = rt.block_on(stream_prompt_text("claude", "explain rust"));
        let combined = chunks.join(" ");
        assert!(combined.starts_with("[claude]"), "Should start with provider tag");
        assert!(combined.contains("rust"), "Should contain the prompt");
    }

    /// Test that AiStreamChunk events can be processed by the app handler
    #[test]
    #[cfg(feature = "ai")]
    fn test_ai_stream_chunk_event_creation() {
        let id = uuid::Uuid::new_v4();
        let event = crate::app::AppEvent::AiStreamChunk("hello".to_string(), id);
        match event {
            crate::app::AppEvent::AiStreamChunk(text, uuid) => {
                assert_eq!(text, "hello");
                assert_eq!(uuid, id);
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    #[cfg(feature = "ai")]
    fn test_ai_stream_done_event_creation() {
        let id = uuid::Uuid::new_v4();
        let event = crate::app::AppEvent::AiStreamDone(id);
        match event {
            crate::app::AppEvent::AiStreamDone(uuid) => {
                assert_eq!(uuid, id);
            }
            _ => panic!("Wrong event type"),
        }
    }
}
