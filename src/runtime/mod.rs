use anyhow::Result;

use crate::{
    approval::{Approval, PolicyApproval},
    audit::{AuditLogger, FileAuditLogger},
    config::{load_config, ConfigOverrides, RuntimeConfig},
    output::TerminalRenderer,
    provider::Provider,
    provider_factory::ProviderFactory,
    tools::ToolRegistry,
};

pub struct Runtime {
    pub config: RuntimeConfig,
    pub tool_registry: ToolRegistry,
    pub renderer: TerminalRenderer,
    pub approval: Box<dyn Approval>,
    pub audit: Box<dyn AuditLogger>,
    pub provider_factory: ProviderFactory,
}

impl Runtime {
    pub fn from_overrides(overrides: ConfigOverrides) -> Result<Self> {
        Ok(Self {
            config: load_config(overrides)?,
            tool_registry: ToolRegistry::new(),
            renderer: TerminalRenderer::new(),
            approval: Box::new(PolicyApproval::new()),
            audit: Box::new(FileAuditLogger::new()),
            provider_factory: ProviderFactory::new(),
        })
    }

    pub fn provider(&self) -> Result<Box<dyn Provider>> {
        self.provider_factory.create(&self.config)
    }
}
