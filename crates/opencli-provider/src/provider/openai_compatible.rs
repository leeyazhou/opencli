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
        for_each_sse_data_line, serialize_openai_message, serialize_openai_tool,
    },
};

pub struct OpenAiCompatibleProvider {
    client: Client,
    base_url: String,
    api_key: String,
    model: String,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct OpenAiChatCompletionResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: OpenAiAssistantMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAiAssistantMessage {
    content: Option<String>,
    #[serde(default)]
    tool_calls: Vec<OpenAiToolCallWire>,
}

#[derive(Debug, Clone, serde::Serialize, Deserialize, Default)]
struct OpenAiToolCallWire {
    id: String,
    function: OpenAiToolFunctionWire,
}

#[derive(Debug, Clone, serde::Serialize, Deserialize, Default)]
struct OpenAiToolFunctionWire {
    name: String,
    arguments: String,
}

impl OpenAiCompatibleProvider {
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
            temperature: config.temperature,
            max_tokens: config.max_tokens,
        })
    }
}

#[async_trait(?Send)]
impl Provider for OpenAiCompatibleProvider {
    fn name(&self) -> &'static str {
        "openai-compatible"
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
        info!(provider = "openai-compatible", model = %self.model, message_count = messages.len(), tool_count = tools.len(), "sending completion request");
        let response = cancelable_request(
            self.client
                .post(format!(
                    "{}/chat/completions",
                    self.base_url.trim_end_matches('/')
                ))
                .bearer_auth(&self.api_key)
                .json(&json!({
                    "model": self.model,
                    "messages": messages.iter().map(serialize_openai_message).collect::<Vec<_>>(),
                    "temperature": self.temperature,
                    "max_tokens": self.max_tokens,
                    "tools": tools.iter().map(serialize_openai_tool).collect::<Vec<_>>(),
                    "stream": false,
                }))
                .send(),
            cancel_requested,
        )
        .await?;

        ensure_success(response)
            .await?
            .json::<OpenAiChatCompletionResponse>()
            .await
            .context("failed to parse provider response")
            .map(|body| {
                let message = body
                    .choices
                    .into_iter()
                    .next()
                    .unwrap_or(OpenAiChoice {
                        message: OpenAiAssistantMessage {
                            content: Some(String::new()),
                            tool_calls: Vec::new(),
                        },
                    })
                    .message;

                ChatResponse {
                    content: message.content.unwrap_or_default(),
                    tool_calls: message
                        .tool_calls
                        .into_iter()
                        .map(|call| ToolCall {
                            id: call.id,
                            name: call.function.name,
                            arguments: call.function.arguments,
                        })
                        .collect(),
                }
            })
    }

    async fn stream_text(
        &self,
        messages: &[ChatMessage],
        renderer: &mut dyn Renderer,
        cancel_requested: Option<&Arc<AtomicBool>>,
    ) -> Result<String> {
        info!(provider = "openai-compatible", model = %self.model, message_count = messages.len(), "starting streaming request");
        let response = cancelable_request(
            self.client
                .post(format!(
                    "{}/chat/completions",
                    self.base_url.trim_end_matches('/')
                ))
                .bearer_auth(&self.api_key)
                .json(&json!({
                    "model": self.model,
                    "messages": messages.iter().map(serialize_openai_message).collect::<Vec<_>>(),
                    "temperature": self.temperature,
                    "max_tokens": self.max_tokens,
                    "stream": true,
                }))
                .send(),
            cancel_requested,
        )
        .await?;

        let mut response = ensure_success(response).await?;
        let mut output = String::new();

        for_each_sse_data_line(&mut response, cancel_requested, |payload| {
            if payload == "[DONE]" {
                renderer.render_line("")?;
                return Ok(ControlFlow::Break(()));
            }
            let parsed: Value = match serde_json::from_str(payload) {
                Ok(value) => value,
                Err(_) => return Ok(ControlFlow::Continue(())),
            };
            if let Some(text) = parsed
                .pointer("/choices/0/delta/content")
                .and_then(Value::as_str)
            {
                debug!(
                    provider = "openai-compatible",
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
        info!(provider = "openai-compatible", "listing models");
        let response = self
            .client
            .get(format!("{}/models", self.base_url.trim_end_matches('/')))
            .bearer_auth(&self.api_key)
            .send()
            .await?;
        let body = ensure_success(response)
            .await?
            .json::<ModelListResponse>()
            .await
            .context("failed to parse models response")?;
        Ok(collect_model_ids(body))
    }
}

#[cfg(test)]
mod tests {
    use mockito::Server;

    use super::*;

    #[tokio::test]
    async fn lists_models_from_mock_server() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("GET", "/models")
            .match_header("authorization", "Bearer test-key")
            .with_status(200)
            .with_body(r#"{"data":[{"id":"gpt-test"}]}"#)
            .create_async()
            .await;

        let config = ProviderConfig {
            base_url: server.url(),
            api_key: "test-key".into(),
            model: "gpt-4.1".into(),
            anthropic_version: "2023-06-01".into(),
            temperature: 0.2,
            max_tokens: 4096,
            request_timeout_ms: 120_000,
        };
        let provider = OpenAiCompatibleProvider::from_config(&config)
            .expect("provider config should be valid");
        let models = provider
            .list_models()
            .await
            .expect("model listing should succeed");
        assert_eq!(models, vec!["gpt-test".to_string()]);
    }
}
