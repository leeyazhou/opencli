use std::sync::Arc;

use agent_client_protocol::schema::{
    AgentCapabilities, AgentNotification, ContentBlock, ContentChunk, InitializeRequest,
    InitializeResponse, NewSessionRequest, NewSessionResponse, PromptRequest, PromptResponse,
    SessionNotification, SessionUpdate, StopReason, TextContent,
};
use agent_client_protocol::{
    Agent, Client, ConnectionTo, Dispatch, Stdio, on_receive_dispatch, on_receive_request,
};
use opencli_config::ConfigOverrides;
use uuid::Uuid;

/// Run opencli as an ACP agent via stdio transport.
///
/// This starts an ACP-compliant agent server that listens on stdin/stdout
/// for JSON-RPC messages and processes prompts using the existing opencli
/// agent infrastructure.
pub async fn run_acp_agent(overrides: ConfigOverrides) -> anyhow::Result<()> {
    // Resolve config from overrides.
    let config = Arc::new(opencli_config::load_config(overrides)?);

    let config_prompt = Arc::clone(&config);

    Agent
        .builder()
        .name("opencli")
        // Handler for initialize
        .on_receive_request(
            async move |req: InitializeRequest, responder, _cx| {
                let resp = InitializeResponse::new(req.protocol_version)
                    .agent_capabilities(AgentCapabilities::new());
                responder.respond(resp)
            },
            on_receive_request!(),
        )
        // Handler for new session
        .on_receive_request_from(
            Client,
            async move |_req: NewSessionRequest, responder, _cx| {
                let session_id = Uuid::new_v4().to_string();
                responder.respond(NewSessionResponse::new(session_id))
            },
            on_receive_request!(),
        )
        // Handler for prompt — bridges !Send agent logic via spawn_blocking
        .on_receive_request_from(
            Client,
            async move |req: PromptRequest, responder, cx: ConnectionTo<Client>| {
                let session_id = req.session_id.clone();
                let prompt_text = extract_text_prompt(&req.prompt);

                // Notify the client we're starting
                let _ = cx.send_notification(AgentNotification::SessionNotification(
                    SessionNotification::new(
                        session_id.clone(),
                        SessionUpdate::AgentThoughtChunk(ContentChunk::new(
                            ContentBlock::Text(TextContent::new(format!("Processing: {prompt_text}"))),
                        )),
                    ),
                ));

                let config = Arc::clone(&config_prompt);

                // Run the agent turn on a dedicated thread since the existing
                // tool/provider infrastructure is not Send.
                let result = tokio::task::spawn_blocking(move || {
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .expect("build local runtime");
                    rt.block_on(async move {
                        let mut messages =
                            vec![opencli_provider::ChatMessage::user(prompt_text)];
                        let agent_id = Uuid::new_v4().to_string();
                        crate::agent_turn::run_agent_turn(
                            crate::agent_turn::AgentTurnRequest {
                                config: &config,
                                messages: &mut messages,
                                cancel_requested: None,
                                delegation_depth: 0,
                                agent_id: &agent_id,
                                parent_agent_id: None,
                                max_steps: config.agent_max_steps,
                            },
                        )
                        .await
                    })
                })
                .await
                .unwrap_or_else(|e| Err(anyhow::anyhow!("Task panicked: {e}")));

                match result {
                    Ok(output) => {
                        let _ =
                            cx.send_notification(AgentNotification::SessionNotification(
                                SessionNotification::new(
                                    session_id.clone(),
                                    SessionUpdate::AgentMessageChunk(ContentChunk::new(
                                        ContentBlock::Text(TextContent::new(output.output)),
                                    )),
                                ),
                            ));
                        responder.respond(PromptResponse::new(StopReason::EndTurn))
                    }
                    Err(err) => {
                        let _ =
                            cx.send_notification(AgentNotification::SessionNotification(
                                SessionNotification::new(
                                    session_id,
                                    SessionUpdate::AgentMessageChunk(ContentChunk::new(
                                        ContentBlock::Text(TextContent::new(format!(
                                            "Error: {err:#}"
                                        ))),
                                    )),
                                ),
                            ));
                        responder.respond(PromptResponse::new(StopReason::Refusal))
                    }
                }
            },
            on_receive_request!(),
        )
        // Fallback for unhandled messages
        .on_receive_dispatch(
            async move |msg: Dispatch, cx: ConnectionTo<Client>| {
                let method = format!("{:?}", msg.method());
                msg.respond_with_error(
                    agent_client_protocol::util::internal_error(format!(
                        "unhandled message: {method}"
                    )),
                    cx,
                )
            },
            on_receive_dispatch!(),
        )
        .connect_to(Stdio::new())
        .await
        .map_err(|e| anyhow::anyhow!("ACP agent error: {e}"))?;

    Ok(())
}

/// Extract text content from a list of ContentBlocks.
fn extract_text_prompt(blocks: &[ContentBlock]) -> String {
    blocks
        .iter()
        .filter_map(|block| match block {
            ContentBlock::Text(text) => Some(text.text.clone()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}
