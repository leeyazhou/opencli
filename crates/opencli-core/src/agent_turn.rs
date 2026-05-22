use std::sync::{Arc, atomic::AtomicBool, mpsc};

use anyhow::Result;
use opencli_audit::FileAuditLogger;
use opencli_config::RuntimeConfig;
use opencli_output::BufferRenderer;
use opencli_provider::{ChatMessage, ProviderFactory};

use crate::{
    agent::run_agent_loop,
    tools::{ToolExecutionContext, ToolRegistry},
    tui::state::TurnEvent,
};

pub(crate) struct AgentTurnRequest<'a> {
    pub config: &'a RuntimeConfig,
    pub messages: &'a mut Vec<ChatMessage>,
    pub cancel_requested: Option<&'a Arc<AtomicBool>>,
    pub delegation_depth: u32,
    pub agent_id: &'a str,
    pub parent_agent_id: Option<&'a str>,
    pub max_steps: usize,
    pub event_sender: Option<&'a mpsc::Sender<TurnEvent>>,
}

pub(crate) struct AgentTurnOutput {
    pub output: String,
}

pub(crate) async fn run_agent_turn(request: AgentTurnRequest<'_>) -> Result<AgentTurnOutput> {
    let provider = ProviderFactory::new().create(request.config)?;
    let audit = FileAuditLogger::new();
    let tool_registry = ToolRegistry::new();
    let mut renderer = BufferRenderer::new();
    let tool_context = ToolExecutionContext {
        config: request.config,
        audit: &audit,
        cancel_requested: request.cancel_requested,
        delegation_depth: request.delegation_depth,
        agent_id: Some(request.agent_id),
        parent_agent_id: request.parent_agent_id,
    };
    let output = run_agent_loop(
        provider.as_ref(),
        &tool_registry,
        &mut renderer,
        &tool_context,
        request.cancel_requested,
        request.max_steps,
        request.messages,
        request.event_sender,
    )
    .await?;

    Ok(AgentTurnOutput { output })
}

#[cfg(test)]
mod tests {
    use mockito::Server;

    use super::{AgentTurnRequest, run_agent_turn};
    use opencli_config::RuntimeConfig;
    use opencli_provider::ChatMessage;

    #[tokio::test]
    async fn run_agent_turn_returns_output_and_messages() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("POST", "/chat/completions")
            .match_header("authorization", "Bearer test-key")
            .with_status(200)
            .with_body(r#"{"choices":[{"message":{"content":"helper output","tool_calls":[]}}]}"#)
            .create_async()
            .await;

        let config = RuntimeConfig {
            base_url: server.url(),
            api_key: "test-key".into(),
            ..RuntimeConfig::default()
        };

        let mut messages = vec![ChatMessage::user("hello")];
        let result = run_agent_turn(AgentTurnRequest {
            config: &config,
            messages: &mut messages,
            cancel_requested: None,
            delegation_depth: 0,
            agent_id: "agent-1",
            parent_agent_id: None,
            max_steps: 1,
            event_sender: None,
        })
        .await
        .expect("agent turn should succeed");

        assert_eq!(result.output, "helper output");
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[1].role, "assistant");
        assert_eq!(messages[1].content, "helper output");
    }
}
