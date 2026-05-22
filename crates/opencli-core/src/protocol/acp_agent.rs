use std::sync::Arc;

use agent_client_protocol::schema::{
    AgentCapabilities, AgentNotification, ContentBlock, ContentChunk, InitializeRequest,
    InitializeResponse, NewSessionRequest, NewSessionResponse, PromptRequest, PromptResponse,
    SessionNotification, SessionUpdate, StopReason, TextContent,
};
use agent_client_protocol::{
    Agent, Client, ConnectionTo, Dispatch, Stdio, on_receive_dispatch, on_receive_request,
};
use opencli_config::{ConfigOverrides, RuntimeConfig};
use uuid::Uuid;

/// Run opencli as an ACP agent via stdio transport.
pub async fn run_acp_agent(overrides: ConfigOverrides) -> anyhow::Result<()> {
    let config = Arc::new(opencli_config::load_config(overrides)?);

    let config_prompt = Arc::clone(&config);
    let config_session_new = Arc::clone(&config);
    let config_dispatch = Arc::clone(&config);

    Agent
        .builder()
        .name("opencli")
        .on_receive_request(
            async move |req: InitializeRequest, responder, _cx| {
                let resp = InitializeResponse::new(req.protocol_version)
                    .agent_capabilities(AgentCapabilities::new());
                responder.respond(resp)
            },
            on_receive_request!(),
        )
        .on_receive_request_from(
            Client,
            async move |_req: NewSessionRequest, responder, _cx| {
                match opencli_session::create_session(&config_session_new) {
                    Ok(session) => responder.respond(NewSessionResponse::new(session.id)),
                    Err(e) => {
                        responder.respond_with_error(agent_client_protocol::util::internal_error(
                            format!("failed to create session: {e:#}"),
                        ))
                    }
                }
            },
            on_receive_request!(),
        )
        .on_receive_request_from(
            Client,
            async move |req: PromptRequest, responder, cx: ConnectionTo<Client>| {
                let session_id = req.session_id.clone();
                let session_id_str: String = session_id.0.as_ref().to_string();
                let prompt_text = extract_text_prompt(&req.prompt);

                let mut config = (*config_prompt).clone();
                // 解析并提取元数据中的 model 字段，若存在且非空则覆盖现有模型配置
                if let Some(model) = req
                    .meta
                    .as_ref()
                    .and_then(|meta| meta.get("model"))
                    .and_then(|v| v.as_str())
                    .filter(|m| !m.is_empty())
                {
                    config.model = model.to_string();
                }
                let config = Arc::new(config);

                // Load existing session or start fresh
                let mut session = match opencli_session::load_session(&config, &session_id_str) {
                    Ok(s) => s,
                    Err(_) => match opencli_session::create_session(&config) {
                        Ok(s) => s,
                        Err(e) => {
                            return responder.respond_with_error(
                                agent_client_protocol::util::internal_error(format!(
                                    "failed to create session: {e:#}"
                                )),
                            );
                        }
                    },
                };

                opencli_session::append_message(
                    &mut session,
                    opencli_provider::ChatMessage::user(prompt_text.clone()),
                );

                let _ = cx.send_notification(AgentNotification::SessionNotification(
                    SessionNotification::new(
                        session_id.clone(),
                        SessionUpdate::AgentThoughtChunk(ContentChunk::new(ContentBlock::Text(
                            TextContent::new(format!("Processing: {prompt_text}")),
                        ))),
                    ),
                ));

                let config_for_blocking = config.clone();
                let messages = session.messages.clone();
                let result = tokio::task::spawn_blocking(move || {
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .expect("build local runtime");
                    rt.block_on(async move {
                        let mut messages = messages;
                        let agent_id = Uuid::new_v4().to_string();
                        crate::agent_turn::run_agent_turn(crate::agent_turn::AgentTurnRequest {
                            config: &config_for_blocking,
                            messages: &mut messages,
                            cancel_requested: None,
                            delegation_depth: 0,
                            agent_id: &agent_id,
                            parent_agent_id: None,
                            max_steps: config_for_blocking.agent_max_steps,
                            event_sender: None,
                        })
                        .await
                    })
                })
                .await
                .unwrap_or_else(|e| Err(anyhow::anyhow!("Task panicked: {e}")));

                match result {
                    Ok(output) => {
                        session
                            .messages
                            .push(opencli_provider::ChatMessage::assistant(
                                output.output.clone(),
                                Vec::new(),
                            ));
                        let _ = opencli_session::save_session(&config, &session);

                        let _ = cx.send_notification(AgentNotification::SessionNotification(
                            SessionNotification::new(
                                session_id,
                                SessionUpdate::AgentMessageChunk(ContentChunk::new(
                                    ContentBlock::Text(TextContent::new(output.output)),
                                )),
                            ),
                        ));
                        responder.respond(PromptResponse::new(StopReason::EndTurn))
                    }
                    Err(err) => {
                        let _ = cx.send_notification(AgentNotification::SessionNotification(
                            SessionNotification::new(
                                session_id,
                                SessionUpdate::AgentMessageChunk(ContentChunk::new(
                                    ContentBlock::Text(TextContent::new(format!("Error: {err:#}"))),
                                )),
                            ),
                        ));
                        responder.respond(PromptResponse::new(StopReason::Refusal))
                    }
                }
            },
            on_receive_request!(),
        )
        .on_receive_dispatch(
            async move |msg: Dispatch, cx: ConnectionTo<Client>| {
                let method = msg.method().to_string();

                match method.as_str() {
                    "session/list" => handle_session_list(msg, &config_dispatch),
                    "session/load" => handle_session_load(msg, &config_dispatch),
                    "session/delete" => handle_session_delete(msg, &config_dispatch),
                    _ => msg.respond_with_error(
                        agent_client_protocol::util::internal_error(format!(
                            "unhandled message: {method}"
                        )),
                        cx,
                    ),
                }
            },
            on_receive_dispatch!(),
        )
        .connect_to(Stdio::new())
        .await
        .map_err(|e| anyhow::anyhow!("ACP agent error: {e}"))?;

    Ok(())
}

