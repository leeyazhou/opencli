use anyhow::{Result, bail};
use opencli_config::RuntimeConfig;

use crate::{OpenAiCompatibleProvider, Provider, ProviderConfig};

#[cfg(feature = "anthropic")]
use crate::AnthropicProvider;

pub struct ProviderFactory;

impl Default for ProviderFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl ProviderFactory {
    pub fn new() -> Self {
        Self
    }

    pub fn create(&self, config: &RuntimeConfig) -> Result<Box<dyn Provider>> {
        let provider_config = ProviderConfig {
            base_url: config.base_url.clone(),
            api_key: config.api_key.clone(),
            model: config.model.clone(),
            anthropic_version: config.anthropic_version.clone(),
            temperature: config.temperature,
            max_tokens: config.max_tokens,
            request_timeout_ms: config.request_timeout_ms,
        };

        match config.provider.as_str() {
            "openai-compatible" => Ok(Box::new(OpenAiCompatibleProvider::from_config(
                &provider_config,
            )?)),
            #[cfg(feature = "anthropic")]
            "anthropic" => Ok(Box::new(AnthropicProvider::from_config(&provider_config)?)),
            #[cfg(not(feature = "anthropic"))]
            "anthropic" => bail!("anthropic provider disabled at compile time"),
            other => bail!("unsupported provider: {other}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_supported_providers() {
        let factory = ProviderFactory::new();

        let openai = RuntimeConfig {
            api_key: "test".into(),
            ..RuntimeConfig::default()
        };
        assert!(factory.create(&openai).is_ok());

        #[cfg(feature = "anthropic")]
        {
            let anthropic = RuntimeConfig {
                provider: "anthropic".into(),
                base_url: "https://api.anthropic.com/v1".into(),
                api_key: "test".into(),
                ..RuntimeConfig::default()
            };
            assert!(factory.create(&anthropic).is_ok());
        }
    }
}
