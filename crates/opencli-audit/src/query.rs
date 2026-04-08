use std::{
    fs::{read_to_string, remove_file, write},
    path::Path,
};

use anyhow::Result;
use opencli_config::RuntimeConfig;
use serde::Serialize;
use std::collections::BTreeMap;

use super::AuditRecord;

#[derive(Debug, Clone, Serialize)]
pub struct AuditStats {
    pub total_records: usize,
    pub by_event_type: BTreeMap<String, usize>,
    pub by_tool_name: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditGraphNode {
    pub agent_id: String,
    pub parent_agent_id: Option<String>,
    pub role: Option<String>,
    pub task: Option<String>,
    pub success: Option<bool>,
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

pub fn compute_agent_graph(config: &RuntimeConfig) -> Result<Vec<AuditGraphNode>> {
    let records = read_all_records(config, Some("subagent"), None)?;
    let mut graph: BTreeMap<String, AuditGraphNode> = BTreeMap::new();

    for record in records {
        let Some(agent_id) = record
            .payload
            .get("agentId")
            .and_then(|v| v.as_str())
            .map(ToString::to_string)
        else {
            continue;
        };

        let node = graph.entry(agent_id.clone()).or_insert(AuditGraphNode {
            agent_id,
            parent_agent_id: record
                .payload
                .get("parentAgentId")
                .and_then(|v| v.as_str())
                .map(ToString::to_string),
            role: record
                .payload
                .get("role")
                .and_then(|v| v.as_str())
                .map(ToString::to_string),
            task: record
                .payload
                .get("task")
                .and_then(|v| v.as_str())
                .map(ToString::to_string),
            success: None,
        });

        if record.event_type == "a2a_finish" {
            node.success = record.payload.get("success").and_then(|v| v.as_bool());
        }
    }

    Ok(graph.into_values().collect())
}

pub fn render_agent_graph(nodes: &[AuditGraphNode]) -> String {
    if nodes.is_empty() {
        return "No agent graph records found.".to_string();
    }

    let mut children: BTreeMap<Option<String>, Vec<&AuditGraphNode>> = BTreeMap::new();
    for node in nodes {
        children
            .entry(node.parent_agent_id.clone())
            .or_default()
            .push(node);
    }

    for group in children.values_mut() {
        group.sort_by(|a, b| a.agent_id.cmp(&b.agent_id));
    }

    let mut lines = Vec::new();
    render_children(&children, None, 0, &mut lines);
    lines.join("\n")
}

fn render_children(
    children: &BTreeMap<Option<String>, Vec<&AuditGraphNode>>,
    parent: Option<String>,
    depth: usize,
    lines: &mut Vec<String>,
) {
    if let Some(nodes) = children.get(&parent) {
        for node in nodes {
            let indent = "  ".repeat(depth);
            let status = match node.success {
                Some(true) => "ok",
                Some(false) => "fail",
                None => "unknown",
            };
            let role = node.role.as_deref().unwrap_or("unknown-role");
            let task = node.task.as_deref().unwrap_or("unknown-task");
            lines.push(format!(
                "{indent}- {role} [{}] {task} ({status})",
                node.agent_id
            ));
            render_children(children, Some(node.agent_id.clone()), depth + 1, lines);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AuditGraphNode, render_agent_graph};

    #[test]
    fn renders_agent_graph_tree() {
        let graph = vec![
            AuditGraphNode {
                agent_id: "parent".into(),
                parent_agent_id: None,
                role: Some("planner".into()),
                task: Some("top level".into()),
                success: Some(true),
            },
            AuditGraphNode {
                agent_id: "child".into(),
                parent_agent_id: Some("parent".into()),
                role: Some("researcher".into()),
                task: Some("child task".into()),
                success: Some(false),
            },
        ];

        let rendered = render_agent_graph(&graph);
        assert!(rendered.contains("planner [parent] top level (ok)"));
        assert!(rendered.contains("researcher [child] child task (fail)"));
    }
}
