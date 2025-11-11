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
struct App {
    exit: bool,
    list_enabled: bool,
    input_mode: InputMode,
    character_index: usize,
    input: String,
}

#[derive(Debug, Default)]
enum InputMode {
    #[default]
    Normal,
    Editing,
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
        let vertical = Layout::vertical([
            Constraint::Min(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ]);
        let [messages_area, input_area, help_area] = vertical.areas(frame.area());
        self.draw_help_area(help_area, frame);
        self.draw_messages_area(messages_area, frame);
    }

    fn draw_help_area(&self, area: Rect, frame: &mut Frame) {
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
            InputMode::Editing => {
                instructions = Text::from(Line::from(vec![
                    " Editing ".bold(),
                    " Switch Mode ".into(),
                    "<Tab>".blue().bold(),
                ]));
            }
        }
        let help_message = Paragraph::new(instructions);
        frame.render_widget(help_message, area);
    }
    fn draw_messages_area(&self, area: Rect, frame: &mut Frame) {
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
        match self.input_mode {
            InputMode::Normal => match key_event.code {
                KeyCode::Char('q') => self.exit(),
                KeyCode::Tab => self.input_mode = InputMode::Editing,
                _ => {}
            },
            InputMode::Editing => match key_event.code {
                KeyCode::Tab => self.input_mode = InputMode::Normal,
                _ => {
                    self.list_folder();
                }
            },
        }
    }
    fn exit(&mut self) {
        self.exit = true;
    }

    fn list_folder(&mut self) {
        self.list_enabled = true;
    }
}
