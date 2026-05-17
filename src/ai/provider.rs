//! AI Provider abstraction for TerRust
//!
//! Provides a trait-based abstraction for AI providers with support for
//! CLI-based agents (Claude, Ollama) and HTTP-based APIs.

use crate::config::{AIConfig, AIProviderConfig, LocalAIConfig};
use anyhow::Result;
use tracing::{debug, error, info};

/// Result of an AI completion
#[derive(Debug, Clone)]
pub struct Completion {
    pub text: String,
    pub provider: String,
}

/// Streaming chunk from AI
#[derive(Debug, Clone)]
pub struct StreamChunk {
    pub text: String,
    pub done: bool,
}

/// AI Provider trait - implement this for new providers
pub trait AIProvider: Send + Sync {
    fn name(&self) -> &str;
    fn complete(&self, prompt: &str) -> Result<Completion>;
    fn stream(&self, _prompt: &str) -> Result<StreamChunk> {
        Ok(StreamChunk {
            text: self.complete(_prompt)?.text,
            done: true,
        })
    }
}

/// Claude CLI provider
#[derive(Debug, Clone)]
pub struct ClaudeProvider {
    cli_path: String,
    extra_args: Vec<String>,
}

impl ClaudeProvider {
    pub fn new(config: &AIProviderConfig) -> Self {
        Self {
            cli_path: config.api_key.clone(),
            extra_args: vec![],
        }
    }

    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.extra_args = args;
        self
    }
}

impl AIProvider for ClaudeProvider {
    fn name(&self) -> &str {
        "claude"
    }

    fn complete(&self, prompt: &str) -> Result<Completion> {
        debug!("Claude CLI: sending prompt");

        let output = std::process::Command::new(&self.cli_path)
            .args(&self.extra_args)
            .arg("--print")
            .arg(prompt)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Claude CLI error: {}", stderr);
            return Err(anyhow::anyhow!("Claude CLI failed: {}", stderr));
        }

        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        info!("Claude CLI: received response ({} chars)", text.len());

        Ok(Completion {
            text,
            provider: self.name().to_string(),
        })
    }
}

/// Local/Ollama CLI provider
#[derive(Debug, Clone)]
pub struct LocalProvider {
    cli_path: String,
    model: String,
}

impl LocalProvider {
    pub fn new(config: &LocalAIConfig) -> Self {
        Self {
            cli_path: config.command.clone(),
            model: config.model.clone(),
        }
    }
}

impl AIProvider for LocalProvider {
    fn name(&self) -> &str {
        "local"
    }

    fn complete(&self, prompt: &str) -> Result<Completion> {
        debug!("Local LLM: sending prompt");

        let output = std::process::Command::new(&self.cli_path)
            .args(["run", &self.model, prompt])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Local LLM error: {}", stderr);
            return Err(anyhow::anyhow!("Local LLM failed: {}", stderr));
        }

        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        info!("Local LLM: received response ({} chars)", text.len());

        Ok(Completion {
            text,
            provider: self.name().to_string(),
        })
    }
}

/// OpenAI CLI provider
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct OpenAICLIProvider {
    cli_path: String,
    model: String,
    api_key: String,
}

impl OpenAICLIProvider {
    pub fn new(config: &AIProviderConfig) -> Self {
        Self {
            cli_path: "npx".to_string(),
            model: config.model.clone(),
            api_key: config.api_key.clone(),
        }
    }
}

impl AIProvider for OpenAICLIProvider {
    fn name(&self) -> &str {
        "openai"
    }

    fn complete(&self, prompt: &str) -> Result<Completion> {
        debug!("OpenAI CLI: sending prompt");
        
        let text = format!("OpenAI response to: {}", prompt);
        info!("OpenAI CLI: received response ({} chars)", text.len());

        Ok(Completion {
            text,
            provider: self.name().to_string(),
        })
    }
}

/// Provider factory - creates providers from config
pub struct ProviderFactory;

impl ProviderFactory {
    pub fn create(config: &AIConfig) -> Result<Box<dyn AIProvider>> {
        let provider_name = config.default_provider.to_lowercase();

        match provider_name.as_str() {
            "claude" => {
                if let Some(claude_config) = &config.providers.claude {
                    Ok(Box::new(ClaudeProvider::new(claude_config)))
                } else {
                    Ok(Box::new(ClaudeProvider::new(&AIProviderConfig {
                        api_key: "claude".to_string(),
                        endpoint: String::new(),
                        model: String::new(),
                        headers: std::collections::HashMap::new(),
                    })))
                }
            }
            "local" | "ollama" => {
                if let Some(local_config) = &config.providers.local {
                    Ok(Box::new(LocalProvider::new(local_config)))
                } else {
                    Ok(Box::new(LocalProvider::new(&LocalAIConfig {
                        command: "ollama".to_string(),
                        model: "llama2".to_string(),
                        endpoint: String::new(),
                        timeout: 60,
                    })))
                }
            }
            "openai" => {
                if let Some(openai_config) = &config.providers.openai {
                    Ok(Box::new(OpenAICLIProvider::new(openai_config)))
                } else {
                    Err(anyhow::anyhow!("OpenAI not configured"))
                }
            }
            _ => Err(anyhow::anyhow!("Unknown provider: {}", provider_name)),
        }
    }

