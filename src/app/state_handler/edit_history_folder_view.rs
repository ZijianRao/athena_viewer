use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::{
    layout::Rect,
    style::Stylize,
    text::{Line, Text},
    widgets::Paragraph,
    Frame,
};
use tui_input::backend::crossterm::EventHandler;

use crate::app::app_error::AppResult;
use crate::app::App;

impl App {
    pub fn handle_edit_history_folder_view_event(&mut self, event: Event) -> AppResult<()> {
        if let Event::Key(key_event) = event {
            match key_event.code {
                KeyCode::Tab => self.state_holder.borrow_mut().to_search(),
                KeyCode::Up => self.message_holder.move_up(),
                KeyCode::Down => self.message_holder.move_down(),
                KeyCode::Enter => {
                    self.message_holder.submit()?;
                    if !self.state_holder.borrow().is_file_view() {
                        self.input.reset();
                    }
                }

                _ => {
                    self.input.handle_event(&event);
                    self.message_holder
                        .update(Some(self.input.value().to_string()))?;
                }
            }
        }

        Ok(())
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
