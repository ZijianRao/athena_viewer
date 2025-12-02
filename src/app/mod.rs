use std::io::{self};
use std::time::{Duration, Instant};

use ratatui::DefaultTerminal;
use tui_input::Input;

use crate::message_holder;

#[derive(Debug, Default)]
pub struct App {
    input_mode: InputMode,
    input: Input,
    exit: bool,
    message_holder: message_holder::message_holder::MessageHolder,
}

#[derive(Debug, Default, PartialEq)]
enum InputMode {
    #[default]
    Normal,
    FileSearch,
    FileView,
    FileSearchHistory,
}

pub mod draw;
pub mod handle_event;
impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let tick_rate = Duration::from_millis(250);
        let mut last_tick = Instant::now();

        loop {
            terminal.draw(|frame| self.draw(frame))?;
            match self.input_mode {
                InputMode::FileView => self.handle_file_view_event(&mut last_tick, &tick_rate),
                InputMode::Normal => self.handle_normal_event(),
                InputMode::FileSearch => self.handle_file_search_event(),
                InputMode::FileSearchHistory => self.handle_file_search_history_event(),
            }
            if self.exit {
                return Ok(());
            }
        }
    }
}
