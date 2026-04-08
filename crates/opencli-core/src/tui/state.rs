use std::{
    collections::VecDeque,
    sync::{Arc, atomic::AtomicBool, mpsc::Receiver},
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
}

pub struct PendingTask {
    pub receiver: Receiver<Result<(StoredSession, String)>>,
    pub cancel_requested: Arc<AtomicBool>,
}

impl TuiState {
    pub fn new(session: StoredSession) -> Self {
        Self {
            input: String::new(),
            lines: vec!["TUI chat started. Press Esc to quit.".to_string()],
            session,
            pending: None,
            queued_prompts: VecDeque::new(),
            scroll_from_bottom: 0,
            cancel_pending_task: false,
        }
    }
}