fn handle_session_list(
    msg: Dispatch,
    config: &RuntimeConfig,
) -> Result<(), agent_client_protocol::Error> {
    if let Dispatch::Request(_req, responder) = msg {
        match opencli_session::list_sessions(config) {
            Ok(sessions) => {
                let list: Vec<serde_json::Value> = sessions
                    .iter()
                    .map(|s| {
                        serde_json::json!({
                            "id": s.id,
                            "title": s.title,
                            "model": s.model,
                            "createdAt": s.created_at.to_rfc3339(),
                            "updatedAt": s.updated_at.to_rfc3339(),
                            "messageCount": s.messages.len(),
                        })
                    })
                    .collect();
                responder.respond(serde_json::json!({ "sessions": list }))
            }
            Err(e) => responder.respond_with_error(agent_client_protocol::util::internal_error(
                format!("failed to list sessions: {e:#}"),
            )),
        }
    } else {
        Ok(())
    }
}

fn handle_session_load(
    msg: Dispatch,
    config: &RuntimeConfig,
) -> Result<(), agent_client_protocol::Error> {
    if let Dispatch::Request(req, responder) = msg {
        let id = req.params().get("id").and_then(|v| v.as_str());
        match id {
            Some(id) => match opencli_session::load_session(config, id) {
                Ok(session) => {
                    let messages: Vec<serde_json::Value> = session
                        .messages
                        .iter()
                        .map(|m| {
                            serde_json::json!({
                                "role": m.role,
                                "content": m.content,
                            })
                        })
                        .collect();
                    responder.respond(serde_json::json!({
                        "id": session.id,
                        "title": session.title,
                        "model": session.model,
                        "messages": messages,
                    }))
                }
                Err(e) => {
                    responder.respond_with_error(agent_client_protocol::util::internal_error(
                        format!("failed to load session: {e:#}"),
                    ))
                }
            },
            None => responder.respond_with_error(agent_client_protocol::util::internal_error(
                "missing 'id' parameter",
            )),
        }
    } else {
        Ok(())
    }
}

fn handle_session_delete(
    msg: Dispatch,
    config: &RuntimeConfig,
) -> Result<(), agent_client_protocol::Error> {
    if let Dispatch::Request(req, responder) = msg {
        let id = req.params().get("id").and_then(|v| v.as_str());
        match id {
            Some(id) => match opencli_session::delete_session(config, id) {
                Ok(()) => responder.respond(serde_json::json!({ "deleted": true })),
                Err(e) => {
                    responder.respond_with_error(agent_client_protocol::util::internal_error(
                        format!("failed to delete session: {e:#}"),
                    ))
                }
            },
            None => responder.respond_with_error(agent_client_protocol::util::internal_error(
                "missing 'id' parameter",
            )),
        }
    } else {
        Ok(())
    }
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
