use anyhow::{bail, Result};

use crate::{
    config::RuntimeConfig,
    provider::{OpenAiCompatibleProvider, Provider},
};

#[cfg(feature = "anthropic")]
use crate::provider::AnthropicProvider;

pub struct ProviderFactory;

impl ProviderFactory {
    pub fn new() -> Self {
        Self
    }

    pub fn create(&self, config: &RuntimeConfig) -> Result<Box<dyn Provider>> {
        match config.provider.as_str() {
            "openai-compatible" => Ok(Box::new(OpenAiCompatibleProvider::from_config(config)?)),
            #[cfg(feature = "anthropic")]
            "anthropic" => Ok(Box::new(AnthropicProvider::from_config(config)?)),
            #[cfg(not(feature = "anthropic"))]
            "anthropic" => bail!("anthropic provider disabled at compile time"),
            other => bail!("unsupported provider: {other}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RuntimeConfig;

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
