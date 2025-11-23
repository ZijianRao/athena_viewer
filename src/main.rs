use std::time::{Duration, Instant};
use std::{
    env,
    fs::{self},
    io::{self},
};

use ratatui::symbols::scrollbar;
use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
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
    file_opened: Option<PathBuf>,
    file_text: String,
    vertical_scroll_state: ScrollbarState,
    horizontal_scroll_state: ScrollbarState,
    vertical_scroll: usize,
    horizontal_scroll: usize,
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
        let tick_rate = Duration::from_millis(250);
        let mut last_tick = Instant::now();

        loop {
            terminal.draw(|frame| self.draw(frame))?;
            let timeout = tick_rate.saturating_sub(last_tick.elapsed());

            if event::poll(timeout)? {
                let event = event::read()?;
                if let Event::Key(key_event) = event {
                    match self.input_mode {
                        InputMode::FileView => match key_event.code {
                            KeyCode::Char('q') => {
                                self.message_holder.reset();
                                self.input_mode = InputMode::FileSearch;
                            }
                            KeyCode::Char('j') | KeyCode::Down => {
                                self.message_holder.vertical_scroll =
                                    self.message_holder.vertical_scroll.saturating_add(1);
                                self.message_holder.vertical_scroll_state = self
                                    .message_holder
                                    .vertical_scroll_state
                                    .position(self.message_holder.vertical_scroll);
                            }
                            KeyCode::Char('k') | KeyCode::Up => {
                                self.message_holder.vertical_scroll =
                                    self.message_holder.vertical_scroll.saturating_sub(1);
                                self.message_holder.vertical_scroll_state = self
                                    .message_holder
                                    .vertical_scroll_state
                                    .position(self.message_holder.vertical_scroll);
                            }
                            KeyCode::Char('h') | KeyCode::Left => {
                                self.message_holder.horizontal_scroll =
                                    self.message_holder.horizontal_scroll.saturating_sub(1);
                                self.message_holder.horizontal_scroll_state = self
                                    .message_holder
                                    .horizontal_scroll_state
                                    .position(self.message_holder.horizontal_scroll);
                            }
                            KeyCode::Char('l') | KeyCode::Right => {
                                self.message_holder.horizontal_scroll =
                                    self.message_holder.horizontal_scroll.saturating_add(1);
                                self.message_holder.horizontal_scroll_state = self
                                    .message_holder
                                    .horizontal_scroll_state
                                    .position(self.message_holder.horizontal_scroll);
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
                                if self.message_holder.file_opened.is_some() {
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
            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
            }
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
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
                    "Use h j k l or ◄ ▲ ▼ ► to scroll ".bold(),
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
        self.file_opened = None;
        self.file_text.clear();
        self.setup();
    }

    fn setup(&mut self) {
        if self.current_directory.is_empty() {
            self.current_directory = env::current_dir().unwrap().to_string_lossy().into_owned();
        }
        // let current_directory = String::from("/");
        self.messages = self.get_directory_files(&self.current_directory);
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
            self.messages = self.get_directory_files(&new_entrypoint);
            self.current_directory = new_entrypoint;
            self.input = String::new();
        } else {
            match fs::read_to_string(&new_entrypoint_path) {
                Ok(text) => self.file_text = text,
                Err(_) => self.file_text = "Unable to read...".to_string(),
            }
            self.file_opened = Some(new_entrypoint_path);
        }
    }

    fn get_directory_files(&self, path: &str) -> Vec<FileHolder> {
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

    fn draw(&mut self, area: Rect, frame: &mut Frame) {
        match self.file_opened.clone() {
            None => self.draw_file_view_search(area, frame),
            Some(file_path) => {
                self.draw_file_view(area, frame, &file_path);
            }
        }
    }

    fn draw_file_view_search(&mut self, area: Rect, frame: &mut Frame) {
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

    fn get_string_dimensions(&self, text: &str) -> (usize, usize) {
        let lines: Vec<&str> = text.split('\n').collect();
        let num_rows = lines.len();
        let max_line_length = lines.iter().map(|line| line.len()).max().unwrap_or(0);
        (num_rows, max_line_length)
    }

    fn draw_file_view(&mut self, area: Rect, frame: &mut Frame, file_path: &PathBuf) {
        let file_preview = Paragraph::new(self.file_text.clone())
            .block(Block::bordered().title(file_path.to_string_lossy().into_owned()))
            .scroll((self.vertical_scroll as u16, self.horizontal_scroll as u16));

        let (n_rows, max_line_length) = self.get_string_dimensions(&self.file_text);
        self.vertical_scroll_state = self.vertical_scroll_state.content_length(n_rows);
        self.horizontal_scroll_state = self.horizontal_scroll_state.content_length(max_line_length);

        frame.render_widget(file_preview, area);
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight).symbols(scrollbar::VERTICAL),
            area.inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut self.vertical_scroll_state,
        );
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::HorizontalBottom).symbols(scrollbar::HORIZONTAL),
            area.inner(Margin {
                vertical: 0,
                horizontal: 1,
            }),
            &mut self.horizontal_scroll_state,
        );
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
