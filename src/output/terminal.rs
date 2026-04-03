use std::io::{self, Write};

use anyhow::Result;

use crate::markdown::render_markdown;

use super::Renderer;

pub struct TerminalRenderer;

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
