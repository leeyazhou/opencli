use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::sync::{Arc, atomic::AtomicBool};

use crate::{approval::Approval, audit::AuditLogger, config::RuntimeConfig};

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

pub struct ToolExecutionContext<'a> {
    pub config: &'a RuntimeConfig,
    pub approval: &'a dyn Approval,
    pub audit: &'a dyn AuditLogger,
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
