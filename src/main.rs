mod agent;
mod a2a;
mod app;
mod approval;
mod audit;
mod cli;
mod completions;
mod config;
mod context;
mod errors;
mod markdown;
mod message;
mod output;
mod provider;
mod provider_factory;
mod runtime;
mod safety;
mod services;
mod session;
mod telemetry;
mod tools;
#[cfg(feature = "tui")]
mod tui;

use anyhow::Result;
use clap::Parser;

use app::App;
use cli::{AuditCommands, Cli, Commands, SessionCommands};
use config::ConfigOverrides;
use runtime::Runtime;

#[tokio::main]
async fn main() {
    telemetry::init_telemetry();

    if let Err(error) = run().await {
        eprintln!("Error: {error:#}");
        std::process::exit(errors::infer_exit_code(&error));
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();
    let overrides = ConfigOverrides {
        model: cli.model.clone(),
        base_url: cli.base_url.clone(),
        api_key: cli.api_key.clone(),
    };
    let runtime = Runtime::from_overrides(overrides)?;
    let mut app = App::from_runtime(runtime);

    match cli.command {
        Some(Commands::Chat) => app.run_chat().await,
        #[cfg(feature = "tui")]
        Some(Commands::Tui) => tui::run_chat_tui(&mut app).await,
        Some(Commands::A2a(args)) => app.run_a2a(&args).await,
        Some(Commands::A2aBatch(args)) => app.run_a2a_batch(&args).await,
        Some(Commands::Models) => app.run_models().await,
        Some(Commands::Completions { shell }) => completions::generate_completions(shell),
        Some(Commands::Run(args)) => {
            app.run_with_context(&args.prompt, &args.files, &args.dirs)
                .await
        }
        Some(Commands::Config { command }) => app.run_config_command(command).await,
        Some(Commands::Session {
            command: SessionCommands::List,
        }) => app.list_sessions(),
        Some(Commands::Session {
            command: SessionCommands::Resume { id },
        }) => app.run_resume(&id).await,
        Some(Commands::Session {
            command: SessionCommands::Delete { id },
        }) => app.delete_session(&id),
        Some(Commands::Session {
            command: SessionCommands::Rename { id, title },
        }) => app.rename_session(&id, &title),
        Some(Commands::Audit {
            command: AuditCommands::List(args),
        }) => app.run_audit_list(&args),
        Some(Commands::Audit {
            command: AuditCommands::Tail(args),
        }) => app.run_audit_tail(&args),
        Some(Commands::Audit {
            command: AuditCommands::Stats,
        }) => app.run_audit_stats(),
        Some(Commands::Audit {
            command: AuditCommands::Clear,
        }) => app.run_audit_clear(),
        Some(Commands::Audit {
            command: AuditCommands::Export(args),
        }) => app.run_audit_export(&args),
        None => app.run_default_prompt(&cli).await,
    }
}
