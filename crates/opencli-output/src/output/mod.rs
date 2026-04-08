#[cfg(feature = "tui")]
mod buffer;
mod terminal;

#[cfg(feature = "tui")]
pub use buffer::BufferRenderer;
pub use terminal::TerminalRenderer;
