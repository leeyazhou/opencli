use std::{
    fs::{create_dir_all, OpenOptions},
    io::Write,
    path::Path,
};

use anyhow::Result;
use chrono::Utc;
use serde::Serialize;
use serde_json::Value;

use crate::config::RuntimeConfig;

pub trait AuditLogger: Send + Sync {
    fn log(&self, config: &RuntimeConfig, event: AuditEvent<'_>) -> Result<()>;
}

pub struct FileAuditLogger;

impl FileAuditLogger {
    pub fn new() -> Self {
        Self
    }
}

impl AuditLogger for FileAuditLogger {
    fn log(&self, config: &RuntimeConfig, event: AuditEvent<'_>) -> Result<()> {
        let audit_path = Path::new(&config.audit_log_path);
        if let Some(parent) = audit_path.parent() {
            create_dir_all(parent)?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(audit_path)?;
        writeln!(file, "{}", serde_json::to_string(&event.with_timestamp())?)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditEvent<'a> {
    pub event_type: &'a str,
    pub tool_name: &'a str,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct AuditRecord {
    pub timestamp: String,
    pub event_type: String,
    pub tool_name: String,
    pub payload: Value,
}

impl<'a> AuditEvent<'a> {
    fn with_timestamp(self) -> AuditRecord {
        AuditRecord {
            timestamp: Utc::now().to_rfc3339(),
            event_type: self.event_type.to_string(),
            tool_name: self.tool_name.to_string(),
            payload: self.payload,
        }
    }
}
