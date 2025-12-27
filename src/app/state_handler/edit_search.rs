use ratatui::crossterm::event::{Event, KeyCode, KeyModifiers};
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
    pub fn handle_edit_search_event(&mut self, event: Event) -> AppResult<()> {
        if let Event::Key(key_event) = event {
            match key_event.code {
                KeyCode::Tab => self.state_holder.borrow_mut().to_search(),
                KeyCode::Up => self.message_holder.move_up(),
                KeyCode::Down => self.message_holder.move_down(),
                KeyCode::Enter => {
                    self.message_holder.submit()?;
                    self.input.reset();
                }
                _ => {
                    if (key_event.code == KeyCode::Char('c'))
                        & key_event.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        self.input.reset();
                        self.message_holder.update(None);
                    } else {
                        self.input.handle_event(&event);
                        self.message_holder
                            .update(Some(self.input.value().to_string()));
                    }
                }
            }
        }
        Ok(())
    }
    pub fn draw_edit_search(&mut self, help_area: Rect, frame: &mut Frame) {
        let instructions = Text::from(Line::from(vec![
            "FileSearch ".bold(),
            "Switch to".into(),
            " Normal ".bold(),
            "<Tab>".light_blue().bold(),
            " Clear ".bold(),
            "<CTRL+C>".light_blue().bold(),
        ]));

        let help_message = Paragraph::new(instructions);
        frame.render_widget(help_message, help_area);
    }
}
