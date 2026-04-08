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
    use opencli_config::RuntimeConfig;
    use opencli_provider::ChatMessage;
    use uuid::Uuid;

    use super::{
        append_message, create_session, delete_session, list_sessions, load_session,
        rename_session, save_session,
    };

    #[test]
    fn persists_session() {
        let temp_dir = std::env::temp_dir().join(format!("opencli-sessions-{}", Uuid::new_v4()));
        let config = RuntimeConfig {
            session_dir: temp_dir.to_string_lossy().to_string(),
            ..RuntimeConfig::default()
        };

        let mut session = create_session(&config).expect("session creation should succeed");
        append_message(&mut session, ChatMessage::user("hello"));
        save_session(&config, &session).expect("session save should succeed");

        let loaded = load_session(&config, &session.id).expect("session load should succeed");
        assert_eq!(loaded.messages.len(), 1);
        assert_eq!(
            list_sessions(&config)
                .expect("session listing should succeed")
                .len(),
            1
        );
    }

    #[test]
    fn deletes_session() {
        let temp_dir = std::env::temp_dir().join(format!("opencli-sessions-{}", Uuid::new_v4()));
        let config = RuntimeConfig {
            session_dir: temp_dir.to_string_lossy().to_string(),
            ..RuntimeConfig::default()
        };

        let session = create_session(&config).expect("session creation should succeed");
        delete_session(&config, &session.id).expect("session deletion should succeed");
        assert!(
            list_sessions(&config)
                .expect("session listing should succeed")
                .is_empty()
        );
    }

    #[test]
    fn renames_session() {
        let temp_dir = std::env::temp_dir().join(format!("opencli-sessions-{}", Uuid::new_v4()));
        let config = RuntimeConfig {
            session_dir: temp_dir.to_string_lossy().to_string(),
            ..RuntimeConfig::default()
        };

        let session = create_session(&config).expect("session creation should succeed");
        rename_session(&config, &session.id, "renamed title")
            .expect("session rename should succeed");
        let loaded = load_session(&config, &session.id).expect("session load should succeed");
        assert_eq!(loaded.title, "renamed title");
    }
}
