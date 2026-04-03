use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use chrono::Utc;
use uuid::Uuid;

use crate::config::RuntimeConfig;

use super::StoredSession;

pub fn create_session(config: &RuntimeConfig) -> Result<StoredSession> {
    let now = Utc::now();
    let session = StoredSession {
        id: Uuid::new_v4().to_string(),
        title: "Untitled session".to_string(),
        model: config.model.clone(),
        created_at: now,
        updated_at: now,
        messages: Vec::new(),
    };
    save_session(config, &session)?;
    Ok(session)
}

pub fn save_session(config: &RuntimeConfig, session: &StoredSession) -> Result<()> {
    fs::create_dir_all(&config.session_dir)?;
    fs::write(
        session_path(config, &session.id),
        format!("{}\n", serde_json::to_string_pretty(session)?),
    )?;
    Ok(())
}

pub fn load_session(config: &RuntimeConfig, id: &str) -> Result<StoredSession> {
    let content = fs::read_to_string(session_path(config, id))?;
    serde_json::from_str(&content).context("failed to parse session file")
}

pub fn list_sessions(config: &RuntimeConfig) -> Result<Vec<StoredSession>> {
    let dir = PathBuf::from(&config.session_dir);
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut sessions = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        let content = fs::read_to_string(entry.path())?;
        let session = serde_json::from_str::<StoredSession>(&content)
            .with_context(|| format!("failed to parse session file {}", entry.path().display()))?;
        sessions.push(session);
    }

    sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(sessions)
}

pub fn delete_session(config: &RuntimeConfig, id: &str) -> Result<()> {
    let path = session_path(config, id);
    if !path.exists() {
        anyhow::bail!("session not found: {id}");
    }
    fs::remove_file(path)?;
    Ok(())
}

pub fn rename_session(config: &RuntimeConfig, id: &str, title: &str) -> Result<()> {
    let mut session = load_session(config, id)?;
    session.title = title.to_string();
    session.updated_at = Utc::now();
    save_session(config, &session)
}

fn session_path(config: &RuntimeConfig, id: &str) -> PathBuf {
    PathBuf::from(&config.session_dir).join(format!("{id}.json"))
}
