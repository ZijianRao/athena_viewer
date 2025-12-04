use ratatui::crossterm::event::{self, Event, KeyCode};

use crate::app::App;
use crate::state_holder::state_holder::{InputMode, ViewMode};
use tui_input::backend::crossterm::EventHandler;

impl App {
    pub fn handle_edit_search_event(&mut self) {
        let event = event::read().unwrap();
        if let Event::Key(key_event) = event {
            match key_event.code {
                KeyCode::Tab => self.state_holder.input_mode = InputMode::Normal,
                KeyCode::Up => {
                    self.message_holder.highlight_index =
                        self.message_holder.highlight_index.saturating_sub(1);
                }
                KeyCode::Down => {
                    self.message_holder.highlight_index =
                        self.message_holder.highlight_index.saturating_add(1);
                }
                KeyCode::Enter => {
                    self.message_holder.submit();
                    if self.message_holder.file_opened.is_some() {
                        self.state_holder.view_mode = ViewMode::FileView;
                    }
                    self.input.reset();
                }
                _ => {
                    self.input.handle_event(&event);
                    self.message_holder.highlight_index = 0;
                    self.message_holder.update(self.input.value());
                }
            }
        }
    }
}
