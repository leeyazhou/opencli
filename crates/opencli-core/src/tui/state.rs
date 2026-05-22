use std::{
    collections::VecDeque,
    sync::{Arc, atomic::AtomicBool, mpsc::Receiver},
    time::Duration,
};

use anyhow::Result;
use opencli_session::StoredSession;

use crate::app::DetachedSessionTurnResult;

pub struct TuiState {
    pub input: String,
    pub messages: Vec<ChatMessage>,
    pub session: StoredSession,
    pub pending: Option<PendingTask>,
    pub queued_prompts: VecDeque<String>,
    pub scroll_from_bottom: u16,
    pub cancel_pending_task: bool,
    pub completed_turns: usize,
    pub last_event: String,
}

pub struct PendingTask {
    pub receiver: Receiver<Result<DetachedSessionTurnResult>>,
    pub cancel_requested: Arc<AtomicBool>,
    pub started_at: std::time::Instant,
    pub prompt_preview: String,
}

/// Structured chat message for the TUI conversation pane.
#[derive(Debug, Clone)]
pub enum ChatMessage {
    /// A user prompt
    User(String),
    /// The assistant's final response text
    Assistant(String),
    /// System informational or help text
    System(String),
    /// The agent's thinking/reasoning text
    Thought(String),
    /// A tool call in progress
    ToolCall {
        tool_name: String,
        status: ToolStatus,
    },
    /// An error message
    Error(String),
}

#[derive(Debug, Clone)]
pub enum ToolStatus {
    Pending,
    Running,
    Completed(String),
    Failed(String),
}

impl TuiState {
    pub fn new(session: StoredSession) -> Self {
        Self {
            input: String::new(),
            messages: vec![
                ChatMessage::System("# opencli TUI".to_string()),
                ChatMessage::System(
                    "Start typing below and press Enter to send your prompt.".to_string(),
                ),
                ChatMessage::System(
                    "- `Esc` confirms cancellation for the running task".to_string(),
                ),
                ChatMessage::System(
                    "- `Ctrl+L` clears the visible conversation history".to_string(),
                ),
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
