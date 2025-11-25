use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Paragraph},
    Frame,
};

use super::App;
use super::InputMode;

impl App {
    pub fn draw(&mut self, frame: &mut Frame) {
        let vertical = Layout::vertical([
            Constraint::Min(1),
            Constraint::Length(3),
            Constraint::Length(1),
        ]);

        let [messages_area, input_area, help_area] = vertical.areas(frame.area());
        self.draw_help_area(help_area, frame);
        self.draw_input_area(input_area, frame);
        self.message_holder.draw(messages_area, frame);
    }

    pub fn draw_help_area(&self, area: Rect, frame: &mut Frame) {
        let instructions: Text;
        match self.input_mode {
            InputMode::Normal => {
                instructions = Text::from(Line::from(vec![
                    " Normal ".bold(),
                    " Switch Mode ".into(),
                    "<Tab>".blue().bold(),
                    " Quit ".into(),
                    "<Q>".blue().bold(),
                ]));
            }
            InputMode::FileSearch => {
                instructions = Text::from(Line::from(vec![
                    " FileSearch ".bold(),
                    " Switch Mode ".into(),
                    "<Tab>".blue().bold(),
                ]));
            }
            InputMode::FileView => {
                instructions = Text::from(Line::from(vec![
                    " FileView".bold(),
                    "Use h j k l or ◄ ▲ ▼ ► to scroll ".bold(),
                    " Quit ".into(),
                    "<Q>".blue().bold(),
                ]));
            }
        }
        let help_message = Paragraph::new(instructions);
        frame.render_widget(help_message, area);
    }

    pub fn draw_input_area(&self, area: Rect, frame: &mut Frame) {
        // keep 2 for boarders and 1 for cursor
        let width = area.width.max(3) - 3;
        let scroll = self.input.visual_scroll(width as usize);
        let style = match self.input_mode {
            InputMode::FileSearch => Color::Yellow.into(),
            _ => Style::default(),
        };

        let input = Paragraph::new(self.input.value())
            .style(style)
            .scroll((0, scroll as u16))
            .block(Block::bordered().title("Input"));
        frame.render_widget(input, area);

        // https://github.com/sayanarijit/tui-input/blob/main/examples/ratatui_crossterm_input.rs
        if self.input_mode == InputMode::FileSearch {
            let x = self.input.visual_cursor().max(scroll) - scroll + 1;
            frame.set_cursor_position((area.x + x as u16, area.y + 1));
        }
    }
}
