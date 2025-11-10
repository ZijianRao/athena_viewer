use std::{env, fs, io};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    text::{Line, Text},
    widgets::{Block, List, ListItem, Paragraph},
    DefaultTerminal, Frame,
};

#[derive(Debug, Default)]
pub struct App {
    exit: bool,
    list_enabled: bool,
}
fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal);
    ratatui::restore();
    app_result
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let vertical = Layout::vertical([Constraint::Min(1), Constraint::Length(1)]);
        let [messages_area, help_area] = vertical.areas(frame.area());
        self.draw_help_area(help_area, frame);
        self.draw_messages_area(messages_area, frame);
    }

    fn draw_help_area(&self, area: Rect, frame: &mut Frame) {
        let instructions = Text::from(Line::from(vec![
            " List Folder ".into(),
            "<Right>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]));
        let help_message = Paragraph::new(instructions);
        frame.render_widget(help_message, area);
    }
    fn draw_messages_area(&self, area: Rect, frame: &mut Frame) {
        let instructions: Paragraph;
        if self.list_enabled {
            let current_dir = env::current_dir().unwrap();
            let entries = fs::read_dir(&current_dir).unwrap();

            let mut path_holder: Vec<ListItem> = Vec::new();
            for entry in entries {
                let entry = entry.unwrap();
                let path = entry.path();
                let list_item = ListItem::new(Line::from(path.to_string_lossy().into_owned()));
                path_holder.push(list_item);
            }
            let messages = List::new(path_holder)
                .block(Block::bordered().title(current_dir.to_string_lossy().into_owned()));
            frame.render_widget(messages, area);
        } else {
            instructions = Paragraph::new(Text::from(Line::from(vec![
                " List Folder ".into(),
                "<Right>".blue().bold(),
                " Quit ".into(),
                "<Q> ".blue().bold(),
            ])))
            .block(Block::bordered().title("Viewer"));
            frame.render_widget(instructions, area);
        }
    }
    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_events(key_event)
            }
            _ => {}
        }
        Ok(())
    }
    fn handle_key_events(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Right => self.list_folder(),
            _ => {}
        }
    }
    fn exit(&mut self) {
        self.exit = true;
    }

    fn list_folder(&mut self) {
        self.list_enabled = true;
    }
}
