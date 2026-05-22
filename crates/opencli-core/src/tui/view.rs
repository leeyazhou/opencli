use opencli_output::render_markdown_tui;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use unicode_width::UnicodeWidthStr;

use super::state::{ChatMessage, TuiState, ToolStatus};

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

    // --- Body: Conversation pane + Sidebar ---
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(1), Constraint::Length(30)])
        .split(chunks[1]);

    // Conversation pane: show messages with role-based styling
    let conversation = render_conversation(state);
    let visible_height = body[0].height.saturating_sub(2);
    let max_scroll = (conversation.lines.len() as u16).saturating_sub(visible_height);
    let scroll_y = max_scroll.saturating_sub(state.scroll_from_bottom.min(max_scroll));

    let conv_title = match (state.pending.is_some(), state.queued_prompts.is_empty()) {
        (true, true) => " Conversation  waiting ".to_string(),
        (true, false) => " Conversation  waiting + queue ".to_string(),
        (false, _) => " Conversation ".to_string(),
    };
    let history = Paragraph::new(conversation)
        .block(
            Block::default()
                .title(conv_title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .wrap(Wrap { trim: false })
        .scroll((scroll_y, 0));
    frame.render_widget(history, body[0]);

    // Sidebar: status and keybindings
    let sidebar = Paragraph::new(render_sidebar(state))
        .block(
            Block::default()
                .title(" Status ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(sidebar, body[1]);

    // --- Input area ---
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

    // --- Status bar ---
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

    // Cursor positioning
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

    for msg in &state.messages {
        lines.push(Line::from(""));
        match msg {
            ChatMessage::User(text) => {
                lines.push(Line::from(vec![Span::styled(
                    " You ",
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )]));
                for line in render_markdown_tui(&[text.clone()]).lines {
                    let mut styled = line.clone();
                    styled.spans.insert(
                        0,
                        Span::styled(" │ ", Style::default().fg(Color::Green)),
                    );
                    lines.push(styled);
                }
                lines.push(Line::from(vec![Span::styled(
                    " └──────────────────────────────────────",
                    Style::default().fg(Color::DarkGray),
                )]));
            }
            ChatMessage::Assistant(text) => {
                lines.push(Line::from(vec![Span::styled(
                    " Assistant ",
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                )]));
                for line in render_markdown_tui(&[text.clone()]).lines {
                    let mut styled = line.clone();
                    styled.spans.insert(
                        0,
                        Span::styled(" │ ", Style::default().fg(Color::Blue)),
                    );
                    lines.push(styled);
                }
                lines.push(Line::from(vec![Span::styled(
                    " └──────────────────────────────────────",
                    Style::default().fg(Color::DarkGray),
                )]));
            }
            ChatMessage::Thought(text) => {
                lines.push(Line::from(vec![
                    Span::styled(
                        " Thinking ",
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Magenta)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(" {text}"),
                        Style::default()
                            .fg(Color::Magenta)
                            .add_modifier(Modifier::ITALIC),
                    ),
                ]));
            }
            ChatMessage::ToolCall { tool_name, status } => {
                let (status_text, status_color) = match status {
                    ToolStatus::Pending => ("pending", Color::Yellow),
                    ToolStatus::Running => ("running", Color::Yellow),
                    ToolStatus::Completed(outcome) => (outcome.as_str(), Color::Green),
                    ToolStatus::Failed(err) => (err.as_str(), Color::Red),
                };
                lines.push(Line::from(vec![
                    Span::styled(
                        " Tool ",
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(" {tool_name} "),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        status_text.to_string(),
                        Style::default()
                            .fg(status_color)
                            .add_modifier(Modifier::ITALIC),
                    ),
                ]));
            }
            ChatMessage::System(text) => {
                lines.push(Line::from(vec![Span::styled(
                    format!(" {text}"),
                    Style::default().fg(Color::DarkGray),
                )]));
            }
            ChatMessage::Error(text) => {
                lines.push(Line::from(vec![Span::styled(
                    format!(" Error: {text}"),
                    Style::default()
                        .fg(Color::Red)
                        .add_modifier(Modifier::BOLD),
                )]));
            }
        }
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

    // Collect message type counts
    let mut user_count = 0;
    let mut assistant_count = 0;
    let mut system_count = 0;
    for msg in &state.messages {
        match msg {
            ChatMessage::User(_) => user_count += 1,
            ChatMessage::Assistant(_) => assistant_count += 1,
            ChatMessage::System(_) => system_count += 1,
            _ => {}
        }
    }

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
                state.messages.len().to_string(),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::styled("user msgs ", Style::default().fg(Color::DarkGray)),
            Span::styled(user_count.to_string(), Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::styled("asst msgs ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                assistant_count.to_string(),
                Style::default().fg(Color::Blue),
            ),
        ]),
        Line::from(vec![
            Span::styled("sys msgs ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                system_count.to_string(),
                Style::default().fg(Color::DarkGray),
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

fn format_elapsed(duration: std::time::Duration) -> String {
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();
    format!("{}.{:03}s", secs, millis)
}
