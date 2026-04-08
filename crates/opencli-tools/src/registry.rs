use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use tracing::{debug, info, warn};

use super::{
    ListDirTool, ReadFileTool, RunShellTool, SearchFilesTool, Tool, ToolCall, ToolDefinition,
    ToolExecutionContext,
};

pub struct ToolRegistry {
    tools: Vec<Box<dyn Tool>>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: vec![
                Box::new(ReadFileTool),
                Box::new(ListDirTool),
                Box::new(SearchFilesTool),
                Box::new(RunShellTool),
            ],
        }
    }

    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools.iter().map(|tool| tool.definition()).collect()
    }

    pub async fn execute(
        &self,
        call: &ToolCall,
        context: &ToolExecutionContext<'_>,
    ) -> Result<String> {
        let args: Value = if call.arguments.trim().is_empty() {
            json!({})
        } else {
            serde_json::from_str(&call.arguments)
                .with_context(|| format!("invalid JSON arguments for tool {}", call.name))?
        };

        let tool = self
            .tools
            .iter()
            .find(|tool| tool.definition().name == call.name)
            .with_context(|| format!("tool not found: {}", call.name))?;

        let definition = tool.definition();
        if !context
            .allowed_tool_kinds
            .iter()
            .any(|kind| kind == definition.kind)
        {
            warn!(
                tool = definition.name,
                kind = definition.kind,
                "tool kind blocked by policy"
            );
            bail!("tool kind is blocked by policy: {}", definition.kind);
        }
        if !context.allowed_tools.is_empty()
            && !context
                .allowed_tools
                .iter()
                .any(|tool_name| tool_name == definition.name)
        {
            warn!(tool = definition.name, "tool blocked by name policy");
            bail!("tool is blocked by policy: {}", definition.name);
        }

        info!(
            tool = definition.name,
            kind = definition.kind,
            requires_approval = definition.requires_approval,
            "starting tool execution"
        );

        context.audit.log(
            "tool_start",
            definition.name,
            json!({
                "agentId": context.agent_id,
                "parentAgentId": context.parent_agent_id,
                "kind": definition.kind,
                "requiresApproval": definition.requires_approval,
                "arguments": args,
            }),
        )?;

        let result = tool.execute(args.clone(), context).await;

        context.audit.log(
            "tool_finish",
            definition.name,
            match &result {
                Ok(output) => json!({
                    "agentId": context.agent_id,
                    "parentAgentId": context.parent_agent_id,
                    "ok": true,
                    "outputPreview": output.chars().take(200).collect::<String>(),
                }),
                Err(error) => json!({
                    "agentId": context.agent_id,
                    "parentAgentId": context.parent_agent_id,
                    "ok": false,
                    "error": format!("{error:#}"),
                }),
            },
        )?;

        debug!(tool = definition.name, "tool execution completed");

        result
    }
}