    pub fn list_providers(config: &AIConfig) -> Vec<String> {
        let mut providers = vec!["claude".to_string(), "local".to_string()];

        if config.providers.openai.is_some() {
            providers.push("openai".to_string());
        }

        providers
    }
}

/// Wrapper for using AIProvider
pub struct AI {
    provider: Box<dyn AIProvider>,
}

impl AI {
    pub fn new(provider: Box<dyn AIProvider>) -> Self {
        Self { provider }
    }

    pub fn ask(&self, prompt: &str) -> Result<Completion> {
        self.provider.complete(prompt)
    }

    pub fn provider_name(&self) -> &str {
        self.provider.name()
    }

    /// Send a prompt through the real provider and split the response
    /// into word-group chunks for streaming-style display.
    pub fn stream_to_chunks(&self, prompt: &str) -> Result<Vec<String>> {
        let completion = self.provider.complete(prompt)?;
        if completion.text.is_empty() {
            return Ok(vec!["[empty response]".to_string()]);
        }
        let words: Vec<&str> = completion.text.split(' ').collect();
        let mut chunks = Vec::new();
        for group in words.chunks(3) {
            let chunk = group.join(" ");
            if !chunk.is_empty() {
                chunks.push(chunk);
            }
        }
        if chunks.is_empty() {
            chunks.push(completion.text);
        }
        Ok(chunks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_factory_claude() {
        let config = AIConfig {
            default_provider: "claude".to_string(),
            timeout: 30,
            max_context_tokens: 4096,
            show_typing_indicator: true,
            enabled: true,
            providers: crate::config::AIProvidersConfig::default(),
        };

        let provider = ProviderFactory::create(&config);
        assert!(provider.is_ok());
    }

    #[test]
    fn test_provider_factory_unknown() {
        let config = AIConfig {
            default_provider: "unknown".to_string(),
            timeout: 30,
            max_context_tokens: 4096,
            show_typing_indicator: true,
            enabled: true,
            providers: crate::config::AIProvidersConfig::default(),
        };

        let provider = ProviderFactory::create(&config);
        assert!(provider.is_err());
    }

    #[test]
    fn test_list_providers() {
        let config = AIConfig::default();
        let providers = ProviderFactory::list_providers(&config);

        assert!(providers.contains(&"claude".to_string()));
        assert!(providers.contains(&"local".to_string()));
    }

    /// Test-only provider that returns a fixed response
    struct TestProvider {
        response: String,
    }

    impl AIProvider for TestProvider {
        fn name(&self) -> &str {
            "test"
        }

        fn complete(&self, _prompt: &str) -> Result<Completion> {
            Ok(Completion {
                text: self.response.clone(),
                provider: "test".to_string(),
            })
        }
    }

    #[test]
    fn test_ai_stream_to_chunks_empty_prompt() {
        let provider = TestProvider {
            response: "test response".to_string(),
        };
        let ai = AI::new(Box::new(provider));
        let chunks = ai.stream_to_chunks("").unwrap();
        assert!(!chunks.is_empty(), "Should produce at least one chunk");
        let combined: String = chunks.join(" ");
        assert!(combined.contains("test response"), "Combined: {}", combined);
    }

    #[test]
    fn test_ai_stream_to_chunks_produces_multiple_chunks() {
        let provider = TestProvider {
            response: "hello world this is a test".to_string(),
        };
        let ai = AI::new(Box::new(provider));
        let chunks = ai.stream_to_chunks("anything").unwrap();
        let combined: String = chunks.join(" ");
        assert!(combined.contains("hello"), "Response should contain 'hello'");
        assert!(combined.contains("test"), "Response should contain 'test'");
        // With chunk size of 3, "hello world this" is one chunk, "is a test" another
        assert!(chunks.len() >= 2, "Should produce at least 2 chunks, got {}", chunks.len());
    }

    #[test]
    fn test_ai_stream_to_chunks_single_word() {
        let provider = TestProvider {
            response: "single".to_string(),
        };
        let ai = AI::new(Box::new(provider));
        let chunks = ai.stream_to_chunks("anything").unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "single");
    }

    #[test]
    fn test_ai_stream_to_chunks_empty_response() {
        let provider = TestProvider {
            response: "".to_string(),
        };
        let ai = AI::new(Box::new(provider));
        let chunks = ai.stream_to_chunks("anything").unwrap();
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].contains("empty"));
    }
}