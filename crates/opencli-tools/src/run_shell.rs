use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use serde_json::{Value, json};
use std::{
    process::{Command, Stdio},
    sync::atomic::Ordering,
    time::Duration,
};
use wait_timeout::ChildExt;

use crate::safety::{approve_shell, resolve_workspace_path};

use super::{Tool, ToolDefinition, ToolExecutionContext};

pub struct RunShellTool;

#[async_trait(?Send)]
impl Tool for RunShellTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "run_shell",
            description: "Run a shell command inside the workspace with approval checks",
            kind: "shell",
            requires_approval: true,
            parameters: json!({
                "type": "object",
                "properties": {
                    "command": { "type": "string" },
                    "workdir": { "type": "string", "description": "Working directory relative to workspace root" }
                },
                "required": ["command"],
                "additionalProperties": false
            }),
        }
    }

    async fn execute(&self, args: Value, context: &ToolExecutionContext<'_>) -> Result<String> {
        let command = args
            .get("command")
            .and_then(Value::as_str)
            .context("run_shell.command is required")?;
        let workdir = args.get("workdir").and_then(Value::as_str).unwrap_or(".");
        approve_shell(
            command,
            context.approval_mode,
            context.non_interactive_approval,
        )?;
        let cwd = resolve_workspace_path(context.workspace_root, workdir)?;

        let output =
            Command::new(std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string()))
                .arg("-lc")
                .arg(command)
                .current_dir(cwd)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .context("failed to run shell command")?;

        let output = wait_for_output(output, context.shell_timeout_ms, context.cancel_requested)?;

        Ok(serde_json::to_string_pretty(&json!({
            "stdout": String::from_utf8_lossy(&output.stdout).trim(),
            "stderr": String::from_utf8_lossy(&output.stderr).trim(),
            "exitCode": output.status.code(),
        }))?)
    }
}

fn wait_for_output(
    mut child: std::process::Child,
    timeout_ms: u64,
    cancel_requested: Option<&std::sync::Arc<std::sync::atomic::AtomicBool>>,
) -> Result<std::process::Output> {
    let deadline = std::time::Instant::now() + Duration::from_millis(timeout_ms);

    loop {
        if cancel_requested.is_some_and(|flag| flag.load(Ordering::Relaxed)) {
            child.kill().ok();
            child.wait().ok();
            bail!("agent execution canceled")
        }

        if std::time::Instant::now() >= deadline {
            child.kill().ok();
            child.wait().ok();
            bail!("command timed out after {}ms", timeout_ms)
        }

        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        let poll_slice = remaining.min(Duration::from_millis(100));
        match child.wait_timeout(poll_slice)? {
            Some(_) => return Ok(child.wait_with_output()?),
            None => continue,
        }
    }
}
