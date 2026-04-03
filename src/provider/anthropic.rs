use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use tokio::time::{Duration as TokioDuration, sleep};
use tracing::{debug, info};

use crate::{config::RuntimeConfig, message::ChatMessage, output::Renderer, tools::{ToolCall, ToolDefinition}};

use super::{ChatResponse, Provider, ProviderCapabilities, util::{build_client, ensure_success, serialize_anthropic_message, serialize_anthropic_tool}};

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

        let config = RuntimeConfig {
            provider: "anthropic".into(),
            base_url: server.url(),
            api_key: "test-key".into(),
            ..RuntimeConfig::default()
        };
        let provider = AnthropicProvider::from_config(&config).unwrap();
        let models = provider.list_models().await.unwrap();
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
    ToolUse { id: String, name: String, input: Value },
}

#[derive(Debug, Deserialize)]
struct ModelsResponse {
    data: Vec<ModelInfo>,
}

#[derive(Debug, Deserialize)]
struct ModelInfo {
    id: String,
}

impl AnthropicProvider {
    pub fn from_config(config: &RuntimeConfig) -> Result<Self> {
        if config.api_key.is_empty() {
            bail!("missing apiKey, run `ai-cli config init` and update ~/.config/ai-cli/config.json");
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

    async fn complete_chat(&self, messages: &[ChatMessage], tools: &[ToolDefinition], cancel_requested: Option<&Arc<AtomicBool>>) -> Result<ChatResponse> {
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

        let body = ensure_success(response).await?
            .json::<AnthropicMessageResponse>()
            .await
            .context("failed to parse anthropic response")?;
        let mut content = String::new();
        let mut tool_calls = Vec::new();
        for block in body.content {
            match block {
                AnthropicContentBlock::Text { text } => content.push_str(&text),
                AnthropicContentBlock::ToolUse { id, name, input } => tool_calls.push(ToolCall { id, name, arguments: serde_json::to_string(&input)? }),
            }
        }
        Ok(ChatResponse { content, tool_calls })
    }

    async fn stream_text(&self, messages: &[ChatMessage], renderer: &mut dyn Renderer, cancel_requested: Option<&Arc<AtomicBool>>) -> Result<String> {
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
        let mut buffer = String::new();
        let mut output = String::new();
        while let Some(chunk) = cancelable_response_chunk(&mut response, cancel_requested).await? {
            buffer.push_str(&String::from_utf8_lossy(&chunk));
            while let Some(index) = buffer.find('\n') {
                let line = buffer[..index].trim().to_string();
                buffer = buffer[index + 1..].to_string();
                if !line.starts_with("data:") { continue; }
                let payload = line.trim_start_matches("data:").trim();
                if payload.is_empty() || payload == "[DONE]" { continue; }
                let parsed: Value = match serde_json::from_str(payload) { Ok(v) => v, Err(_) => continue };
                if parsed.get("type").and_then(Value::as_str) == Some("message_stop") {
                    renderer.render_line("")?;
                    return Ok(output);
                }
                if let Some(text) = parsed.pointer("/delta/text").and_then(Value::as_str) {
                    debug!(provider = "anthropic", chunk_len = text.len(), "received text delta");
                    renderer.render_text(text)?;
                    output.push_str(text);
                }
            }
        }
        if !output.is_empty() { renderer.render_line("")?; }
        Ok(output)
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        info!(provider = "anthropic", "listing models");
        let response = self.client.get(format!("{}/models", self.base_url.trim_end_matches('/')))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", &self.anthropic_version)
            .send().await?;
        let body = ensure_success(response).await?
            .json::<ModelsResponse>()
            .await
            .context("failed to parse anthropic models response")?;
        Ok(body.data.into_iter().map(|item| item.id).collect())
    }
}

async fn cancelable_request<F, T>(future: F, cancel_requested: Option<&Arc<AtomicBool>>) -> Result<T>
where
    F: std::future::Future<Output = Result<T, reqwest::Error>>,
{
    tokio::pin!(future);
    loop {
        if cancel_requested.is_some_and(|flag| flag.load(Ordering::Relaxed)) {
            bail!("agent execution canceled");
        }

        tokio::select! {
            result = &mut future => return Ok(result?),
            _ = sleep(TokioDuration::from_millis(50)) => {}
        }
    }
}

async fn cancelable_response_chunk(
    response: &mut reqwest::Response,
    cancel_requested: Option<&Arc<AtomicBool>>,
) -> Result<Option<bytes::Bytes>> {
    loop {
        if cancel_requested.is_some_and(|flag| flag.load(Ordering::Relaxed)) {
            bail!("agent execution canceled");
        }

        tokio::select! {
            result = response.chunk() => return Ok(result?),
            _ = sleep(TokioDuration::from_millis(50)) => {}
        }
    }
}
