use std::{env, fs, path::PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct RuntimeConfig {
    pub provider: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub anthropic_version: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub approval_mode: String,
    pub non_interactive_approval: String,
    pub workspace_root: String,
    pub session_dir: String,
    pub audit_log_path: String,
    pub request_timeout_ms: u64,
    pub shell_timeout_ms: u64,
    pub agent_max_steps: usize,
    pub a2a_enabled: bool,
    pub a2a_max_depth: u32,
    pub a2a_max_concurrency: usize,
    pub allowed_tool_kinds: Vec<String>,
    pub allowed_tools: Vec<String>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            provider: "openai-compatible".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            api_key: String::new(),
            model: "gpt-4.1".to_string(),
            anthropic_version: "2023-06-01".to_string(),
            temperature: 0.2,
            max_tokens: 4096,
            approval_mode: "on-write".to_string(),
            non_interactive_approval: "deny".to_string(),
            workspace_root: ".".to_string(),
            session_dir: "~/.config/opencli/sessions".to_string(),
            audit_log_path: "~/.config/opencli/audit.jsonl".to_string(),
            request_timeout_ms: 120_000,
            shell_timeout_ms: 120_000,
            agent_max_steps: 8,
            a2a_enabled: true,
            a2a_max_depth: 2,
            a2a_max_concurrency: 4,
            allowed_tool_kinds: vec![
                "filesystem-read".to_string(),
                "filesystem-search".to_string(),
                "shell".to_string(),
                "agent".to_string(),
            ],
            allowed_tools: Vec::new(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct ConfigOverrides {
    pub model: Option<String>,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
}

pub fn load_config(overrides: ConfigOverrides) -> Result<RuntimeConfig> {
    let path = config_path()?;
    let file_config = match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str::<RuntimeConfig>(&content)
            .with_context(|| format!("failed to parse config file at {}", path.display()))?,
        Err(_) => RuntimeConfig::default(),
    };

    let mut config = file_config;

    if config.provider.is_empty() {
        config.provider = "openai-compatible".to_string();
    }
    if config.base_url.is_empty() {
        config.base_url = default_base_url(&config.provider).to_string();
    }
    if config.anthropic_version.is_empty() {
        config.anthropic_version = "2023-06-01".to_string();
    }
    if config.approval_mode.is_empty() {
        config.approval_mode = "on-write".to_string();
    }
    if config.non_interactive_approval.is_empty() {
        config.non_interactive_approval = "deny".to_string();
    }
    if config.audit_log_path.is_empty() {
        config.audit_log_path = "~/.config/opencli/audit.jsonl".to_string();
    }
    if config.request_timeout_ms == 0 {
        config.request_timeout_ms = 120_000;
    }
    if config.shell_timeout_ms == 0 {
        config.shell_timeout_ms = 120_000;
    }
    if config.agent_max_steps == 0 {
        config.agent_max_steps = 8;
    }
    if config.a2a_max_depth == 0 {
        config.a2a_max_depth = 2;
    }
    if config.a2a_max_concurrency == 0 {
        config.a2a_max_concurrency = 4;
    }
    if config.allowed_tool_kinds.is_empty() {
        config.allowed_tool_kinds = RuntimeConfig::default().allowed_tool_kinds;
    }

    if let Some(model) = overrides.model.or_else(|| env::var("OPENCLI_MODEL").ok()) {
        config.model = model;
    }
    if let Some(base_url) = overrides
        .base_url
        .or_else(|| env::var("OPENCLI_BASE_URL").ok())
    {
        config.base_url = base_url;
    }
    if let Some(api_key) = overrides
        .api_key
        .or_else(|| env::var("OPENCLI_API_KEY").ok())
    {
        config.api_key = api_key;
    }

    config.workspace_root = expand_home(&config.workspace_root)?.display().to_string();
    config.session_dir = expand_home(&config.session_dir)?.display().to_string();
    config.audit_log_path = expand_home(&config.audit_log_path)?.display().to_string();

    Ok(config)
}

pub fn redacted_config(config: &RuntimeConfig) -> serde_json::Value {
    serde_json::json!({
        "provider": config.provider,
        "baseUrl": config.base_url,
        "apiKey": if config.api_key.is_empty() { "" } else { "***redacted***" },
        "model": config.model,
        "anthropicVersion": config.anthropic_version,
        "temperature": config.temperature,
        "maxTokens": config.max_tokens,
        "approvalMode": config.approval_mode,
        "nonInteractiveApproval": config.non_interactive_approval,
        "workspaceRoot": config.workspace_root,
        "sessionDir": config.session_dir,
        "auditLogPath": config.audit_log_path,
        "requestTimeoutMs": config.request_timeout_ms,
        "shellTimeoutMs": config.shell_timeout_ms,
        "agentMaxSteps": config.agent_max_steps,
        "a2aEnabled": config.a2a_enabled,
        "a2aMaxDepth": config.a2a_max_depth,
        "a2aMaxConcurrency": config.a2a_max_concurrency,
        "allowedToolKinds": config.allowed_tool_kinds,
        "allowedTools": config.allowed_tools,
    })
}

pub fn ensure_default_config() -> Result<PathBuf> {
    let path = config_path()?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    if !path.exists() {
        fs::write(
            &path,
            format!(
                "{}\n",
                serde_json::to_string_pretty(&RuntimeConfig::default())?
            ),
        )?;
    }

    Ok(path)
}

pub fn expand_home(input: &str) -> Result<PathBuf> {
    if input == "~" {
        return dirs::home_dir().context("home directory not found");
    }
    if let Some(rest) = input.strip_prefix("~/") {
        return Ok(dirs::home_dir()
            .context("home directory not found")?
            .join(rest));
    }
    Ok(PathBuf::from(input))
}

fn config_path() -> Result<PathBuf> {
    Ok(dirs::home_dir()
        .context("home directory not found")?
        .join(".config")
        .join("opencli")
        .join("config.json"))
}

fn default_base_url(provider: &str) -> &'static str {
    match provider {
        "anthropic" => "https://api.anthropic.com/v1",
        _ => "https://api.openai.com/v1",
    }
}
