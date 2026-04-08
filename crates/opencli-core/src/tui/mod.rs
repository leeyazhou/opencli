mod background;
mod guard;
mod state;
mod view;

use std::{io, time::Duration};

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::app::App;

use self::background::{poll_background, spawn_response_task};
use self::guard::TerminalRestoreGuard;
use self::state::TuiState;
use self::view::render;

pub async fn run_chat_tui(app: &mut App) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let _guard = TerminalRestoreGuard::new();
    let mut state = TuiState::new(app.create_runtime_session()?);

    let result = loop {
        poll_background(&mut state);

        if state.pending.is_none()
            && let Some(prompt) = state.queued_prompts.pop_front()
        {
            state.lines.push(format!("> {prompt}"));
            state.scroll_from_bottom = 0;
            state.pending = Some(spawn_response_task(
                app.runtime_config().clone(),
                state.session.clone(),
                prompt,
            ));
        }

        terminal.draw(|frame| render(frame, &state))?;

        if !event::poll(Duration::from_millis(50))? {
            continue;
        }

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match key.code {
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break Ok(()),
                KeyCode::Esc => {
                    if state.pending.is_some() {
                        if state.cancel_pending_task {
                            if let Some(pending) = &state.pending {
                                pending
                                    .cancel_requested
                                    .store(true, std::sync::atomic::Ordering::Relaxed);
                            }
                            state.cancel_pending_task = false;
                            state
                                .lines
                                .push("Cancel requested for current task.".to_string());
                            state.scroll_from_bottom = 0;
                        } else {
                            state.cancel_pending_task = true;
                        }
                    }
                }
                KeyCode::Up => {
                    state.scroll_from_bottom = state.scroll_from_bottom.saturating_add(1)
                }
                KeyCode::Down => {
                    state.scroll_from_bottom = state.scroll_from_bottom.saturating_sub(1)
                }
                KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    state.lines.clear();
                    state.lines.push("Conversation cleared.".to_string());
                    state.scroll_from_bottom = 0;
                }
                KeyCode::Enter => {
                    let prompt = state.input.trim().to_string();
                    if !prompt.is_empty() {
                        if state.pending.is_none() {
                            state.lines.push(format!("> {prompt}"));
                            state.scroll_from_bottom = 0;
                            state.pending = Some(spawn_response_task(
                                app.runtime_config().clone(),
                                state.session.clone(),
                                prompt,
                            ));
                        } else {
                            state.queued_prompts.push_back(prompt);
                        }
                    }
                    state.input.clear();
                }
                KeyCode::Backspace => {
                    state.cancel_pending_task = false;
                    state.input.pop();
                }
                KeyCode::Char(ch) => {
                    state.cancel_pending_task = false;
                    state.input.push(ch);
                }
                _ => {
                    state.cancel_pending_task = false;
                }
            }
        }
    };

    terminal.show_cursor()?;
    result
}
