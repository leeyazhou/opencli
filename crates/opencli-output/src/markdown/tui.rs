use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
};

use super::plain::render_markdown;

pub fn render_markdown_tui(blocks: &[String]) -> Text<'static> {
    let mut lines = Vec::new();
    for block in blocks {
        let rendered = render_markdown(block);
        let mut in_code_block = false;
        for line in rendered.lines() {
            if line.trim() == "```" {
                in_code_block = !in_code_block;
                lines.push(Line::from(Span::styled(
                    "```",
                    Style::default().fg(Color::DarkGray),
                )));
                continue;
            }
            let styled = if in_code_block {
                Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(Color::Yellow),
                ))
            } else if let Some(rest) = line.strip_prefix("# ") {
                Line::from(Span::styled(
                    format!("# {rest}"),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ))
            } else if let Some(rest) = line.strip_prefix("## ") {
                Line::from(Span::styled(
                    format!("## {rest}"),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ))
            } else if let Some(rest) = line.strip_prefix("### ") {
                Line::from(Span::styled(
                    format!("### {rest}"),
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                ))
            } else if line.starts_with('>') {
                Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(Color::Green),
                ))
            } else if line.starts_with("- ") || ordered_list_line(line) {
                Line::from(Span::styled(
                    line.to_string(),
                    Style::default().fg(Color::Magenta),
                ))
            } else {
                Line::from(line.to_string())
            };
            lines.push(styled);
        }
        lines.push(Line::from(""));
    }
    Text::from(lines)
}

fn ordered_list_line(line: &str) -> bool {
    let mut chars = line.chars();
    let mut saw_digit = false;
    while let Some(ch) = chars.next() {
        if ch.is_ascii_digit() {
            saw_digit = true;
            continue;
        }
        return saw_digit && ch == '.' && chars.next() == Some(' ');
    }
    false
}
