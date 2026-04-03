use anyhow::Result;

use crate::markdown::render_markdown;

use super::Renderer;

pub struct BufferRenderer {
    buffer: String,
}

impl BufferRenderer {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }
}

impl Renderer for BufferRenderer {
    fn render_text(&mut self, text: &str) -> Result<()> {
        self.buffer.push_str(text);
        Ok(())
    }

    fn render_line(&mut self, text: &str) -> Result<()> {
        self.buffer.push_str(&render_markdown(text));
        self.buffer.push('\n');
        Ok(())
    }

    fn render_tool_call(&mut self, tool_name: &str) -> Result<()> {
        self.buffer.push_str(&format!("[tool:{tool_name}]\n"));
        Ok(())
    }
}
