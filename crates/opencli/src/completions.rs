use std::io;

use anyhow::Result;
use clap::{CommandFactory, ValueEnum};
use clap_complete::{Shell, generate};

use crate::cli::Cli;

#[derive(Debug, Clone, ValueEnum)]
pub enum CompletionShell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Elvish,
}

pub fn generate_completions(shell: CompletionShell) -> Result<()> {
    let mut command = Cli::command();
    let shell = match shell {
        CompletionShell::Bash => Shell::Bash,
        CompletionShell::Zsh => Shell::Zsh,
        CompletionShell::Fish => Shell::Fish,
        CompletionShell::PowerShell => Shell::PowerShell,
        CompletionShell::Elvish => Shell::Elvish,
    };

    generate(shell, &mut command, "opencli", &mut io::stdout());
    Ok(())
}
