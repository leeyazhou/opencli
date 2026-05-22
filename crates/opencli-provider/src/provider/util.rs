use std::ops::ControlFlow;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

use anyhow::{Result, bail};
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use tokio::time::{Duration as TokioDuration, sleep};

use opencli_tools::ToolDefinition;

use crate::{message::ChatMessage, provider::ProviderConfig};

pub fn build_client(config: &ProviderConfig) -> Result<Client> {
    Ok(Client::builder()
        .timeout(Duration::from_millis(config.request_timeout_ms))
        .build()?)
}

pub async fn ensure_success(response: reqwest::Response) -> Result<reqwest::Response> {
    if response.status().is_success() {
        Ok(response)
    } else {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        bail!("provider request failed ({status}): {body}")
    }
}

pub async fn cancelable_request<F, T>(
    future: F,
    cancel_requested: Option<&Arc<AtomicBool>>,
) -> Result<T>
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

pub async fn cancelable_response_chunk(
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

pub async fn for_each_sse_data_line<F>(
    response: &mut reqwest::Response,
    cancel_requested: Option<&Arc<AtomicBool>>,
    mut on_data_line: F,
) -> Result<()>
where
    F: FnMut(&str) -> Result<ControlFlow<()>>,
{
    let mut buffer = String::new();

    while let Some(chunk) = cancelable_response_chunk(response, cancel_requested).await? {
        buffer.push_str(&String::from_utf8_lossy(&chunk));
        while let Some(index) = buffer.find('\n') {
            let line = buffer[..index].to_string();
            buffer = buffer[index + 1..].to_string();
            if let ControlFlow::Break(()) = process_sse_line(&line, &mut on_data_line)? {
                return Ok(());
            }
        }
    }

    if !buffer.is_empty()
        && let ControlFlow::Break(()) = process_sse_line(&buffer, &mut on_data_line)?
    {
        return Ok(());
    }

    Ok(())
}

fn process_sse_line<F>(line: &str, on_data_line: &mut F) -> Result<ControlFlow<()>>
where
    F: FnMut(&str) -> Result<ControlFlow<()>>,
{
    let trimmed = line.trim();
    if !trimmed.starts_with("data:") {
        return Ok(ControlFlow::Continue(()));
    }
    let payload = trimmed.trim_start_matches("data:").trim();
    on_data_line(payload)
}

pub fn serialize_openai_tool(tool: &ToolDefinition) -> Value {
    serde_json::json!({
        "type": "function",
        "function": {
            "name": tool.name,
            "description": tool.description,
            "parameters": tool.parameters,
        }
    })
}

#[derive(Debug, Deserialize)]
pub struct ModelListResponse {
    pub data: Vec<ModelListItem>,
}

#[derive(Debug, Deserialize)]
pub struct ModelListItem {
    pub id: String,
}

pub fn collect_model_ids(body: ModelListResponse) -> Vec<String> {
    body.data.into_iter().map(|item| item.id).collect()
}

pub fn serialize_openai_message(message: &ChatMessage) -> Value {
    match message.role.as_str() {
        "tool" => serde_json::json!({
            "role": "tool",
            "content": message.content,
            "tool_call_id": message.tool_call_id,
        }),
        "assistant" if !message.tool_calls.is_empty() => serde_json::json!({
            "role": "assistant",
            "content": if message.content.is_empty() { Value::Null } else { Value::String(message.content.clone()) },
            "tool_calls": message.tool_calls.iter().map(|call| serde_json::json!({
                "id": call.id,
                "type": "function",
                "function": {
                    "name": call.name,
                    "arguments": call.arguments,
                }
            })).collect::<Vec<_>>(),
        }),
        _ => serde_json::json!({
            "role": message.role,
            "content": message.content,
        }),
    }
}

#[cfg(feature = "anthropic")]
pub fn serialize_anthropic_tool(tool: &ToolDefinition) -> Value {
    serde_json::json!({
        "name": tool.name,
        "description": tool.description,
        "input_schema": tool.parameters,
    })
}

#[cfg(feature = "anthropic")]
pub fn serialize_anthropic_message(message: &ChatMessage) -> Value {
    match message.role.as_str() {
        "tool" => serde_json::json!({
            "role": "user",
            "content": [{
                "type": "tool_result",
                "tool_use_id": message.tool_call_id,
                "content": message.content,
            }],
        }),
        "assistant" if !message.tool_calls.is_empty() => {
            let mut blocks = Vec::new();
            if !message.content.is_empty() {
                blocks.push(serde_json::json!({ "type": "text", "text": message.content }));
            }
            blocks.extend(message.tool_calls.iter().map(|call| serde_json::json!({
                "type": "tool_use",
                "id": call.id,
                "name": call.name,
                "input": serde_json::from_str::<Value>(&call.arguments).unwrap_or_else(|_| serde_json::json!({})),
            })));
            serde_json::json!({ "role": "assistant", "content": blocks })
        }
        _ => serde_json::json!({
            "role": message.role,
            "content": message.content,
        }),
    }
}

#[cfg(test)]
mod tests {
    use std::ops::ControlFlow;

    use mockito::Server;

    use super::for_each_sse_data_line;

    #[tokio::test]
    async fn reads_unterminated_final_sse_line() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("GET", "/stream")
            .with_status(200)
            .with_header("content-type", "text/event-stream")
            .with_body("data: first\ndata: second")
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let mut response = client
            .get(format!("{}/stream", server.url()))
            .send()
            .await
            .expect("request should succeed");

        let mut payloads = Vec::new();
        for_each_sse_data_line(&mut response, None, |payload| {
            payloads.push(payload.to_string());
            Ok(ControlFlow::Continue(()))
        })
        .await
        .expect("helper should parse final unterminated line");

        assert_eq!(payloads, vec!["first".to_string(), "second".to_string()]);
    }

    #[tokio::test]
    async fn stops_when_callback_requests_break() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("GET", "/stream")
            .with_status(200)
            .with_header("content-type", "text/event-stream")
            .with_body("data: first\ndata: second\ndata: third\n")
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let mut response = client
            .get(format!("{}/stream", server.url()))
            .send()
            .await
            .expect("request should succeed");

        let mut payloads = Vec::new();
        for_each_sse_data_line(&mut response, None, |payload| {
            payloads.push(payload.to_string());
            if payload == "second" {
                return Ok(ControlFlow::Break(()));
            }
            Ok(ControlFlow::Continue(()))
        })
        .await
        .expect("helper should stop on break");

        assert_eq!(payloads, vec!["first".to_string(), "second".to_string()]);
    }
}
