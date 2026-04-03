use std::sync::{Arc, atomic::AtomicBool};
use std::time::Instant;

use anyhow::{Result, bail};
use futures_util::{FutureExt, StreamExt, stream::FuturesUnordered};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    agent::run_agent_loop,
    approval::PolicyApproval,
    audit::{AuditEvent, AuditLogger, FileAuditLogger},
    config::RuntimeConfig,
    message::ChatMessage,
    output::BufferRenderer,
    provider_factory::ProviderFactory,
    tools::{ToolExecutionContext, ToolRegistry},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubAgentRequest {
    pub role: String,
    pub task: String,
    pub context: Option<String>,
    pub allowed_tool_kinds: Option<Vec<String>>,
    pub allowed_tools: Option<Vec<String>>,
    pub max_steps: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubAgentResult {
    pub agent_id: String,
    pub parent_agent_id: Option<String>,
    pub role: String,
    pub task: String,
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub duration_ms: u128,
}

pub async fn run_subagent(
    config: &RuntimeConfig,
    request: SubAgentRequest,
    cancel_requested: Option<&Arc<AtomicBool>>,
    delegation_depth: u32,
    parent_agent_id: Option<String>,
) -> Result<SubAgentResult> {
    if !config.a2a_enabled {
        bail!("a2a is disabled by configuration");
    }
    if delegation_depth >= config.a2a_max_depth {
        bail!("a2a delegation depth exceeded limit {}", config.a2a_max_depth);
    }

    let mut child_config = config.clone();
    let agent_id = Uuid::new_v4().to_string();
    if let Some(kinds) = request.allowed_tool_kinds.clone() {
        child_config.allowed_tool_kinds = kinds;
    }
    if let Some(tools) = request.allowed_tools.clone() {
        child_config.allowed_tools = tools;
    }

    let provider = ProviderFactory::new().create(&child_config)?;
    let approval = PolicyApproval::new();
    let audit = FileAuditLogger::new();
    let started_at = Instant::now();
    let tool_registry = ToolRegistry::new();
    let mut renderer = BufferRenderer::new();
    let tool_context = ToolExecutionContext {
        config: &child_config,
        approval: &approval,
        audit: &audit,
        cancel_requested,
        delegation_depth: delegation_depth + 1,
        agent_id: Some(agent_id.as_str()),
        parent_agent_id: parent_agent_id.as_deref(),
    };

    let system = build_subagent_system_prompt(&request.role);
    let user = build_subagent_user_prompt(&request.task, request.context.as_deref());
    let mut messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: system,
            tool_call_id: None,
            name: None,
            tool_calls: Vec::new(),
        },
        ChatMessage::user(user),
    ];

    audit.log(&child_config, AuditEvent {
        event_type: "a2a_start",
        tool_name: "subagent",
        payload: serde_json::json!({
            "agentId": agent_id,
            "parentAgentId": parent_agent_id,
            "role": request.role,
            "task": request.task,
        }),
    })?;

    let result = run_agent_loop(
        provider.as_ref(),
        &tool_registry,
        &mut renderer,
        &tool_context,
        cancel_requested,
        request.max_steps.unwrap_or(child_config.agent_max_steps),
        &mut messages,
    )
    .await;

    let duration_ms = started_at.elapsed().as_millis();
    let output = match &result {
        Ok(output) => Some(output.clone()),
        Err(_) => None,
    };
    let error = result.as_ref().err().map(|err| format!("{err:#}"));
    let subagent_result = SubAgentResult {
        agent_id: agent_id.clone(),
        parent_agent_id: parent_agent_id.clone(),
        role: request.role.clone(),
        task: request.task.clone(),
        success: result.is_ok(),
        output,
        error,
        duration_ms,
    };

    audit.log(&child_config, AuditEvent {
        event_type: "a2a_finish",
        tool_name: "subagent",
        payload: serde_json::json!({
            "agentId": agent_id,
            "parentAgentId": parent_agent_id,
            "role": request.role,
            "task": request.task,
            "success": subagent_result.success,
            "durationMs": duration_ms,
            "error": subagent_result.error,
        }),
    })?;

    Ok(subagent_result)
}

pub async fn run_subagents_concurrent(
    config: &RuntimeConfig,
    requests: Vec<SubAgentRequest>,
    cancel_requested: Option<&Arc<AtomicBool>>,
    delegation_depth: u32,
    max_concurrency: Option<usize>,
    parent_agent_id: Option<String>,
) -> Result<Vec<SubAgentResult>> {
    if requests.is_empty() {
        return Ok(Vec::new());
    }

    let limit = max_concurrency.unwrap_or(config.a2a_max_concurrency).max(1);
    let mut futures = FuturesUnordered::new();
    let mut iter = requests.into_iter().enumerate();
    let mut results: Vec<Option<SubAgentResult>> = Vec::new();

    while futures.len() < limit {
        if let Some((index, request)) = iter.next() {
            if results.len() <= index {
                results.resize(index + 1, None);
            }
            let parent = parent_agent_id.clone();
            futures.push(async move { (index, run_subagent(config, request, cancel_requested, delegation_depth, parent).await) }.boxed_local());
        } else {
            break;
        }
    }

    while let Some((index, result)) = futures.next().await {
        results[index] = Some(match result {
            Ok(value) => value,
            Err(error) => SubAgentResult {
                agent_id: Uuid::new_v4().to_string(),
                parent_agent_id: parent_agent_id.clone(),
                role: "unknown".to_string(),
                task: "unknown".to_string(),
                success: false,
                output: None,
                error: Some(format!("{error:#}")),
                duration_ms: 0,
            },
        });

        if let Some((next_index, request)) = iter.next() {
            if results.len() <= next_index {
                results.resize(next_index + 1, None);
            }
            let parent = parent_agent_id.clone();
            futures.push(async move { (next_index, run_subagent(config, request, cancel_requested, delegation_depth, parent).await) }.boxed_local());
        }
    }

    Ok(results.into_iter().flatten().collect())
}

