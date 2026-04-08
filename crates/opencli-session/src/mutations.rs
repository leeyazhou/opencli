use chrono::Utc;
use opencli_provider::ChatMessage;

use super::StoredSession;

pub fn append_message(session: &mut StoredSession, message: ChatMessage) {
    if session.title == "Untitled session" && message.role == "user" {
        session.title = message.content.chars().take(80).collect();
    }
    session.messages.push(message);
    session.updated_at = Utc::now();
}
