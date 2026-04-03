use anyhow::Result;

pub trait Renderer {
    fn render_text(&mut self, text: &str) -> Result<()>;
    fn render_line(&mut self, text: &str) -> Result<()>;
    fn render_tool_call(&mut self, tool_name: &str) -> Result<()>;
}
