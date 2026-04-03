use std::io::{self, IsTerminal, Read};

use anyhow::{bail, Context, Result};
use clap::CommandFactory;

use crate::cli::Cli;

pub fn join_prompt(parts: &[String]) -> Result<String> {
    if parts.is_empty() {
        let mut command = Cli::command();
        command.print_help().context("failed to print help")?;
        println!();
        bail!("missing prompt")
    }

    Ok(parts.join(" ").trim().to_string())
}

pub fn read_stdin_if_piped() -> Result<String> {
    if io::stdin().is_terminal() {
        return Ok(String::new());
    }

    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer.trim().to_string())
}
