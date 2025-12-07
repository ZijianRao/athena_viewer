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
    pub fn handle_edit_search_event(&mut self) {
        let event = event::read().expect("Unable to handle key press event!");
        if let Event::Key(key_event) = event {
            match key_event.code {
                KeyCode::Tab => self.state_holder.borrow_mut().to_search(),
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
    pub fn draw_edit_search(&mut self, help_area: Rect, frame: &mut Frame) {
        let instructions = Text::from(Line::from(vec![
            "FileSearch ".bold(),
            "Switch to".into(),
            " Normal ".bold(),
            "<Tab>".light_blue().bold(),
        ]));

        let help_message = Paragraph::new(instructions);
        frame.render_widget(help_message, help_area);
    }
}
