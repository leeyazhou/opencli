use std::time::Duration;

use anyhow::{Result, bail};
use reqwest::Client;
use serde_json::Value;

use crate::{config::RuntimeConfig, message::ChatMessage, tools::ToolDefinition};

pub fn build_client(config: &RuntimeConfig) -> Result<Client> {
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
