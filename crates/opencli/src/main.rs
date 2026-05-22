mod cli;
mod completions;

use clap::Parser;
use opencli_config::ConfigOverrides;
use opencli_core::commands;

use crate::cli::{AuditCommands, Cli, Commands, ConfigCommands, SessionCommands};

#[tokio::main]
async fn main() {
    opencli_core::telemetry::init_telemetry();

    if let Err(error) = run().await {
        eprintln!("Error: {error:#}");
        std::process::exit(opencli_core::errors::infer_exit_code(&error));
    }
}

async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if let Some(Commands::Completions { shell }) = &cli.command {
        return completions::generate_completions(shell.clone());
    }

    let command = match cli.command {
        Some(Commands::Chat) => Some(commands::Command::Chat),
        #[cfg(feature = "tui")]
        Some(Commands::Tui) => Some(commands::Command::Tui),
        Some(Commands::A2a(args)) => Some(commands::Command::A2a(commands::A2aArgs {
            role: args.role,
            context: args.context,
            max_steps: args.max_steps,
            task: args.task,
        })),
        Some(Commands::A2aBatch(args)) => {
            Some(commands::Command::A2aBatch(commands::A2aBatchArgs {
                file: args.file,
                concurrency: args.concurrency,
            }))
        }
        Some(Commands::Acp) => Some(commands::Command::Acp),
        Some(Commands::Models) => Some(commands::Command::Models),
        Some(Commands::Completions { .. }) => unreachable!(),
        Some(Commands::Run(args)) => Some(commands::Command::Run(commands::RunArgs {
            files: args.files,
            dirs: args.dirs,
            prompt: args.prompt,
        })),
        Some(Commands::Config { command }) => Some(commands::Command::Config(match command {
            ConfigCommands::Init => commands::ConfigCommand::Init,
            ConfigCommands::Show => commands::ConfigCommand::Show,
            ConfigCommands::Doctor => commands::ConfigCommand::Doctor,
        })),
        Some(Commands::Session { command }) => Some(commands::Command::Session(match command {
            SessionCommands::List => commands::SessionCommand::List,
            SessionCommands::Resume { id } => commands::SessionCommand::Resume { id },
            SessionCommands::Delete { id } => commands::SessionCommand::Delete { id },
            SessionCommands::Rename { id, title } => commands::SessionCommand::Rename { id, title },
        })),
        Some(Commands::Audit { command }) => Some(commands::Command::Audit(match command {
            AuditCommands::List(args) => commands::AuditCommand::List(commands::AuditListArgs {
                limit: args.limit,
                tool: args.tool,
                event_type: args.event_type,
            }),
            AuditCommands::Tail(args) => commands::AuditCommand::Tail(commands::AuditTailArgs {
                lines: args.lines,
                follow: args.follow,
                tool: args.tool,
                event_type: args.event_type,
            }),
            AuditCommands::Stats => commands::AuditCommand::Stats,
            AuditCommands::Graph => commands::AuditCommand::Graph,
            AuditCommands::Clear => commands::AuditCommand::Clear,
            AuditCommands::Export(args) => {
                commands::AuditCommand::Export(commands::AuditExportArgs {
                    output: args.output,
                    tool: args.tool,
                    event_type: args.event_type,
                })
            }
        })),
        None => None,
    };

    let overrides = ConfigOverrides {
        model: cli.model,
        base_url: cli.base_url,
        api_key: cli.api_key,
    };

    opencli_core::run_command(&cli.prompt, command, overrides).await
}
