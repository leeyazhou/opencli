use std::io::{self, IsTerminal, Write};

use anyhow::{Result, bail};

use crate::{
    config::RuntimeConfig,
    safety::{CommandRisk, classify_command},
};

pub trait Approval: Send + Sync {
    fn approve_shell(&self, command: &str, config: &RuntimeConfig) -> Result<()>;
}

pub struct PolicyApproval;

impl PolicyApproval {
    pub fn new() -> Self {
        Self
    }
}

impl Approval for PolicyApproval {
    fn approve_shell(&self, command: &str, config: &RuntimeConfig) -> Result<()> {
        let risk = classify_command(command);

        if risk == CommandRisk::Dangerous {
            bail!("blocked dangerous command: {command}");
        }

        if io::stdin().is_terminal() && io::stdout().is_terminal() {
            return approve_interactive(command, config, risk);
        }

        approve_non_interactive(command, config, risk)
    }
}

fn approve_interactive(command: &str, config: &RuntimeConfig, risk: CommandRisk) -> Result<()> {
    match config.approval_mode.as_str() {
        "never-ask" => Ok(()),
        "on-write" if risk == CommandRisk::Read => Ok(()),
        "always-ask" | "on-write" => prompt_for_approval(command),
        other => bail!("unsupported approvalMode: {other}"),
    }
}

fn approve_non_interactive(command: &str, config: &RuntimeConfig, risk: CommandRisk) -> Result<()> {
    if config.approval_mode == "never-ask" {
        return Ok(());
    }

    if config.approval_mode == "on-write" && risk == CommandRisk::Read {
        return Ok(());
    }

    match config.non_interactive_approval.as_str() {
        "deny" => bail!("command requires approval in non-interactive mode: {command}"),
        "allow-read-only" if risk == CommandRisk::Read => Ok(()),
        "allow-all" => Ok(()),
        other => bail!("unsupported nonInteractiveApproval: {other}"),
    }
}

fn prompt_for_approval(command: &str) -> Result<()> {
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
