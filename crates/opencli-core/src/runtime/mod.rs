use anyhow::Result;
use opencli_audit::{AuditLogger, FileAuditLogger};
use opencli_config::{ConfigOverrides, RuntimeConfig, load_config};
use opencli_output::TerminalRenderer;
use opencli_provider::{Provider, ProviderFactory};

use crate::tools::ToolRegistry;

pub struct Runtime {
    pub config: RuntimeConfig,
    pub tool_registry: ToolRegistry,
    pub renderer: TerminalRenderer,
    pub audit: Box<dyn AuditLogger>,
}

impl Runtime {
    pub fn from_overrides(overrides: ConfigOverrides) -> Result<Self> {
        Ok(Self {
            config: load_config(overrides)?,
            tool_registry: ToolRegistry::new(),
            renderer: TerminalRenderer::new(),
            audit: Box::new(FileAuditLogger::new()),
        })
    }

    pub fn provider(&self) -> Result<Box<dyn Provider>> {
        ProviderFactory::new().create(&self.config)
    }
}
