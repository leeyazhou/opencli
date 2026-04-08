use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use glob::Pattern;
use serde_json::{Value, json};
use std::process::{Command, Stdio};

use crate::safety::resolve_workspace_path;

use super::{Tool, ToolDefinition, ToolExecutionContext};

pub struct SearchFilesTool;

#[async_trait(?Send)]
impl Tool for SearchFilesTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "search_files",
            description: "Search file contents in the workspace",
            kind: "filesystem-search",
            requires_approval: false,
            parameters: json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string" },
                    "path": { "type": "string" },
                    "include": { "type": "string", "description": "Optional glob like *.rs" }
                },
                "required": ["query"],
                "additionalProperties": false
            }),
        }
    }

    async fn execute(&self, args: Value, context: &ToolExecutionContext<'_>) -> Result<String> {
        let query = args
            .get("query")
            .and_then(Value::as_str)
            .context("search_files.query is required")?;
        let path = args.get("path").and_then(Value::as_str).unwrap_or(".");
        let include = args.get("include").and_then(Value::as_str);
        let search_path = resolve_workspace_path(context.workspace_root, path)?;

        let output = Command::new("rg")
            .arg("--line-number")
            .arg("--color")
            .arg("never")
            .args(include.into_iter().flat_map(|pattern| ["--glob", pattern]))
            .arg(query)
            .arg(&search_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .context("failed to run rg")?;

        match output.status.code() {
            Some(0) => {
                let mut text = String::from_utf8_lossy(&output.stdout).to_string();
                if let Some(glob_pattern) = include {
                    text = filter_search_output_by_glob(&text, glob_pattern);
                }
                Ok(text.trim().to_string())
            }
            Some(1) => Ok("No matches found.".to_string()),
            _ => bail!(String::from_utf8_lossy(&output.stderr).trim().to_string()),
        }
    }
}

fn filter_search_output_by_glob(text: &str, include: &str) -> String {
    let pattern = match Pattern::new(include) {
        Ok(pattern) => pattern,
        Err(_) => return text.to_string(),
    };

    text.lines()
        .filter(|line| {
            let path = line.split(':').next().unwrap_or_default();
            pattern.matches(path)
                || path
                    .rsplit('/')
                    .next()
                    .is_some_and(|name| pattern.matches(name))
        })
        .collect::<Vec<_>>()
        .join("\n")
}