fn build_subagent_system_prompt(role: &str) -> String {
    format!(
        "You are a delegated sub-agent working for a parent coding agent. Your role is: {role}. Focus only on the assigned task, use tools when necessary, and return a concise but complete result for the parent agent."
    )
}

fn build_subagent_user_prompt(task: &str, context: Option<&str>) -> String {
    match context {
        Some(context) if !context.trim().is_empty() => {
            format!("Task:\n{task}\n\nContext:\n{context}")
        }
        _ => task.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use mockito::{Matcher, Server};

    use super::*;

    #[tokio::test]
    async fn run_subagent_returns_structured_result() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("POST", "/chat/completions")
            .match_header("authorization", "Bearer test-key")
            .with_status(200)
            .with_body(r#"{"choices":[{"message":{"content":"subagent output","tool_calls":[]}}]}"#)
            .create_async()
            .await;

        let config = RuntimeConfig {
            base_url: server.url(),
            api_key: "test-key".into(),
            ..RuntimeConfig::default()
        };

        let result = run_subagent(
            &config,
            SubAgentRequest {
                role: "researcher".into(),
                task: "inspect module".into(),
                context: None,
                allowed_tool_kinds: Some(vec!["filesystem-read".into()]),
                allowed_tools: None,
                max_steps: Some(1),
            },
            None,
            0,
            Some("parent-1".into()),
        )
        .await
        .unwrap();

        assert_eq!(result.parent_agent_id.as_deref(), Some("parent-1"));
        assert_eq!(result.role, "researcher");
        assert_eq!(result.task, "inspect module");
        assert_eq!(result.output.as_deref(), Some("subagent output"));
        assert!(result.success);
        assert!(result.error.is_none());
        assert!(!result.agent_id.is_empty());
    }

    #[tokio::test]
    async fn run_subagents_concurrent_preserves_request_order() {
        let mut server = Server::new_async().await;
        let _first = server
            .mock("POST", "/chat/completions")
            .match_body(Matcher::Regex("first task".into()))
            .with_status(200)
            .with_body(r#"{"choices":[{"message":{"content":"first output","tool_calls":[]}}]}"#)
            .create_async()
            .await;
        let _second = server
            .mock("POST", "/chat/completions")
            .match_body(Matcher::Regex("second task".into()))
            .with_status(200)
            .with_body(r#"{"choices":[{"message":{"content":"second output","tool_calls":[]}}]}"#)
            .create_async()
            .await;

        let config = RuntimeConfig {
            base_url: server.url(),
            api_key: "test-key".into(),
            ..RuntimeConfig::default()
        };

        let results = run_subagents_concurrent(
            &config,
            vec![
                SubAgentRequest {
                    role: "researcher".into(),
                    task: "first task".into(),
                    context: None,
                    allowed_tool_kinds: None,
                    allowed_tools: None,
                    max_steps: Some(1),
                },
                SubAgentRequest {
                    role: "researcher".into(),
                    task: "second task".into(),
                    context: None,
                    allowed_tool_kinds: None,
                    allowed_tools: None,
                    max_steps: Some(1),
                },
            ],
            None,
            0,
            Some(2),
            Some("parent-batch".into()),
        )
        .await
        .unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].task, "first task");
        assert_eq!(results[0].output.as_deref(), Some("first output"));
        assert_eq!(results[0].parent_agent_id.as_deref(), Some("parent-batch"));
        assert_eq!(results[1].task, "second task");
        assert_eq!(results[1].output.as_deref(), Some("second output"));
        assert_eq!(results[1].parent_agent_id.as_deref(), Some("parent-batch"));
    }

    #[tokio::test]
    async fn run_subagents_concurrent_returns_partial_failures() {
        let mut server = Server::new_async().await;
        let _ok = server
            .mock("POST", "/chat/completions")
            .match_body(Matcher::Regex("ok task".into()))
            .with_status(200)
            .with_body(r#"{"choices":[{"message":{"content":"ok output","tool_calls":[]}}]}"#)
            .create_async()
            .await;
        let _fail = server
            .mock("POST", "/chat/completions")
            .match_body(Matcher::Regex("bad task".into()))
            .with_status(500)
            .with_body(r#"{"error":"boom"}"#)
            .create_async()
            .await;

        let config = RuntimeConfig {
            base_url: server.url(),
            api_key: "test-key".into(),
            ..RuntimeConfig::default()
        };

        let results = run_subagents_concurrent(
            &config,
            vec![
                SubAgentRequest {
                    role: "researcher".into(),
                    task: "ok task".into(),
                    context: None,
                    allowed_tool_kinds: None,
                    allowed_tools: None,
                    max_steps: Some(1),
                },
                SubAgentRequest {
                    role: "researcher".into(),
                    task: "bad task".into(),
                    context: None,
                    allowed_tool_kinds: None,
                    allowed_tools: None,
                    max_steps: Some(1),
                },
            ],
            None,
            0,
            Some(2),
            Some("parent-batch".into()),
        )
        .await
        .unwrap();

        assert_eq!(results.len(), 2);
        assert!(results[0].success);
        assert_eq!(results[0].output.as_deref(), Some("ok output"));
        assert!(!results[1].success);
        assert!(results[1].error.is_some());
    }
}
