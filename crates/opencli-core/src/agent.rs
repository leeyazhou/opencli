use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use anyhow::{Result, bail};
use opencli_provider::{ChatMessage, Provider, Renderer};

use crate::tools::{ToolExecutionContext, ToolRegistry};

pub async fn run_agent_loop(
    provider: &dyn Provider,
    tool_registry: &ToolRegistry,
    renderer: &mut dyn Renderer,
    tool_context: &ToolExecutionContext<'_>,
    cancel_requested: Option<&Arc<AtomicBool>>,
    max_steps: usize,
    messages: &mut Vec<ChatMessage>,
) -> Result<String> {
    for _ in 0..max_steps {
        ensure_not_canceled(cancel_requested)?;
        let response = provider
            .complete_chat(messages, &tool_registry.definitions(), cancel_requested)
            .await?;
        let assistant =
            ChatMessage::assistant(response.content.clone(), response.tool_calls.clone());
        messages.push(assistant);

        if response.tool_calls.is_empty() {
            if !response.content.is_empty() {
                renderer.render_line(&response.content)?;
            }
            return Ok(response.content);
        }

        for tool_call in response.tool_calls {
            ensure_not_canceled(cancel_requested)?;
            renderer.render_tool_call(&tool_call.name)?;
            let content = match tool_registry.execute(&tool_call, tool_context).await {
                Ok(result) => result,
                Err(error) => format!("Tool execution failed: {error:#}"),
            };

            messages.push(ChatMessage::tool(tool_call, content));
        }
    }

    bail!("agent loop exceeded {max_steps} steps")
}

fn ensure_not_canceled(cancel_requested: Option<&Arc<AtomicBool>>) -> Result<()> {
    if cancel_requested.is_some_and(|flag| flag.load(Ordering::Relaxed)) {
        bail!("agent execution canceled")
    }
    Ok(())
}

pub async fn run_streaming_text(
    provider: &dyn Provider,
    renderer: &mut dyn Renderer,
    messages: &[ChatMessage],
) -> Result<String> {
    provider.stream_text(messages, renderer, None).await
}
