use clap::{Args, Parser, Subcommand};

use crate::completions::CompletionShell;

#[derive(Debug, Parser)]
#[command(name = "ai-cli", about = "Code-focused AI CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(global = true, short = 'm', long)]
    pub model: Option<String>,

    #[arg(global = true, long)]
    pub base_url: Option<String>,

    #[arg(global = true, long)]
    pub api_key: Option<String>,

    #[arg()]
    pub prompt: Vec<String>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Chat,
    #[cfg(feature = "tui")]
    Tui,
    A2a(A2aArgs),
    A2aBatch(A2aBatchArgs),
    Models,
    Completions {
        shell: CompletionShell,
    },
    Run(RunArgs),
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    Session {
        #[command(subcommand)]
        command: SessionCommands,
    },
    Audit {
        #[command(subcommand)]
        command: AuditCommands,
    },
}

#[derive(Debug, Args)]
pub struct RunArgs {
    #[arg(short = 'f', long = "file")]
    pub files: Vec<String>,

    #[arg(short = 'd', long = "dir")]
    pub dirs: Vec<String>,

    #[arg()]
    pub prompt: Vec<String>,
}

#[derive(Debug, Args)]
pub struct A2aArgs {
    #[arg(long)]
    pub role: String,

    #[arg(long)]
    pub context: Option<String>,

    #[arg(long)]
    pub max_steps: Option<usize>,

    #[arg()]
    pub task: Vec<String>,
}

#[derive(Debug, Args)]
pub struct A2aBatchArgs {
    #[arg(long)]
    pub file: String,

    #[arg(long)]
    pub concurrency: Option<usize>,
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommands {
    Init,
    Show,
    Doctor,
}

#[derive(Debug, Subcommand)]
pub enum SessionCommands {
    List,
    Resume { id: String },
    Delete { id: String },
    Rename { id: String, title: String },
}

#[derive(Debug, Subcommand)]
pub enum AuditCommands {
    List(AuditListArgs),
    Tail(AuditTailArgs),
    Stats,
    Graph,
    Clear,
    Export(AuditExportArgs),
}

#[derive(Debug, Args)]
pub struct AuditListArgs {
    #[arg(long, default_value_t = 50)]
    pub limit: usize,

    #[arg(long)]
    pub tool: Option<String>,

    #[arg(long = "event")]
    pub event_type: Option<String>,
}

#[derive(Debug, Args)]
pub struct AuditTailArgs {
    #[arg(long, default_value_t = 20)]
    pub lines: usize,

    #[arg(long)]
    pub follow: bool,

    #[arg(long)]
    pub tool: Option<String>,

    #[arg(long = "event")]
    pub event_type: Option<String>,
}

#[derive(Debug, Args)]
pub struct AuditExportArgs {
    #[arg(long)]
    pub output: String,

    #[arg(long)]
    pub tool: Option<String>,

    #[arg(long = "event")]
    pub event_type: Option<String>,
}
