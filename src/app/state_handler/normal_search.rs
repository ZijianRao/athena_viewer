use ratatui::crossterm::event::{self, Event, KeyCode};
use ratatui::{
    layout::Rect,
    style::Stylize,
    text::{Line, Text},
    widgets::Paragraph,
    Frame,
};

use crate::app::App;

impl App {
    pub fn handle_normal_search_event(&mut self) {
        let event = event::read().expect("Unable to handle key press event!");
        if let Event::Key(key_event) = event {
            match key_event.code {
                KeyCode::Char('q') => self.exit = true,
                KeyCode::Char('u') => self.message_holder.refresh_current_folder_cache(),
                KeyCode::Char('h') => {
                    self.state_holder.borrow_mut().to_history_search();
                    self.message_holder.reset();
                }
                KeyCode::Tab => self.state_holder.borrow_mut().to_search_edit(),
                _ => {}
            }
        }
    }

    pub fn draw_help_normal_search(&mut self, help_area: Rect, frame: &mut Frame) {
        let instructions = Text::from(Line::from(vec![
            "Normal ".bold(),
            "Switch to".into(),
            " FileSearch ".bold(),
            "<Tab>".light_blue().bold(),
            " Quit ".into(),
            "<Q>".light_blue().bold(),
            " Update ".into(),
            "<U>".light_blue().bold(),
            " Switch to ".into(),
            "FileSearchHistory ".bold(),
            "<H>".light_blue().bold(),
        ]));
        let help_message = Paragraph::new(instructions);
        frame.render_widget(help_message, help_area);
    }
}
