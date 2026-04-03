mod plain;
#[cfg(feature = "tui")]
mod tui;

pub use plain::render_markdown;
#[cfg(feature = "tui")]
pub use tui::render_markdown_tui;

#[cfg(test)]
mod tests {
    use super::render_markdown;
    #[cfg(feature = "tui")]
    use super::render_markdown_tui;

    #[test]
    fn renders_basic_markdown() {
        let input = "# Title\n\n- a\n- b\n\n```rust\nfn main() {}\n```";
        let rendered = render_markdown(input);
        assert!(rendered.contains("# Title"));
        assert!(rendered.contains("- a"));
        assert!(rendered.contains("```"));
        assert!(rendered.contains("# language: rust"));
    }

    #[test]
    #[cfg(feature = "tui")]
    fn renders_tui_lines() {
        let text = render_markdown_tui(&["# Title\n\n> note".to_string()]);
        assert!(text.lines.len() >= 2);
    }
}
