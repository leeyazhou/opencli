mod common;

use std::{io::{self, Write}, path::PathBuf, time::Duration};
use std::sync::{Arc, atomic::AtomicBool};

use anyhow::{Result, bail};
use tracing::{Instrument, info, info_span};
use uuid::Uuid;

use crate::{
    a2a::{SubAgentRequest, run_subagent, run_subagents_concurrent},
    agent::{run_agent_loop, run_streaming_text},
    audit::{clear_records, compute_agent_graph, compute_stats, export_records, read_all_records, read_records, render_agent_graph},
    cli::{A2aArgs, A2aBatchArgs, AuditExportArgs, AuditListArgs, AuditTailArgs, Cli, ConfigCommands},
    config::{self, ensure_default_config},
    context,
    message::ChatMessage,
    output::Renderer,
    runtime::Runtime,
    services::config_doctor::run_config_doctor,
    session::{StoredSession, append_message, create_session, delete_session, list_sessions, load_session, rename_session, save_session},
    tools::ToolExecutionContext,
};

#[cfg(feature = "tui")]
use crate::{approval::PolicyApproval, output::BufferRenderer, provider_factory::ProviderFactory, tools::ToolRegistry};

use self::common::{join_prompt, read_stdin_if_piped};

pub struct App {
    runtime: Runtime,
}

impl App {
    pub fn from_runtime(runtime: Runtime) -> Self {
        Self { runtime }
    }

    #[cfg(feature = "tui")]
    pub fn runtime_config(&self) -> &config::RuntimeConfig {
        &self.runtime.config
    }

    pub async fn run_prompt(&mut self, prompt: String) -> Result<()> {
        if prompt.trim().is_empty() {
            bail!("missing prompt");
        }

        let request_id = Uuid::new_v4().to_string();
        let span = info_span!("run_prompt", request_id = %request_id);
        info!(request_id = %request_id, "starting prompt execution");

        let mut messages = vec![ChatMessage::user(prompt)];
        let provider = self.runtime.provider()?;

        if prompt_only(&messages) {
            run_streaming_text(provider.as_ref(), &mut self.runtime.renderer, &messages)
                .instrument(span)
                .await?;
        } else {
            let config = &self.runtime.config;
            let approval = self.runtime.approval.as_ref();
            let audit = self.runtime.audit.as_ref();
            let tool_registry = &self.runtime.tool_registry;
            let renderer = &mut self.runtime.renderer;
            let tool_context = ToolExecutionContext { config, approval, audit, cancel_requested: None, delegation_depth: 0, agent_id: Some(request_id.as_str()), parent_agent_id: None };
            run_agent_loop(provider.as_ref(), tool_registry, renderer, &tool_context, None, self.runtime.config.agent_max_steps, &mut messages)
                .instrument(span)
                .await?;
        }

        Ok(())
    }

    pub async fn run_with_context(&mut self, prompt_parts: &[String], files: &[String], dirs: &[String]) -> Result<()> {
        let prompt = join_prompt(prompt_parts)?;
        let context = context::collect_context(&PathBuf::from(&self.runtime.config.workspace_root), files, dirs)?;
        let final_prompt = if context.is_empty() { prompt } else { format!("{prompt}\n\nContext:\n{context}") };
        self.run_prompt(final_prompt).await
    }

    pub async fn run_default_prompt(&mut self, cli: &Cli) -> Result<()> {
        let prompt = join_prompt(&cli.prompt)?;
        let stdin_text = read_stdin_if_piped()?;
        let final_prompt = [prompt, stdin_text].into_iter().filter(|part| !part.is_empty()).collect::<Vec<_>>().join("\n\n");
        self.run_prompt(final_prompt).await
    }

    pub async fn run_chat(&mut self) -> Result<()> {
        let mut session = create_session(&self.runtime.config)?;
        self.runtime.renderer.render_line(&format!("Interactive chat started ({}). Type /exit to quit.", session.id))?;
        self.run_interactive_session(&mut session).await
    }

