use anyhow::{Context, Result};
use serde_json::{Value, json};

use crate::a2a::{SubAgentRequest, run_subagent};

use super::{CoreTool, ToolDefinition, ToolExecutionContext};

pub struct DelegateAgentTool;

impl CoreTool for DelegateAgentTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "delegate_agent",
            description: "Delegate a focused subtask to a sub-agent and return its result",
            kind: "agent",
            requires_approval: false,
            parameters: json!({
                "type": "object",
                "properties": {
                    "role": { "type": "string", "description": "Role name for the delegated agent" },
                    "task": { "type": "string", "description": "Focused subtask for the delegated agent" },
                    "context": { "type": "string", "description": "Optional extra context for the subtask" },
                    "allowed_tool_kinds": { "type": "array", "items": { "type": "string" } },
                    "allowed_tools": { "type": "array", "items": { "type": "string" } },
                    "max_steps": { "type": "integer", "minimum": 1 }
                },
                "required": ["role", "task"],
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
            let role = args
                .get("role")
                .and_then(Value::as_str)
                .context("delegate_agent.role is required")?;
            let task = args
                .get("task")
                .and_then(Value::as_str)
                .context("delegate_agent.task is required")?;
            let request = SubAgentRequest {
                role: role.to_string(),
                task: task.to_string(),
                context: args
                    .get("context")
                    .and_then(Value::as_str)
                    .map(ToString::to_string),
                allowed_tool_kinds: args
                    .get("allowed_tool_kinds")
                    .and_then(Value::as_array)
                    .map(|items| {
                        items
                            .iter()
                            .filter_map(Value::as_str)
                            .map(ToString::to_string)
                            .collect()
                    }),
                allowed_tools: args
                    .get("allowed_tools")
                    .and_then(Value::as_array)
                    .map(|items| {
                        items
                            .iter()
                            .filter_map(Value::as_str)
                            .map(ToString::to_string)
                            .collect()
                    }),
                max_steps: args
                    .get("max_steps")
                    .and_then(Value::as_u64)
                    .map(|v| v as usize),
            };

            let result = run_subagent(
                context.config,
                request,
                context.cancel_requested,
                context.delegation_depth,
                context.agent_id.map(ToString::to_string),
            )
            .await?;

            Ok(serde_json::to_string_pretty(&result)?)
        })
    }
}
