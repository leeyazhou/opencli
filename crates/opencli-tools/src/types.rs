use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::sync::{Arc, atomic::AtomicBool};

#[derive(Debug, Clone)]
pub struct ToolDefinition {
    pub name: &'static str,
    pub description: &'static str,
    pub kind: &'static str,
    pub requires_approval: bool,
    pub parameters: Value,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

pub trait ToolAuditLogger: Send + Sync {
    fn log(&self, event_type: &str, tool_name: &str, payload: Value) -> Result<()>;
}

pub struct ToolExecutionContext<'a> {
    pub workspace_root: &'a str,
    pub allowed_tool_kinds: &'a [String],
    pub allowed_tools: &'a [String],
    pub approval_mode: &'a str,
    pub non_interactive_approval: &'a str,
    pub shell_timeout_ms: u64,
    pub audit: &'a dyn ToolAuditLogger,
    pub cancel_requested: Option<&'a Arc<AtomicBool>>,
    pub delegation_depth: u32,
    pub agent_id: Option<&'a str>,
    pub parent_agent_id: Option<&'a str>,
}

#[async_trait(?Send)]
pub trait Tool: Send + Sync {
    fn definition(&self) -> ToolDefinition;
    async fn execute(&self, args: Value, context: &ToolExecutionContext<'_>) -> Result<String>;
}
