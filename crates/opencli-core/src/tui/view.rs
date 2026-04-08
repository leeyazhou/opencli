use opencli_output::render_markdown_tui;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use unicode_width::UnicodeWidthStr;

use super::state::TuiState;

pub fn render(frame: &mut Frame<'_>, state: &TuiState) {
    let input_height = input_height(state);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(input_height),
            Constraint::Length(2),
        ])
        .split(frame.area());

    let header = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(
                " opencli ",
                Style::default()
                    .bg(Color::Cyan)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  Workspace-aware coding assistant"),
        ]),
        Line::from(vec![
            Span::styled("session ", Style::default().fg(Color::DarkGray)),
            Span::styled(&state.session.id, Style::default().fg(Color::White)),
        ]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(header, chunks[0]);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(1), Constraint::Length(30)])
        .split(chunks[1]);

    let history_text = render_conversation(state);
    let visible_height = body[0].height.saturating_sub(2);
    let max_scroll = (history_text.lines.len() as u16).saturating_sub(visible_height);
    let scroll_y = max_scroll.saturating_sub(state.scroll_from_bottom.min(max_scroll));

    let conversation_title = if state.pending.is_some() {
        format!(
            " Conversation  waiting{} ",
            if state.queued_prompts.is_empty() {
                ""
            } else {
                " + queue"
            }
        )
    } else {
        " Conversation ".to_string()
    };

    let history = Paragraph::new(history_text)
        .block(
            Block::default()
                .title(conversation_title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .wrap(Wrap { trim: false })
        .scroll((scroll_y, 0));
    frame.render_widget(history, body[0]);

    let sidebar = Paragraph::new(render_sidebar(state))
        .block(
            Block::default()
                .title(" Sidebar ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(sidebar, body[1]);

    let input_title = if state.pending.is_some() {
        if state.cancel_pending_task {
            " Input  press Esc again to cancel "
        } else {
            " Input  response in progress "
        }
    } else {
        " Input  ready  Shift+Enter newline "
    };
    let input_widget = Paragraph::new(state.input.as_str())
        .block(
            Block::default()
                .title(input_title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(if state.cancel_pending_task {
                    Color::Red
                } else if state.pending.is_some() {
                    Color::Yellow
                } else {
                    Color::Green
                })),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(input_widget, chunks[2]);

    let status = if state.pending.is_some() {
        if state.cancel_pending_task {
            format!(
                "Esc cancel  Shift+Enter newline  Ctrl+L clear  queue:{}",
                state.queued_prompts.len()
            )
        } else {
            format!(
                "Enter queue  Shift+Enter newline  Esc confirm cancel  queue:{}",
                state.queued_prompts.len()
            )
        }
    } else {
        "Enter send  Shift+Enter newline  Ctrl+L clear  Ctrl+C exit".to_string()
    };
    let status_widget = Paragraph::new(status).style(
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::ITALIC),
    );
    frame.render_widget(status_widget, chunks[3]);

    let input_lines: Vec<&str> = state.input.split('\n').collect();
    let cursor_line = input_lines.len().saturating_sub(1) as u16;
    let current_line = input_lines.last().copied().unwrap_or("");
    let max_cursor_x = chunks[2].width.saturating_sub(2);
    let cursor_offset =
        (UnicodeWidthStr::width(current_line) as u16).min(max_cursor_x.saturating_sub(1));
    let max_cursor_y = chunks[2].height.saturating_sub(2);
    frame.set_cursor_position((
        chunks[2].x + 1 + cursor_offset,
        chunks[2].y + 1 + cursor_line.min(max_cursor_y),
    ));
}

fn input_height(state: &TuiState) -> u16 {
    let lines = state.input.lines().count().max(1) as u16;
    (lines + 2).clamp(4, 8)
}

fn render_conversation(state: &TuiState) -> Text<'static> {
    let mut lines = Vec::new();

    for block in &state.lines {
        let (title, accent, body, rail, corner) = classify_block(block);
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(
                format!(" {title} "),
                Style::default().fg(accent).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "  ·─────────────────────────────────────",
                Style::default().fg(Color::DarkGray),
            ),
        ]));

        for line in render_markdown_tui(&[body]).lines {
            let mut styled_line = line.clone();
            styled_line.spans.insert(
                0,
                Span::styled(format!(" {rail} "), Style::default().fg(accent)),
            );
            lines.push(styled_line);
        }

        lines.push(Line::from(vec![Span::styled(
            format!(" {corner}──────────────────────────────────────"),
            Style::default().fg(Color::DarkGray),
        )]));
    }

    Text::from(lines)
}

fn render_sidebar(state: &TuiState) -> Text<'static> {
    let mode = if state.cancel_pending_task {
        "confirm cancel"
    } else if state.pending.is_some() {
        "waiting"
    } else {
        "idle"
    };

    Text::from(vec![
        Line::from(vec![
            Span::styled("mode ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                mode,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("queue ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                state.queued_prompts.len().to_string(),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(vec![
            Span::styled("messages ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                state.lines.len().to_string(),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::styled("turns ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                state.completed_turns.to_string(),
                Style::default().fg(Color::LightBlue),
            ),
        ]),
        Line::from(vec![
            Span::styled("elapsed ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                state
                    .pending_elapsed()
                    .map(format_elapsed)
                    .unwrap_or_else(|| "-".to_string()),
                Style::default().fg(Color::Magenta),
            ),
        ]),
        Line::from(vec![
            Span::styled("input lines ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                state.input.lines().count().max(1).to_string(),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Activity",
            Style::default()
                .fg(Color::LightCyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(state.last_event.clone()),
        if let Some(pending) = &state.pending {
            Line::from(format!("now: {}", pending.prompt_preview))
        } else {
            Line::from("now: idle")
        },
        Line::from(""),
        Line::from(Span::styled(
            "Keys",
            Style::default()
                .fg(Color::LightCyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(" Enter        send / queue"),
        Line::from(" Shift+Enter  newline"),
        Line::from(" Esc          cancel task"),
        Line::from(" Ctrl+L       clear view"),
        Line::from(" Ctrl+C       exit"),
        Line::from(" Up/Down      scroll"),
        Line::from(""),
        Line::from(Span::styled(
            "Current session",
            Style::default()
                .fg(Color::LightCyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(state.session.title.clone()),
    ])
}

fn classify_block(block: &str) -> (&'static str, Color, String, char, char) {
    if let Some(rest) = block.strip_prefix("> ") {
        ("You", Color::Green, rest.to_string(), '│', '└')
    } else if block.starts_with("Error:") {
        ("Error", Color::Red, block.to_string(), '┃', '┗')
    } else if block.starts_with('#') || block.starts_with("- ") {
        ("System", Color::DarkGray, block.to_string(), '┆', '└')
    } else {
        ("Assistant", Color::Blue, block.to_string(), '│', '└')
    }
}

fn format_elapsed(duration: std::time::Duration) -> String {
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();
    format!("{}.{:03}s", secs, millis)
}
