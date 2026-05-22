use std::sync::{
    Arc,
    atomic::AtomicBool,
    mpsc::{self},
};
use std::time::Instant;

use opencli_config::RuntimeConfig;
use opencli_session::StoredSession;

use crate::app::App;

use super::state::{ChatMessage, PendingTask, TaskMessage, TurnEvent};

pub fn spawn_response_task(
    config: RuntimeConfig,
    session_snapshot: StoredSession,
    prompt: String,
) -> PendingTask {
    let (sender, receiver) = mpsc::channel();
    let cancel_requested = Arc::new(AtomicBool::new(false));
    let cancel_requested_for_thread = Arc::clone(&cancel_requested);
    let preview = prompt_preview(&prompt);

    let (event_tx, event_rx) = mpsc::channel();

    std::thread::spawn(move || {
        let runtime = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(runtime) => runtime,
            Err(error) => {
                let _ = sender.send(TaskMessage::Finished(Err(anyhow::anyhow!(error))));
                return;
            }
        };

        let forward_sender = sender.clone();
        std::thread::spawn(move || {
            for event in event_rx {
                let _ = forward_sender.send(TaskMessage::Event(event));
            }
        });

        let result = runtime.block_on(App::run_detached_session_turn(
            config,
            session_snapshot,
            prompt,
            cancel_requested_for_thread,
            Some(event_tx),
        ));
        let _ = sender.send(TaskMessage::Finished(Ok(result)));
    });

    PendingTask {
        receiver,
        cancel_requested,
        started_at: Instant::now(),
        prompt_preview: preview,
    }
}

pub fn poll_background(state: &mut super::state::TuiState) {
    if let Some(pending) = &state.pending {
        loop {
            match pending.receiver.try_recv() {
                Ok(TaskMessage::Event(event)) => {
                    match event {
                        TurnEvent::Thought(text) => {
                            state.last_event = format!("thought: {}", prompt_preview(&text));
                            state.messages.push(ChatMessage::Thought(text));
                        }
                        TurnEvent::ToolCall { tool_name, status } => {
                            state.last_event = format!("tool: {tool_name}");
                            // Update existing tool message or add new one
                            if let Some(ChatMessage::ToolCall {
                                tool_name: existing_name,
                                status: existing_status,
                            }) = state.messages.iter_mut().rev().find(|m| {
                                matches!(m, ChatMessage::ToolCall { tool_name: n, .. } if n == &tool_name)
                            }) {
                                *existing_name = tool_name;
                                *existing_status = status;
                            } else {
                                state.messages.push(ChatMessage::ToolCall {
                                    tool_name,
                                    status,
                                });
                            }
                        }
                    }
                }
                Ok(TaskMessage::Finished(Ok(result))) => {
                    state.session = result.session;
                    match result.output {
                        Ok(output) => {
                            if !output.is_empty() {
                                state.messages.push(ChatMessage::Assistant(output));
                            }
                            state.completed_turns += 1;
                            state.last_event = format!("completed {}", pending.prompt_preview);
                        }
                        Err(error) => {
                            if !format!("{error:#}").contains("agent execution canceled") {
                                state
                                    .messages
                                    .push(ChatMessage::Error(format!("{error:#}")));
                                state.last_event = format!("failed {}", pending.prompt_preview);
                            } else {
                                state.last_event = format!("canceled {}", pending.prompt_preview);
                            }
                        }
                    }
                    state.scroll_from_bottom = 0;
                    state.cancel_pending_task = false;
                    state.pending = None;
                    break;
                }
                Ok(TaskMessage::Finished(Err(error))) => {
                    if !format!("{error:#}").contains("agent execution canceled") {
                        state
                            .messages
                            .push(ChatMessage::Error(format!("{error:#}")));
                        state.last_event = format!("failed {}", pending.prompt_preview);
                    } else {
                        state.last_event = format!("canceled {}", pending.prompt_preview);
                    }
                    state.scroll_from_bottom = 0;
                    state.cancel_pending_task = false;
                    state.pending = None;
                    break;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    state.messages.push(ChatMessage::System(
                        "Error: background response task disconnected".to_string(),
                    ));
                    state.last_event = "background task disconnected".to_string();
                    state.scroll_from_bottom = 0;
                    state.cancel_pending_task = false;
                    state.pending = None;
                    break;
                }
                Err(mpsc::TryRecvError::Empty) => break,
            }
        }
    }
}

fn prompt_preview(prompt: &str) -> String {
    let single_line = prompt.replace('\n', " ");
    let mut preview = single_line.chars().take(24).collect::<String>();
    if single_line.chars().count() > 24 {
        preview.push_str("...");
    }
    preview
}
