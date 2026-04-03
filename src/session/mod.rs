mod mutations;
mod store;
mod types;

pub use mutations::append_message;
pub use store::{
    create_session, delete_session, list_sessions, load_session, rename_session, save_session,
};
pub use types::StoredSession;

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::{config::RuntimeConfig, message::ChatMessage};

    use super::{
        append_message, create_session, delete_session, list_sessions, load_session,
        rename_session, save_session,
    };

    #[test]
    fn persists_session() {
        let temp_dir = std::env::temp_dir().join(format!("ai-cli-sessions-{}", Uuid::new_v4()));
        let config = RuntimeConfig {
            session_dir: temp_dir.to_string_lossy().to_string(),
            ..RuntimeConfig::default()
        };

        let mut session = create_session(&config).unwrap();
        append_message(&mut session, ChatMessage::user("hello"));
        save_session(&config, &session).unwrap();

        let loaded = load_session(&config, &session.id).unwrap();
        assert_eq!(loaded.messages.len(), 1);
        assert_eq!(list_sessions(&config).unwrap().len(), 1);
    }

    #[test]
    fn deletes_session() {
        let temp_dir = std::env::temp_dir().join(format!("ai-cli-sessions-{}", Uuid::new_v4()));
        let config = RuntimeConfig {
            session_dir: temp_dir.to_string_lossy().to_string(),
            ..RuntimeConfig::default()
        };

        let session = create_session(&config).unwrap();
        delete_session(&config, &session.id).unwrap();
        assert!(list_sessions(&config).unwrap().is_empty());
    }

    #[test]
    fn renames_session() {
        let temp_dir = std::env::temp_dir().join(format!("ai-cli-sessions-{}", Uuid::new_v4()));
        let config = RuntimeConfig {
            session_dir: temp_dir.to_string_lossy().to_string(),
            ..RuntimeConfig::default()
        };

        let session = create_session(&config).unwrap();
        rename_session(&config, &session.id, "renamed title").unwrap();
        let loaded = load_session(&config, &session.id).unwrap();
        assert_eq!(loaded.title, "renamed title");
    }
}
