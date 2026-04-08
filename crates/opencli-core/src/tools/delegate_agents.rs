use anyhow::{Context, Result};
use serde_json::{Value, json};

use crate::a2a::{SubAgentRequest, run_subagents_concurrent};

use super::{CoreTool, ToolDefinition, ToolExecutionContext};

pub struct DelegateAgentsTool;

impl CoreTool for DelegateAgentsTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "delegate_agents",
            description: "Delegate multiple subtasks to sub-agents concurrently and return all results",
            kind: "agent",
            requires_approval: false,
            parameters: json!({
                "type": "object",
                "properties": {
                    "tasks": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "role": { "type": "string" },
                                "task": { "type": "string" },
                                "context": { "type": "string" },
                                "allowed_tool_kinds": { "type": "array", "items": { "type": "string" } },
                                "allowed_tools": { "type": "array", "items": { "type": "string" } },
                                "max_steps": { "type": "integer", "minimum": 1 }
                            },
                            "required": ["role", "task"],
                            "additionalProperties": false
                        }
                    },
                    "max_concurrency": { "type": "integer", "minimum": 1 }
                },
                "required": ["tasks"],
                "additionalProperties": false
            }),
        }
    }

    fn execute<'a>(
        &'a self,
        args: Value,
        context: &'a ToolExecutionContext<'_>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String>> + 'a>> {
        Box::pin(async move {
            let tasks = args
                .get("tasks")
                .and_then(Value::as_array)
                .context("delegate_agents.tasks is required")?;
            let requests = tasks
                .iter()
                .map(parse_request)
                .collect::<Result<Vec<_>>>()?;
            let results = run_subagents_concurrent(
                context.config,
                requests,
                context.cancel_requested,
                context.delegation_depth,
                args.get("max_concurrency")
                    .and_then(Value::as_u64)
                    .map(|v| v as usize),
                context.agent_id.map(ToString::to_string),
            )
            .await?;
            Ok(serde_json::to_string_pretty(&results)?)
        })
    }
}

fn parse_request(value: &Value) -> Result<SubAgentRequest> {
    Ok(SubAgentRequest {
        role: value
            .get("role")
            .and_then(Value::as_str)
            .context("delegate_agents.tasks[].role is required")?
            .to_string(),
        task: value
            .get("task")
            .and_then(Value::as_str)
            .context("delegate_agents.tasks[].task is required")?
            .to_string(),
        context: value
            .get("context")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        allowed_tool_kinds: value
            .get("allowed_tool_kinds")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(ToString::to_string)
                    .collect()
            }),
        allowed_tools: value
            .get("allowed_tools")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(ToString::to_string)
                    .collect()
            }),
        max_steps: value
            .get("max_steps")
            .and_then(Value::as_u64)
            .map(|v| v as usize),
    })
}
