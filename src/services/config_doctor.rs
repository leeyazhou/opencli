use std::path::Path;

use anyhow::Result;
use tracing::{info, warn};

use crate::{config::RuntimeConfig, provider_factory::ProviderFactory};

pub async fn run_config_doctor(config: &RuntimeConfig) -> Result<Vec<String>> {
    let mut issues = Vec::new();

    info!(provider = %config.provider, base_url = %config.base_url, "running config doctor");

    if config.api_key.is_empty() {
        issues.push("apiKey is missing".to_string());
    }
    if config.base_url.is_empty() {
        issues.push("baseUrl is missing".to_string());
    }
    if config.provider == "openai-compatible" && !config.base_url.contains("/v1") {
        issues.push("openai-compatible baseUrl usually needs a /v1 suffix".to_string());
    }
    if config.provider == "anthropic" && !config.base_url.contains("anthropic") {
        issues.push("anthropic provider usually needs an Anthropic API base URL".to_string());
    }
    if config.allowed_tool_kinds.is_empty() {
        issues.push("allowedToolKinds is empty".to_string());
    }
    if !Path::new(&config.workspace_root).exists() {
        issues.push("workspaceRoot does not exist".to_string());
    }

    match ProviderFactory::new().create(config) {
        Ok(provider) => match provider.list_models().await {
            Ok(models) if models.is_empty() => {
                warn!(provider = %config.provider, "provider probe returned no models");
                issues.push("provider endpoint probe succeeded but returned no models".to_string());
            }
            Ok(_) => {}
            Err(error) => {
                warn!(provider = %config.provider, error = %error, "provider endpoint probe failed");
                issues.push(format!("provider endpoint probe failed: {error:#}"));
            }
        },
        Err(error) => {
            warn!(provider = %config.provider, error = %error, "provider initialization failed");
            issues.push(format!("provider initialization failed: {error:#}"));
        }
    }

    Ok(issues)
}
