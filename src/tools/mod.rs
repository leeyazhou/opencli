mod delegate_agent;
mod delegate_agents;
mod list_dir;
mod read_file;
mod registry;
mod run_shell;
mod search_files;
mod types;

pub use delegate_agent::DelegateAgentTool;
pub use delegate_agents::DelegateAgentsTool;
pub use list_dir::ListDirTool;
pub use read_file::ReadFileTool;
pub use registry::ToolRegistry;
pub use run_shell::RunShellTool;
pub use search_files::SearchFilesTool;
pub use types::{Tool, ToolCall, ToolDefinition, ToolExecutionContext};

#[cfg(test)]
mod tests {
    use serde_json::json;
    use uuid::Uuid;
    use std::fs;

    use crate::{approval::PolicyApproval, audit::FileAuditLogger, config::RuntimeConfig};

    use super::{ToolCall, ToolExecutionContext, ToolRegistry};

    #[tokio::test]
    async fn reads_file_tool() {
        let root = std::env::temp_dir().join(format!("ai-cli-tools-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("a.txt"), "hello").unwrap();
        let config = RuntimeConfig {
            workspace_root: root.to_string_lossy().to_string(),
            ..RuntimeConfig::default()
        };
        let registry = ToolRegistry::new();
        let approval = PolicyApproval::new();
        let audit = FileAuditLogger::new();
        let context = ToolExecutionContext { config: &config, approval: &approval, audit: &audit, cancel_requested: None, delegation_depth: 0, agent_id: None, parent_agent_id: None };

        let result = registry.execute(&ToolCall {
            id: "1".into(),
            name: "read_file".into(),
            arguments: json!({"path":"a.txt"}).to_string(),
        }, &context).await.unwrap();

        assert_eq!(result, "hello");
    }
}
