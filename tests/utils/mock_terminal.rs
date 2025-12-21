use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::Terminal;

/// create a mock terminal for testing without actual TTY
pub fn create_test_terminal() -> Terminal<TestBackend> {
    let backend = TestBackend::new(80, 24); // standard terminal size
    Terminal::new(backend).unwrap()
}

pub mod events {
    use super::*;

    pub fn key(code: KeyCode) -> Event {
        // Event::Key(KeyEvent::new(code, KeyModifiers::empty()))
        key_with_modifiers(code, KeyModifiers::empty())
    }

    pub fn key_with_modifiers(code: KeyCode, modifiers: KeyModifiers) -> Event {
        Event::Key(KeyEvent::new(code, modifiers))
    }

    // common key combinations
    pub fn ctrl_c() -> Event {
        key_with_modifiers(KeyCode::Char('c'), KeyModifiers::CONTROL)
    }
    pub fn ctrl_d() -> Event {
        key_with_modifiers(KeyCode::Char('d'), KeyModifiers::CONTROL)
    }
    pub fn ctrl_k() -> Event {
        key_with_modifiers(KeyCode::Char('k'), KeyModifiers::CONTROL)
    }
    pub fn ctrl_z() -> Event {
        key_with_modifiers(KeyCode::Char('z'), KeyModifiers::CONTROL)
    }
    pub fn tab() -> Event {
        key(KeyCode::Tab)
    }
    pub fn enter() -> Event {
        key(KeyCode::Enter)
    }
    pub fn escape() -> Event {
        key(KeyCode::Esc)
    }

    // navigation
    pub fn down() -> Event {
        key(KeyCode::Down)
    }
    pub fn up() -> Event {
        key(KeyCode::Up)
    }
    pub fn left() -> Event {
        key(KeyCode::Left)
    }
    pub fn right() -> Event {
        key(KeyCode::Right)
    }
    pub fn page_down() -> Event {
        key(KeyCode::PageDown)
    }
    pub fn page_up() -> Event {
        key(KeyCode::PageUp)
    }
    pub fn home() -> Event {
        key(KeyCode::Home)
    }
    pub fn end() -> Event {
        key(KeyCode::End)
    }

    // charcter keys
    pub fn char(c: char) -> Event {
        key(KeyCode::Char(c))
    }
}
