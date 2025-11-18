use std::{env, fs, io};

use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, List, ListItem, Paragraph},
    DefaultTerminal, Frame,
};
use std::path::PathBuf;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

#[derive(Debug, Default)]
struct App {
    input_mode: InputMode,
    input: Input,
    message_holder: MessageHolder,
}

#[derive(Debug, Default)]
struct MessageHolder {
    messages: Vec<FileHolder>,
    current_directory: String,
    input: String,
}

#[derive(Debug)]
struct FileHolder {
    file_name: String,
    is_file: bool,
    parent_folder: PathBuf,
}

#[derive(Debug, Default, PartialEq)]
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
        loop {
            terminal.draw(|frame| self.draw(frame))?;
            let event = event::read()?;
            if let Event::Key(key_event) = event {
                match self.input_mode {
                    InputMode::Normal => match key_event.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Tab => {
                            self.input_mode = InputMode::Editing;
                            self.message_holder.setup();
                        }
                        _ => {}
                    },
                    InputMode::Editing => match key_event.code {
                        KeyCode::Tab => self.input_mode = InputMode::Normal,
                        KeyCode::Enter => {
                            self.message_holder.submit();
                            self.input.reset();
                        }
                        _ => {
                            self.input.handle_event(&event);
                            self.message_holder.update(self.input.value());
                        }
                    },
                }
            }
        }
    }

    fn draw(&self, frame: &mut Frame) {
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

    fn draw_input_area(&self, area: Rect, frame: &mut Frame) {
        // keep 2 for boarders and 1 for cursor
        let width = area.width.max(3) - 3;
        let scroll = self.input.visual_scroll(width as usize);
        let style = match self.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Color::Yellow.into(),
        };

        let input = Paragraph::new(self.input.value())
            .style(style)
            .scroll((0, scroll as u16))
            .block(Block::bordered().title("Input"));
        frame.render_widget(input, area);

        // https://github.com/sayanarijit/tui-input/blob/main/examples/ratatui_crossterm_input.rs
        if self.input_mode == InputMode::Editing {
            let x = self.input.visual_cursor().max(scroll) - scroll + 1;
            frame.set_cursor_position((area.x + x as u16, area.y + 1));
        }
    }
}

impl From<PathBuf> for FileHolder {
    fn from(path: PathBuf) -> Self {
        let file_name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap();

        FileHolder {
            file_name: file_name,
            is_file: path.is_file(),
            parent_folder: path.parent().unwrap().to_path_buf(),
        }
    }
}

impl MessageHolder {
    fn update(&mut self, input: &str) {
        self.input = input.to_string();
    }

    fn setup(&mut self) {
        let current_directory = env::current_dir().unwrap().to_string_lossy().into_owned();
        // let current_directory = String::from("/");
        self.messages = self.get_child_filename_group(&current_directory);
        self.current_directory = current_directory;
    }

    fn submit(&mut self) {
        // assume a new folder will be opened
        // self.input = input.to_string();
        let mut path_holder: Vec<FileHolder> = std::mem::take(&mut self.messages)
            .into_iter()
            .filter(|entry| self.should_select(&entry.file_name))
            .collect();
        assert_eq!(path_holder.len(), 1);

        let filename = path_holder.pop().unwrap().file_name;
        let new_current_directory = format!("{}/{}", self.current_directory, filename);
        self.messages = self.get_child_filename_group(&new_current_directory);
        self.current_directory = new_current_directory;
        self.input = String::new();
    }

    fn get_child_filename_group(&self, path: &str) -> Vec<FileHolder> {
        fs::read_dir(&PathBuf::from(path))
            .unwrap()
            .filter_map(|entry| entry.ok().map(|e| FileHolder::from(e.path())))
            .collect()
    }

    fn draw(&self, area: Rect, frame: &mut Frame) {
        let path_holder: Vec<ListItem> = self
            .messages
            .iter()
            .filter(|entry| self.should_select(&entry.file_name))
            .map(|entry| {
                ListItem::new(Line::from(entry.file_name.clone()).style(if entry.is_file {
                    Style::default()
                } else {
                    Color::LightCyan.into()
                }))
            })
            .collect();
        let messages =
            List::new(path_holder).block(Block::bordered().title(self.current_directory.clone()));
        frame.render_widget(messages, area);
    }

    fn should_select(&self, name: &str) -> bool {
        if self.input.is_empty() {
            return true;
        }

        let mut counter = 0;
        for char in name.chars() {
            if char.eq_ignore_ascii_case(
                &self
                    .input
                    .chars()
                    .nth(counter)
                    .expect("Should not reach out of bounds"),
            ) {
                counter += 1;
            }
            if counter == self.input.len() {
                return true;
            }
        }

        false
    }
}
