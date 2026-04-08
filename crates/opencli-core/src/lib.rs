pub mod a2a;
pub mod agent;
pub mod app;
pub mod commands;
pub mod context;
pub mod errors;
pub mod runtime;
pub mod services;
pub mod telemetry;
pub mod tools;
#[cfg(feature = "tui")]
pub mod tui;

use anyhow::Result;
use opencli_config::ConfigOverrides;

use app::App;
use commands::{AuditCommand, Command, SessionCommand};
use runtime::Runtime;

pub async fn run_command(
    prompt: &[String],
    command: Option<Command>,
    overrides: ConfigOverrides,
) -> Result<()> {
    let runtime = Runtime::from_overrides(overrides)?;
    let mut app = App::from_runtime(runtime);

    match command {
        Some(Command::Chat) => app.run_chat().await,
        #[cfg(feature = "tui")]
        Some(Command::Tui) => tui::run_chat_tui(&mut app).await,
        Some(Command::A2a(args)) => app.run_a2a(&args).await,
        Some(Command::A2aBatch(args)) => app.run_a2a_batch(&args).await,
        Some(Command::Models) => app.run_models().await,
        Some(Command::Run(args)) => {
            app.run_with_context(&args.prompt, &args.files, &args.dirs)
                .await
        }
        Some(Command::Config(command)) => app.run_config_command(command).await,
        Some(Command::Session(SessionCommand::List)) => app.list_sessions(),
        Some(Command::Session(SessionCommand::Resume { id })) => app.run_resume(&id).await,
        Some(Command::Session(SessionCommand::Delete { id })) => app.delete_session(&id),
        Some(Command::Session(SessionCommand::Rename { id, title })) => {
            app.rename_session(&id, &title)
        }
        Some(Command::Audit(AuditCommand::List(args))) => app.run_audit_list(&args),
        Some(Command::Audit(AuditCommand::Tail(args))) => app.run_audit_tail(&args),
        Some(Command::Audit(AuditCommand::Stats)) => app.run_audit_stats(),
        Some(Command::Audit(AuditCommand::Graph)) => app.run_audit_graph(),
        Some(Command::Audit(AuditCommand::Clear)) => app.run_audit_clear(),
        Some(Command::Audit(AuditCommand::Export(args))) => app.run_audit_export(&args),
        None => app.run_default_prompt(prompt).await,
    }
}
