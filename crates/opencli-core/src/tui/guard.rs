use std::io;

use crossterm::{
    execute,
    terminal::{LeaveAlternateScreen, disable_raw_mode},
};

pub struct TerminalRestoreGuard;

impl Default for TerminalRestoreGuard {
    fn default() -> Self {
        Self::new()
    }
}

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
