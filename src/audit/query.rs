use std::{
    fs::{read_to_string, remove_file, write},
    path::Path,
};

use anyhow::Result;
use serde::Serialize;
use std::collections::BTreeMap;

use crate::config::RuntimeConfig;

use super::AuditRecord;

#[derive(Debug, Clone, Serialize)]
pub struct AuditStats {
    pub total_records: usize,
    pub by_event_type: BTreeMap<String, usize>,
    pub by_tool_name: BTreeMap<String, usize>,
}

pub fn read_records(
    config: &RuntimeConfig,
    limit: usize,
    tool: Option<&str>,
    event_type: Option<&str>,
) -> Result<Vec<AuditRecord>> {
    let mut records = read_all_records(config, tool, event_type)?;
    records.reverse();
    records.truncate(limit);
    Ok(records)
}

pub fn read_all_records(
    config: &RuntimeConfig,
    tool: Option<&str>,
    event_type: Option<&str>,
) -> Result<Vec<AuditRecord>> {
    let path = Path::new(&config.audit_log_path);
    if !path.exists() {
        return Ok(Vec::new());
    }

    Ok(read_to_string(path)?
        .lines()
        .filter_map(|line| serde_json::from_str::<AuditRecord>(line).ok())
        .filter(|record| tool.is_none_or(|value| record.tool_name == value))
        .filter(|record| event_type.is_none_or(|value| record.event_type == value))
        .collect())
}

pub fn clear_records(config: &RuntimeConfig) -> Result<()> {
    let path = Path::new(&config.audit_log_path);
    if path.exists() {
        remove_file(path)?;
    }
    Ok(())
}

pub fn export_records(
    config: &RuntimeConfig,
    output: &str,
    tool: Option<&str>,
    event_type: Option<&str>,
) -> Result<usize> {
    let records = read_all_records(config, tool, event_type)?;
    write(
        output,
        format!("{}\n", serde_json::to_string_pretty(&records)?),
    )?;
    Ok(records.len())
}

pub fn compute_stats(config: &RuntimeConfig) -> Result<AuditStats> {
    let records = read_all_records(config, None, None)?;
    let mut by_event_type = BTreeMap::new();
    let mut by_tool_name = BTreeMap::new();

    for record in &records {
        *by_event_type.entry(record.event_type.clone()).or_insert(0) += 1;
        *by_tool_name.entry(record.tool_name.clone()).or_insert(0) += 1;
    }

    Ok(AuditStats {
        total_records: records.len(),
        by_event_type,
        by_tool_name,
    })
}
