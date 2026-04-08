use opencli_output::render_markdown_tui;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use unicode_width::UnicodeWidthStr;

use super::state::TuiState;

pub fn render(frame: &mut Frame<'_>, state: &TuiState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let history_text = render_markdown_tui(&state.lines);
    let visible_height = chunks[0].height.saturating_sub(2);
    let max_scroll = (history_text.lines.len() as u16).saturating_sub(visible_height);
    let scroll_y = max_scroll.saturating_sub(state.scroll_from_bottom.min(max_scroll));

    let history = Paragraph::new(history_text)
        .block(Block::default().title("Conversation").borders(Borders::ALL))
        .wrap(Wrap { trim: false })
        .scroll((scroll_y, 0));
    frame.render_widget(history, chunks[0]);

    let input_title = if state.pending.is_some() {
        if state.cancel_pending_task {
            "Input (press Esc again to cancel task)"
        } else {
            "Input (waiting for response...)"
        }
    } else {
        "Input"
    };
    let input_widget = Paragraph::new(state.input.as_str())
        .block(Block::default().title(input_title).borders(Borders::ALL));
    frame.render_widget(input_widget, chunks[1]);

    let status = if state.pending.is_some() {
        if state.cancel_pending_task {
            if state.queued_prompts.is_empty() {
                "Ctrl+C exit | Esc cancel task | Up/Down scroll".to_string()
            } else {
                format!(
                    "Ctrl+C exit | Esc cancel task | Up/Down scroll | Queued: {}",
                    state.queued_prompts.len()
                )
            }
        } else if state.queued_prompts.is_empty() {
            "Ctrl+C exit | Esc confirm cancel | Ctrl+L clear | Up/Down scroll | Waiting".to_string()
        } else {
            format!(
                "Ctrl+C exit | Esc confirm cancel | Ctrl+L clear | Up/Down scroll | Waiting | Queued: {}",
                state.queued_prompts.len()
            )
        }
    } else {
        "Ctrl+C exit | Enter send | Ctrl+L clear | Up/Down scroll".to_string()
    };
    let status_widget = Paragraph::new(status).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(status_widget, chunks[2]);

    let max_cursor_x = chunks[1].width.saturating_sub(2);
    let cursor_offset =
        (UnicodeWidthStr::width(state.input.as_str()) as u16).min(max_cursor_x.saturating_sub(1));
    frame.set_cursor_position((chunks[1].x + 1 + cursor_offset, chunks[1].y + 1));
}
