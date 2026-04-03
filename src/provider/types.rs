use anyhow::Result;
use async_trait::async_trait;
use std::sync::{Arc, atomic::AtomicBool};

use crate::{message::ChatMessage, output::Renderer, tools::{ToolCall, ToolDefinition}};

#[derive(Debug)]
pub struct ChatResponse {
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct ProviderCapabilities {
    pub supports_streaming: bool,
    pub supports_tools: bool,
    pub supports_model_listing: bool,
}

#[async_trait(?Send)]
pub trait Provider: Send + Sync {
    fn name(&self) -> &'static str;
    fn capabilities(&self) -> ProviderCapabilities;

    async fn complete_chat(
        &self,
        messages: &[ChatMessage],
        tools: &[ToolDefinition],
        cancel_requested: Option<&Arc<AtomicBool>>,
    ) -> Result<ChatResponse>;

    async fn stream_text(
        &self,
        messages: &[ChatMessage],
        renderer: &mut dyn Renderer,
        cancel_requested: Option<&Arc<AtomicBool>>,
    ) -> Result<String>;

    async fn list_models(&self) -> Result<Vec<String>>;
}
