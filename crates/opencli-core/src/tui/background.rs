use std::sync::{
    Arc,
    atomic::AtomicBool,
    mpsc::{self},
};
use std::time::Instant;

use opencli_config::RuntimeConfig;
use opencli_session::StoredSession;

use crate::app::App;

pub fn spawn_response_task(
    config: RuntimeConfig,
    session_snapshot: StoredSession,
    prompt: String,
) -> super::state::PendingTask {
    let (sender, receiver) = mpsc::channel();
    let cancel_requested = Arc::new(AtomicBool::new(false));
    let cancel_requested_for_thread = Arc::clone(&cancel_requested);
    let preview = prompt_preview(&prompt);

    std::thread::spawn(move || {
        let runtime = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(runtime) => runtime,
            Err(error) => {
                let _ = sender.send(Err(anyhow::anyhow!(error)));
                return;
            }
        };

        let result = runtime.block_on(App::run_detached_session_turn(
            config,
            session_snapshot,
            prompt,
            cancel_requested_for_thread,
        ));
        let _ = sender.send(result);
    });

    super::state::PendingTask {
        receiver,
        cancel_requested,
        started_at: Instant::now(),
        prompt_preview: preview,
    }
}

pub fn poll_background(state: &mut super::state::TuiState) {
    if let Some(pending) = &state.pending {
        match pending.receiver.try_recv() {
            Ok(Ok((updated_session, output))) => {
                state.session = updated_session;
                if !output.is_empty() {
                    state.lines.push(output);
                }
                state.completed_turns += 1;
                state.last_event = format!("completed {}", pending.prompt_preview);
                state.scroll_from_bottom = 0;
                state.cancel_pending_task = false;
                state.pending = None;
            }
            Ok(Err(error)) => {
                if !format!("{error:#}").contains("agent execution canceled") {
                    state.lines.push(format!("Error: {error:#}"));
                    state.last_event = format!("failed {}", pending.prompt_preview);
                } else {
                    state.last_event = format!("canceled {}", pending.prompt_preview);
                }
                state.scroll_from_bottom = 0;
                state.cancel_pending_task = false;
                state.pending = None;
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                state
                    .lines
                    .push("Error: background response task disconnected".to_string());
                state.last_event = "background task disconnected".to_string();
                state.scroll_from_bottom = 0;
                state.cancel_pending_task = false;
                state.pending = None;
            }
            Err(mpsc::TryRecvError::Empty) => {}
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
