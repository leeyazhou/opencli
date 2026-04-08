use std::{
    collections::VecDeque,
    sync::{Arc, atomic::AtomicBool, mpsc::Receiver},
    time::Duration,
};

use anyhow::Result;
use opencli_session::StoredSession;

pub struct TuiState {
    pub input: String,
    pub lines: Vec<String>,
    pub session: StoredSession,
    pub pending: Option<PendingTask>,
    pub queued_prompts: VecDeque<String>,
    pub scroll_from_bottom: u16,
    pub cancel_pending_task: bool,
    pub completed_turns: usize,
    pub last_event: String,
}

pub struct PendingTask {
    pub receiver: Receiver<Result<(StoredSession, String)>>,
    pub cancel_requested: Arc<AtomicBool>,
    pub started_at: std::time::Instant,
    pub prompt_preview: String,
}

impl TuiState {
    pub fn new(session: StoredSession) -> Self {
        Self {
            input: String::new(),
            lines: vec![
                "# opencli TUI".to_string(),
                "> Start typing below and press Enter to send your prompt.".to_string(),
                "- `Esc` confirms cancellation for the running task".to_string(),
                "- `Ctrl+L` clears the visible conversation history".to_string(),
            ],
            session,
            pending: None,
            queued_prompts: VecDeque::new(),
            scroll_from_bottom: 0,
            cancel_pending_task: false,
            completed_turns: 0,
            last_event: "idle".to_string(),
        }
    }

    pub fn pending_elapsed(&self) -> Option<Duration> {
        self.pending
            .as_ref()
            .map(|pending| pending.started_at.elapsed())
    }
}