    pub async fn run_resume(&mut self, id: &str) -> Result<()> {
        let mut session = load_session(&self.runtime.config, id)?;
        self.runtime.renderer.render_line(&format!("Resumed session {}. Type /exit to quit.", session.id))?;
        self.run_interactive_session(&mut session).await
    }

    pub async fn run_models(&mut self) -> Result<()> {
        let provider = self.runtime.provider()?;
        info!(provider = provider.name(), capabilities = ?provider.capabilities(), "listing models");
        for model in provider.list_models().await? {
            self.runtime.renderer.render_line(&model)?;
        }
        Ok(())
    }

    pub async fn run_a2a(&mut self, args: &A2aArgs) -> Result<()> {
        let task = join_prompt(&args.task)?;
        let output = run_subagent(
            &self.runtime.config,
            SubAgentRequest {
                role: args.role.clone(),
                task,
                context: args.context.clone(),
                allowed_tool_kinds: None,
                allowed_tools: None,
                max_steps: args.max_steps,
            },
            None,
            0,
            None,
        )
        .await?;
        self.runtime.renderer.render_line(&serde_json::to_string_pretty(&output)?)?;
        Ok(())
    }

    pub async fn run_a2a_batch(&mut self, args: &A2aBatchArgs) -> Result<()> {
        let content = std::fs::read_to_string(&args.file)?;
        let requests = serde_json::from_str::<Vec<SubAgentRequest>>(&content)?;
        let results = run_subagents_concurrent(
            &self.runtime.config,
            requests,
            None,
            0,
            args.concurrency,
            None,
        )
        .await?;
        self.runtime.renderer.render_line(&serde_json::to_string_pretty(&results)?)?;
        Ok(())
    }

    pub async fn run_config_command(&mut self, command: ConfigCommands) -> Result<()> {
        match command {
            ConfigCommands::Init => {
                let path = ensure_default_config()?;
                self.runtime.renderer.render_line(&format!("Created config at {}", path.display()))?;
                Ok(())
            }
            ConfigCommands::Show => {
                self.runtime.renderer.render_line(&serde_json::to_string_pretty(&config::redacted_config(&self.runtime.config))?)?;
                Ok(())
            }
            ConfigCommands::Doctor => {
                let issues = run_config_doctor(&self.runtime.config).await?;
                if issues.is_empty() {
                    self.runtime.renderer.render_line("Config doctor: OK")?;
                } else {
                    self.runtime.renderer.render_line("Config doctor found issues:")?;
                    for issue in issues {
                        self.runtime.renderer.render_line(&format!("- {issue}"))?;
                    }
                }
                Ok(())
            }
        }
    }

    pub fn list_sessions(&mut self) -> Result<()> {
        for session in list_sessions(&self.runtime.config)? {
            self.runtime.renderer.render_line(&format!("{}  {}  {}", session.id, session.updated_at.to_rfc3339(), session.title))?;
        }
        Ok(())
    }

    pub fn delete_session(&mut self, id: &str) -> Result<()> {
        delete_session(&self.runtime.config, id)?;
        self.runtime.renderer.render_line(&format!("Deleted session {id}"))?;
        Ok(())
    }

    pub fn rename_session(&mut self, id: &str, title: &str) -> Result<()> {
        rename_session(&self.runtime.config, id, title)?;
        self.runtime.renderer.render_line(&format!("Renamed session {id}"))?;
        Ok(())
    }

    pub fn run_audit_list(&mut self, args: &AuditListArgs) -> Result<()> {
        for record in read_records(&self.runtime.config, args.limit, args.tool.as_deref(), args.event_type.as_deref())? {
            self.runtime.renderer.render_line(&serde_json::to_string(&record)?)?;
        }
        Ok(())
    }

