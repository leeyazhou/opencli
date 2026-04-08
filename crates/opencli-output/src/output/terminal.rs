use std::io::{self, Write};

use anyhow::Result;
use opencli_provider::Renderer;

use crate::markdown::render_markdown;

pub struct TerminalRenderer;

impl Default for TerminalRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalRenderer {
    pub fn new() -> Self {
        Self
    }
}

impl Renderer for TerminalRenderer {
    fn render_text(&mut self, text: &str) -> Result<()> {
        print!("{text}");
        io::stdout().flush()?;
        Ok(())
    }

    fn render_line(&mut self, text: &str) -> Result<()> {
        println!("{}", render_markdown(text));
        Ok(())
    }

    fn render_tool_call(&mut self, tool_name: &str) -> Result<()> {
        println!("[tool:{tool_name}]");
        Ok(())
    }
}
