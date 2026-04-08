use std::sync::{
    Arc,
    atomic::AtomicBool,
    mpsc::{self},
};

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
                state.scroll_from_bottom = 0;
                state.cancel_pending_task = false;
                state.pending = None;
            }
            Ok(Err(error)) => {
                if !format!("{error:#}").contains("agent execution canceled") {
                    state.lines.push(format!("Error: {error:#}"));
                }
                state.scroll_from_bottom = 0;
                state.cancel_pending_task = false;
                state.pending = None;
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                state
                    .lines
                    .push("Error: background response task disconnected".to_string());
                state.scroll_from_bottom = 0;
                state.cancel_pending_task = false;
                state.pending = None;
            }
            Err(mpsc::TryRecvError::Empty) => {}
        }
    }
}
