#[cfg(feature = "anthropic")]
mod anthropic;
mod openai_compatible;
mod types;
mod util;

#[cfg(feature = "anthropic")]
pub use anthropic::AnthropicProvider;
pub use openai_compatible::OpenAiCompatibleProvider;
pub use types::{ChatResponse, Provider, ProviderCapabilities, ProviderConfig};

#[cfg(test)]
mod tests {
    use opencli_tools::ToolCall;

    use crate::message::ChatMessage;

    #[cfg(feature = "anthropic")]
    use super::util::serialize_anthropic_message;
    use super::util::serialize_openai_message;

    #[test]
    fn serializes_openai_tool_message() {
        let message = ChatMessage::tool(
            ToolCall {
                id: "tool_1".into(),
                name: "read_file".into(),
                arguments: "{}".into(),
            },
            "file content",
        );

        let value = serialize_openai_message(&message);
        assert_eq!(value["role"], "tool");
        assert_eq!(value["tool_call_id"], "tool_1");
    }

    #[test]
    #[cfg(feature = "anthropic")]
    fn serializes_anthropic_tool_result_message() {
        use serde_json::Value;

        let message = ChatMessage::tool(
            ToolCall {
                id: "tool_1".into(),
                name: "run_shell".into(),
                arguments: "{}".into(),
            },
            "result",
        );

        let value: Value = serialize_anthropic_message(&message);
        assert_eq!(value["role"], "user");
        assert_eq!(value["content"][0]["type"], "tool_result");
    }
}
