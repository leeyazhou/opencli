mod delegate_agent;
mod delegate_agents;
use anyhow::{Context, Result};
use opencli_audit::{AuditEvent, AuditLogger};
use opencli_config::RuntimeConfig;
use serde_json::{Value, json};

pub use opencli_tools::{ToolCall, ToolDefinition};

use self::{delegate_agent::DelegateAgentTool, delegate_agents::DelegateAgentsTool};

pub struct ToolExecutionContext<'a> {
    pub config: &'a RuntimeConfig,
    pub audit: &'a dyn AuditLogger,
    pub cancel_requested: Option<&'a std::sync::Arc<std::sync::atomic::AtomicBool>>,
    pub delegation_depth: u32,
    pub agent_id: Option<&'a str>,
    pub parent_agent_id: Option<&'a str>,
}

trait CoreTool {
    fn definition(&self) -> ToolDefinition;
    fn matches(&self, name: &str) -> bool {
        self.definition().name == name
    }
    fn execute<'a>(
        &'a self,
        args: Value,
        context: &'a ToolExecutionContext<'_>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String>> + 'a>>;
}

struct AuditAdapter<'a> {
    config: &'a RuntimeConfig,
    audit: &'a dyn AuditLogger,
}

impl opencli_tools::ToolAuditLogger for AuditAdapter<'_> {
    fn log(&self, event_type: &str, tool_name: &str, payload: Value) -> Result<()> {
        self.audit.log(
            self.config,
            AuditEvent {
                event_type,
                tool_name,
                payload,
            },
        )
    }
}

pub struct ToolRegistry {
    generic: opencli_tools::ToolRegistry,
    local: Vec<Box<dyn CoreTool>>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            generic: opencli_tools::ToolRegistry::new(),
            local: vec![Box::new(DelegateAgentTool), Box::new(DelegateAgentsTool)],
        }
    }

    pub fn definitions(&self) -> Vec<ToolDefinition> {
        let mut definitions = self.generic.definitions();
        definitions.extend(self.local.iter().map(|tool| tool.definition()));
        definitions
    }

    pub async fn execute(
        &self,
        call: &ToolCall,
        context: &ToolExecutionContext<'_>,
    ) -> Result<String> {
        if let Some(tool) = self.local.iter().find(|tool| tool.matches(&call.name)) {
            let args: Value = if call.arguments.trim().is_empty() {
                json!({})
            } else {
                serde_json::from_str(&call.arguments)
                    .with_context(|| format!("invalid JSON arguments for tool {}", call.name))?
            };
            return tool.execute(args, context).await;
        }

        let audit = AuditAdapter {
            config: context.config,
            audit: context.audit,
        };
        let generic_context = opencli_tools::ToolExecutionContext {
            workspace_root: &context.config.workspace_root,
            allowed_tool_kinds: &context.config.allowed_tool_kinds,
            allowed_tools: &context.config.allowed_tools,
            approval_mode: &context.config.approval_mode,
            non_interactive_approval: &context.config.non_interactive_approval,
            shell_timeout_ms: context.config.shell_timeout_ms,
            audit: &audit,
            cancel_requested: context.cancel_requested,
            delegation_depth: context.delegation_depth,
            agent_id: context.agent_id,
            parent_agent_id: context.parent_agent_id,
        };
        self.generic.execute(call, &generic_context).await
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use std::fs;
    use uuid::Uuid;

    use opencli_audit::FileAuditLogger;
    use opencli_config::RuntimeConfig;

    use super::{ToolCall, ToolExecutionContext, ToolRegistry};

    #[tokio::test]
    async fn reads_file_tool() {
        let root = std::env::temp_dir().join(format!("opencli-tools-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).expect("tool test root should be created");
        fs::write(root.join("a.txt"), "hello").expect("tool test file should be written");
        let config = RuntimeConfig {
            workspace_root: root.to_string_lossy().to_string(),
            ..RuntimeConfig::default()
        };
        let registry = ToolRegistry::new();
        let audit = FileAuditLogger::new();
        let context = ToolExecutionContext {
            config: &config,
            audit: &audit,
            cancel_requested: None,
            delegation_depth: 0,
            agent_id: None,
            parent_agent_id: None,
        };

        let result = registry
            .execute(
                &ToolCall {
                    id: "1".into(),
                    name: "read_file".into(),
                    arguments: json!({"path":"a.txt"}).to_string(),
                },
                &context,
            )
            .await
            .expect("read_file tool should succeed");

        assert_eq!(result, "hello");
    }
}
