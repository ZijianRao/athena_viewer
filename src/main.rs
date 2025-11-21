use std::{
    env,
    fs::{self, File},
    io::{self, BufRead, BufReader},
};

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
    file_to_read: Option<PathBuf>,
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
    FileSearch,
    FileView,
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
                    InputMode::FileView => match key_event.code {
                        KeyCode::Char('q') => {
                            self.message_holder.reset();
                            self.input_mode = InputMode::FileSearch;
                        }
                        _ => (),
                    },
                    InputMode::Normal => match key_event.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Tab => {
                            self.input_mode = InputMode::FileSearch;
                            self.message_holder.setup();
                        }
                        _ => {}
                    },
                    InputMode::FileSearch => match key_event.code {
                        KeyCode::Tab => self.input_mode = InputMode::Normal,
                        KeyCode::Enter => {
                            self.message_holder.submit();
                            if self.message_holder.file_to_read.is_some() {
                                self.input_mode = InputMode::FileView;
                            }
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
                    " Quit ".into(),
                    "<Q>".blue().bold(),
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

    fn reset(&mut self) {
        self.input.clear();
        self.file_to_read = None;
        self.setup();
    }

    fn setup(&mut self) {
        let current_directory = env::current_dir().unwrap().to_string_lossy().into_owned();
        // let current_directory = String::from("/");
        self.messages = self.get_child_filename_group(&current_directory);
        self.current_directory = current_directory;
    }

    fn submit(&mut self) {
        let path_holder: Vec<FileHolder> = std::mem::take(&mut self.messages)
            .into_iter()
            .filter(|entry| self.should_select(&entry.file_name))
            .collect();
        assert!(!path_holder.is_empty());

        let filename = &path_holder[0].file_name;
        let new_entrypoint = format!("{}/{}", self.current_directory, filename);
        let new_entrypoint_path = PathBuf::from(new_entrypoint.clone());
        if new_entrypoint_path.is_dir() {
            self.messages = self.get_child_filename_group(&new_entrypoint);
            self.current_directory = new_entrypoint;
            self.input = String::new();
        } else {
            self.file_to_read = Some(new_entrypoint_path);
        }
    }

    fn get_child_filename_group(&self, path: &str) -> Vec<FileHolder> {
        let path_buf = PathBuf::from(path);
        let mut entries = Vec::new();

        // add if not at root
        if let Some(parent) = path_buf.parent() {
            entries.push(FileHolder {
                file_name: "..".to_string(),
                is_file: false,
                parent_folder: parent.to_path_buf(), // BUG: incorrect parent of parent
            })
        }

        entries.extend(
            fs::read_dir(&path_buf)
                .unwrap()
                .filter_map(|entry| entry.ok().map(|e| FileHolder::from(e.path()))),
        );

        entries
    }

    fn draw(&self, area: Rect, frame: &mut Frame) {
        match &self.file_to_read {
            None => self.draw_file_view_search(area, frame),
            Some(file_path) => {
                self.draw_file_view(area, frame, file_path);
            }
        }
    }

    fn draw_file_view_search(&self, area: Rect, frame: &mut Frame) {
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

    fn draw_file_view(&self, area: Rect, frame: &mut Frame, file_path: &PathBuf) {
        let horizontal = Layout::horizontal([Constraint::Ratio(1, 2); 2]);
        let [left, right] = horizontal.areas(area);

        let messages = Paragraph::new(file_path.to_string_lossy().into_owned())
            .block(Block::bordered().title(self.current_directory.clone()));

        let file = fs::File::open(file_path).unwrap();
        let reader = BufReader::new(file);

        let lines_result: Result<Vec<String>, _> = reader.lines().take(30).collect();
        let text: String;
        match lines_result {
            Ok(lines) => text = lines.join("\n"),
            Err(_) => text = "Unable to read...".to_string(),
        }
        let file_preview = Paragraph::new(text).block(Block::bordered());

        frame.render_widget(messages, left);
        frame.render_widget(file_preview, right);
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
