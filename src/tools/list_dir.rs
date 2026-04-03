use anyhow::Result;
use async_trait::async_trait;
use serde_json::{Value, json};
use std::fs;

use crate::safety::resolve_workspace_path;

use super::{Tool, ToolDefinition, ToolExecutionContext};

pub struct ListDirTool;

#[async_trait(?Send)]
impl Tool for ListDirTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "list_dir",
            description: "List directory entries under the workspace",
            kind: "filesystem-read",
            requires_approval: false,
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Directory path relative to workspace root" }
                },
                "additionalProperties": false
            }),
        }
    }

    async fn execute(&self, args: Value, context: &ToolExecutionContext<'_>) -> Result<String> {
        let path = args.get("path").and_then(Value::as_str).unwrap_or(".");
        let dir_path = resolve_workspace_path(&context.config.workspace_root, path)?;
        let mut entries = fs::read_dir(dir_path)?.collect::<Result<Vec<_>, _>>()?;
        entries.sort_by_key(|entry| entry.file_name());
        Ok(entries.into_iter().map(|entry| {
            let ty = entry.file_type().ok();
            let suffix = if ty.as_ref().is_some_and(|file_type| file_type.is_dir()) { "/" } else { "" };
            format!("{}{}", entry.file_name().to_string_lossy(), suffix)
        }).collect::<Vec<_>>().join("\n"))
    }
}
