use terrust::ai::{AIClient, CompletionResult, CompletionProvider};
use terrust::config::AIConfig;

fn test_ai_config(enabled: bool) -> AIConfig {
    AIConfig {
        enabled,
        default_provider: "mock".to_string(),
        timeout: 30,
        max_context_tokens: 4096,
        show_typing_indicator: false,
        providers: terrust::config::AIProvidersConfig::default(),
    }
}

#[test]
fn test_ai_client_creation() {
    let config = test_ai_config(true);
    let client = AIClient::new(config);
    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(client.send_prompt("hello"));
    assert!(result.is_ok());
    assert!(result.unwrap().contains("hello"));
}

#[test]
fn test_ai_client_disabled() {
    let config = test_ai_config(false);
    let client = AIClient::new(config);
    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(client.send_prompt("test"));
    assert!(result.is_err());
}

#[test]
fn test_ai_client_suggestions() {
    let config = test_ai_config(true);
    let client = AIClient::new(config);
    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(client.get_suggestions("git"));
    assert!(result.is_ok());
    let suggestions = result.unwrap();
    assert!(!suggestions.is_empty());
}

#[test]
fn test_completion_result_creation() {
    let result = CompletionResult {
        text: "test response".to_string(),
        confidence: 0.95,
        source: "mock".to_string(),
    };
    assert_eq!(result.text, "test response");
    assert_eq!(result.confidence, 0.95);
}

#[test]
fn test_completion_provider_creation() {
    let config = test_ai_config(true);
    let provider = CompletionProvider::new(config);
    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(provider.complete("hello"));
    assert!(result.is_ok());
    assert!(result.unwrap().text.contains("hello"));
}

#[test]
fn test_ai_client_config_access() {
    let config = test_ai_config(true);
    let client = AIClient::new(config);
    assert!(!client.config().show_typing_indicator);
    assert_eq!(client.config().timeout, 30);
}

#[test]
fn test_ai_client_update_config() {
    let config = test_ai_config(true);
    let mut client = AIClient::new(config);
    let mut new_config = test_ai_config(true);
    new_config.timeout = 60;
    client.update_config(new_config);
    assert_eq!(client.config().timeout, 60);
}

#[test]
fn test_ai_provider_factory() {
    use terrust::ai::ProviderFactory;

    let claude_config = AIConfig {
        default_provider: "claude".to_string(),
        ..test_ai_config(true)
    };
    let provider = ProviderFactory::create(&claude_config);
    assert!(provider.is_ok());

    let unknown_config = AIConfig {
        default_provider: "unknown".to_string(),
        ..test_ai_config(true)
    };
    let provider = ProviderFactory::create(&unknown_config);
    assert!(provider.is_err());
}
