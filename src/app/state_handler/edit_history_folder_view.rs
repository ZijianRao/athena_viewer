use ratatui::crossterm::event::{self, Event, KeyCode};
use ratatui::{
    layout::Rect,
    style::Stylize,
    text::{Line, Text},
    widgets::Paragraph,
    Frame,
};
use tui_input::backend::crossterm::EventHandler;

use crate::app::App;

impl App {
    pub fn handle_edit_history_folder_view_event(&mut self) {
        let event = event::read().unwrap();
        if let Event::Key(key_event) = event {
            match key_event.code {
                KeyCode::Tab => {
                    self.state_holder.to_search();
                    self.message_holder.view_history = false;
                }
                KeyCode::Up => {
                    self.message_holder.highlight_index =
                        self.message_holder.highlight_index.saturating_sub(1);
                }
                KeyCode::Down => {
                    self.message_holder.highlight_index =
                        self.message_holder.highlight_index.saturating_add(1);
                }
                KeyCode::Enter => {
                    self.message_holder.submit_for_history();
                    if self.message_holder.file_opened.is_some() {
                        self.state_holder.to_file_view();
                    }
                    self.input.reset();
                    self.state_holder.to_search();
                    self.message_holder.view_history = false;
                }
                _ => {
                    self.input.handle_event(&event);
                    self.message_holder.highlight_index = 0;
                    self.message_holder.update(self.input.value());
                }
            }
        }
    }
    pub fn draw_help_edit_history_folder_view(&mut self, help_area: Rect, frame: &mut Frame) {
        let instructions = Text::from(Line::from(vec![
            "FileSearchHistory".bold(),
            " Switch to".into(),
            " FileSearch".bold(),
            "<Tab>".light_blue().bold(),
        ]));
        let help_message = Paragraph::new(instructions);
        frame.render_widget(help_message, help_area);
    }
}
