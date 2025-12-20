use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::Terminal;
use std::io;

/// create a mock terminal for testing without actual TTY
pub fn create_test_terminal() -> Terminal<TestBackend> {
    let backend = TestBackend::new(80, 24); // standard terminal size
    Terminal::new(backend).unwrap()
}
