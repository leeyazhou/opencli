use anyhow::{Context, Result};
use async_trait::async_trait;
use serde_json::{Value, json};
use std::fs;

use crate::safety::resolve_workspace_path;

use super::{Tool, ToolDefinition, ToolExecutionContext};

pub struct ReadFileTool;

#[async_trait(?Send)]
impl Tool for ReadFileTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "read_file",
            description: "Read a UTF-8 text file from the workspace",
            kind: "filesystem-read",
            requires_approval: false,
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "File path relative to workspace root" }
                },
                "required": ["path"],
                "additionalProperties": false
            }),
        }
    }

    async fn execute(&self, args: Value, context: &ToolExecutionContext<'_>) -> Result<String> {
        let path = args
            .get("path")
            .and_then(Value::as_str)
            .context("read_file.path is required")?;
        let file_path = resolve_workspace_path(context.workspace_root, path)?;
        Ok(fs::read_to_string(file_path)?)
    }
}
