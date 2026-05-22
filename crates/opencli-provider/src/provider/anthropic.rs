use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use serde_json::{Value, json};
use std::ops::ControlFlow;
use std::sync::{Arc, atomic::AtomicBool};
use tracing::{debug, info};

use opencli_tools::{ToolCall, ToolDefinition};

use crate::{message::ChatMessage, output::Renderer};

use super::{
    ChatResponse, Provider, ProviderCapabilities, ProviderConfig,
    util::{
        ModelListResponse, build_client, cancelable_request, collect_model_ids, ensure_success,
        for_each_sse_data_line, serialize_anthropic_message, serialize_anthropic_tool,
    },
};

pub struct AnthropicProvider {
    client: Client,
    base_url: String,
    api_key: String,
    model: String,
    anthropic_version: String,
    temperature: f32,
    max_tokens: u32,
}

#[cfg(all(test, feature = "anthropic"))]
mod tests {
    use mockito::Server;

    use super::*;

    #[tokio::test]
    async fn lists_models_from_mock_server() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("GET", "/models")
            .match_header("x-api-key", "test-key")
            .with_status(200)
            .with_body(r#"{"data":[{"id":"claude-test"}]}"#)
            .create_async()
            .await;

        let config = ProviderConfig {
            base_url: server.url(),
            api_key: "test-key".into(),
            model: "claude-3-5-sonnet-latest".into(),
            anthropic_version: "2023-06-01".into(),
            temperature: 0.2,
            max_tokens: 4096,
            request_timeout_ms: 120_000,
        };
        let provider =
            AnthropicProvider::from_config(&config).expect("provider config should be valid");
        let models = provider
            .list_models()
            .await
            .expect("model listing should succeed");
        assert_eq!(models, vec!["claude-test".to_string()]);
    }
}

#[derive(Debug, Deserialize)]
struct AnthropicMessageResponse {
    content: Vec<AnthropicContentBlock>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum AnthropicContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },
}

impl AnthropicProvider {
    pub fn from_config(config: &ProviderConfig) -> Result<Self> {
        if config.api_key.is_empty() {
            bail!(
                "missing apiKey, run `opencli config init` and update ~/.config/opencli/config.json"
            );
        }

        Ok(Self {
            client: build_client(config)?,
            base_url: config.base_url.clone(),
            api_key: config.api_key.clone(),
            model: config.model.clone(),
            anthropic_version: config.anthropic_version.clone(),
            temperature: config.temperature,
            max_tokens: config.max_tokens,
        })
    }
}

#[async_trait(?Send)]
impl Provider for AnthropicProvider {
    fn name(&self) -> &'static str {
        "anthropic"
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            supports_streaming: true,
            supports_tools: true,
            supports_model_listing: true,
        }
    }

    async fn complete_chat(
        &self,
        messages: &[ChatMessage],
        tools: &[ToolDefinition],
        cancel_requested: Option<&Arc<AtomicBool>>,
    ) -> Result<ChatResponse> {
        info!(provider = "anthropic", model = %self.model, message_count = messages.len(), tool_count = tools.len(), "sending completion request");
        let response = cancelable_request(
            self.client.post(format!("{}/messages", self.base_url.trim_end_matches('/')))
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", &self.anthropic_version)
                .json(&json!({
                    "model": self.model,
                    "max_tokens": self.max_tokens,
                    "temperature": self.temperature,
                    "messages": messages.iter().map(serialize_anthropic_message).collect::<Vec<_>>(),
                    "tools": tools.iter().map(serialize_anthropic_tool).collect::<Vec<_>>(),
                    "stream": false,
                }))
                .send(),
            cancel_requested,
        ).await?;

        let body = ensure_success(response)
            .await?
            .json::<AnthropicMessageResponse>()
            .await
            .context("failed to parse anthropic response")?;
        let mut content = String::new();
        let mut tool_calls = Vec::new();
        for block in body.content {
            match block {
                AnthropicContentBlock::Text { text } => content.push_str(&text),
                AnthropicContentBlock::ToolUse { id, name, input } => tool_calls.push(ToolCall {
                    id,
                    name,
                    arguments: serde_json::to_string(&input)?,
                }),
            }
        }
        Ok(ChatResponse {
            content,
            tool_calls,
        })
    }

    async fn stream_text(
        &self,
        messages: &[ChatMessage],
        renderer: &mut dyn Renderer,
        cancel_requested: Option<&Arc<AtomicBool>>,
    ) -> Result<String> {
        info!(provider = "anthropic", model = %self.model, message_count = messages.len(), "starting streaming request");
        let response = cancelable_request(
            self.client.post(format!("{}/messages", self.base_url.trim_end_matches('/')))
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", &self.anthropic_version)
                .json(&json!({
                    "model": self.model,
                    "max_tokens": self.max_tokens,
                    "temperature": self.temperature,
                    "messages": messages.iter().map(serialize_anthropic_message).collect::<Vec<_>>(),
                    "stream": true,
                }))
                .send(),
            cancel_requested,
        ).await?;
        let mut response = ensure_success(response).await?;
        let mut output = String::new();
        for_each_sse_data_line(&mut response, cancel_requested, |payload| {
            if payload.is_empty() || payload == "[DONE]" {
                return Ok(ControlFlow::Continue(()));
            }
            let parsed: Value = match serde_json::from_str(payload) {
                Ok(value) => value,
                Err(_) => return Ok(ControlFlow::Continue(())),
            };
            if parsed.get("type").and_then(Value::as_str) == Some("message_stop") {
                renderer.render_line("")?;
                return Ok(ControlFlow::Break(()));
            }
            if let Some(text) = parsed.pointer("/delta/text").and_then(Value::as_str) {
                debug!(
                    provider = "anthropic",
                    chunk_len = text.len(),
                    "received text delta"
                );
                renderer.render_text(text)?;
                output.push_str(text);
            }
            Ok(ControlFlow::Continue(()))
        })
        .await?;
        if !output.is_empty() {
            renderer.render_line("")?;
        }
        Ok(output)
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        info!(provider = "anthropic", "listing models");
        let response = self
            .client
            .get(format!("{}/models", self.base_url.trim_end_matches('/')))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", &self.anthropic_version)
            .send()
            .await?;
        let body = ensure_success(response)
            .await?
            .json::<ModelListResponse>()
            .await
            .context("failed to parse anthropic models response")?;
        Ok(collect_model_ids(body))
    }
}