    pub fn run_audit_tail(&mut self, args: &AuditTailArgs) -> Result<()> {
        let mut last_emitted = 0usize;
        loop {
            let records = read_all_records(&self.runtime.config, args.tool.as_deref(), args.event_type.as_deref())?;
            let start = if last_emitted == 0 { records.len().saturating_sub(args.lines) } else { last_emitted.min(records.len()) };
            for record in records.iter().skip(start) {
                self.runtime.renderer.render_line(&serde_json::to_string(record)?)?;
            }
            if !args.follow { break; }
            last_emitted = records.len();
            std::thread::sleep(Duration::from_millis(750));
        }
        Ok(())
    }

    pub fn run_audit_clear(&mut self) -> Result<()> {
        clear_records(&self.runtime.config)?;
        self.runtime.renderer.render_line("Cleared audit log")?;
        Ok(())
    }

    pub fn run_audit_stats(&mut self) -> Result<()> {
        let stats = compute_stats(&self.runtime.config)?;
        self.runtime.renderer.render_line(&serde_json::to_string_pretty(&stats)?)?;
        Ok(())
    }

    pub fn run_audit_graph(&mut self) -> Result<()> {
        let graph = compute_agent_graph(&self.runtime.config)?;
        self.runtime.renderer.render_line(&render_agent_graph(&graph))?;
        Ok(())
    }

    pub fn run_audit_export(&mut self, args: &AuditExportArgs) -> Result<()> {
        let count = export_records(&self.runtime.config, &args.output, args.tool.as_deref(), args.event_type.as_deref())?;
        self.runtime.renderer.render_line(&format!("Exported {count} audit records to {}", args.output))?;
        Ok(())
    }

    #[cfg(feature = "tui")]
    pub fn create_runtime_session(&self) -> Result<StoredSession> {
        create_session(&self.runtime.config)
    }

    #[cfg(feature = "tui")]
    pub async fn run_detached_session_turn(config: config::RuntimeConfig, mut session: StoredSession, prompt: String, cancel_requested: Arc<AtomicBool>) -> Result<(StoredSession, String)> {
        let request_id = Uuid::new_v4().to_string();
        let span = info_span!("detached_session_turn", request_id = %request_id);
        let provider = ProviderFactory::new().create(&config)?;
        let approval = PolicyApproval::new();
        let audit = crate::audit::FileAuditLogger::new();
        let tool_registry = ToolRegistry::new();
        let mut renderer = BufferRenderer::new();
        append_message(&mut session, ChatMessage::user(prompt));
        let tool_context = ToolExecutionContext { config: &config, approval: &approval, audit: &audit, cancel_requested: Some(&cancel_requested), delegation_depth: 0, agent_id: Some(request_id.as_str()), parent_agent_id: None };
        let output = run_agent_loop(provider.as_ref(), &tool_registry, &mut renderer, &tool_context, Some(&cancel_requested), config.agent_max_steps, &mut session.messages)
            .instrument(span)
            .await?;
        save_session(&config, &session)?;
        Ok((session, output))
    }

    async fn run_interactive_session(&mut self, session: &mut StoredSession) -> Result<()> {
        let provider = self.runtime.provider()?;
        loop {
            print!("> ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim().to_string();
            if input.is_empty() { continue; }
            if input == "/exit" || input == "/quit" { break; }

            append_message(session, ChatMessage::user(input));
            let config = &self.runtime.config;
            let approval = self.runtime.approval.as_ref();
            let audit = self.runtime.audit.as_ref();
            let tool_registry = &self.runtime.tool_registry;
            let renderer = &mut self.runtime.renderer;
            let turn_agent_id = Uuid::new_v4().to_string();
            let tool_context = ToolExecutionContext { config, approval, audit, cancel_requested: None, delegation_depth: 0, agent_id: Some(turn_agent_id.as_str()), parent_agent_id: None };
            run_agent_loop(provider.as_ref(), tool_registry, renderer, &tool_context, None, self.runtime.config.agent_max_steps, &mut session.messages).await?;
            save_session(&self.runtime.config, session)?;
        }
        Ok(())
    }
}

fn prompt_only(messages: &[ChatMessage]) -> bool {
    messages.len() == 1 && messages.first().is_some_and(|message| message.role == "user")
}
