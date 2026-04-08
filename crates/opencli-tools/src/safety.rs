use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommandRisk {
    Read,
    Write,
    Dangerous,
}

pub fn resolve_workspace_path(workspace_root: &str, target_path: &str) -> Result<PathBuf> {
    let root = Path::new(workspace_root).canonicalize()?;
    let candidate = root.join(target_path);
    let normalized = normalize_path(&candidate);

    if normalized.starts_with(&root) {
        Ok(normalized)
    } else {
        bail!("path is outside workspace: {target_path}")
    }
}

pub fn approve_shell(
    command: &str,
    approval_mode: &str,
    non_interactive_approval: &str,
) -> Result<()> {
    let risk = classify_command(command);

    if risk == CommandRisk::Dangerous {
        bail!("blocked dangerous command: {command}");
    }

    if std::io::stdin().is_terminal() && std::io::stdout().is_terminal() {
        return approve_interactive(command, approval_mode, risk);
    }

    approve_non_interactive(command, approval_mode, non_interactive_approval, risk)
}

fn classify_command(command: &str) -> CommandRisk {
    let lowered = command.to_lowercase();

    for pattern in ["sudo ", "rm -rf /", "mkfs", " dd ", "shutdown", "reboot"] {
        if lowered.contains(pattern) {
            return CommandRisk::Dangerous;
        }
    }

    for pattern in [
        "rm ",
        "mv ",
        "cp ",
        "mkdir ",
        "touch ",
        "npm install",
        "pnpm add",
        "cargo add",
        "git add",
        "git commit",
        "git restore",
        "git clean",
        ">",
        "tee ",
    ] {
        if lowered.contains(pattern) {
            return CommandRisk::Write;
        }
    }

    CommandRisk::Read
}

fn approve_interactive(command: &str, approval_mode: &str, risk: CommandRisk) -> Result<()> {
    match approval_mode {
        "never-ask" => Ok(()),
        "on-write" if risk == CommandRisk::Read => Ok(()),
        "always-ask" | "on-write" => prompt_for_approval(command),
        other => bail!("unsupported approvalMode: {other}"),
    }
}

fn approve_non_interactive(
    command: &str,
    approval_mode: &str,
    non_interactive_approval: &str,
    risk: CommandRisk,
) -> Result<()> {
    if approval_mode == "never-ask" {
        return Ok(());
    }

    if approval_mode == "on-write" && risk == CommandRisk::Read {
        return Ok(());
    }

    match non_interactive_approval {
        "deny" => bail!("command requires approval in non-interactive mode: {command}"),
        "allow-read-only" if risk == CommandRisk::Read => Ok(()),
        "allow-all" => Ok(()),
        other => bail!("unsupported nonInteractiveApproval: {other}"),
    }
}

fn prompt_for_approval(command: &str) -> Result<()> {
    use std::io::{self, Write};

    print!("Approve shell command? {command} [y/N] ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let answer = input.trim().to_lowercase();

    if answer == "y" || answer == "yes" {
        Ok(())
    } else {
        bail!("command rejected by user: {command}")
    }
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            std::path::Component::CurDir => {}
            other => normalized.push(other.as_os_str()),
        }
    }

    normalized
}
