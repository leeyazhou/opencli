use std::io;

use crossterm::{
    execute,
    terminal::{disable_raw_mode, LeaveAlternateScreen},
};

pub struct TerminalRestoreGuard;

impl TerminalRestoreGuard {
    pub fn new() -> Self {
        Self
    }
}

impl Drop for TerminalRestoreGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen);
    }
}
