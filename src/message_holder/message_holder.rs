use std::{
    env,
    fs::{self},
};

use chrono::{DateTime, Local};
use ratatui::symbols::scrollbar;
use ratatui::{
    layout::{Margin, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

use std::path::PathBuf;

#[derive(Debug)]
pub struct FileTextInfo {
    text: String,
    pub n_rows: usize,
    pub max_line_length: usize,
}

#[derive(Debug, Default)]
pub struct MessageHolder {
    messages: Vec<FileHolder>,
    current_directory: String,
    input: String,
    last_refresh_time: DateTime<Local>,
    pub file_opened: Option<PathBuf>,
    pub file_text_info: Option<FileTextInfo>,
    pub vertical_scroll_state: ScrollbarState,
    pub horizontal_scroll_state: ScrollbarState,
    pub vertical_scroll: usize,
    pub horizontal_scroll: usize,
}

#[derive(Debug)]
struct FileHolder {
    file_name: String,
    is_file: bool,
    parent_folder: PathBuf,
}

impl FileTextInfo {
    fn new(value: &PathBuf) -> Self {
        let content = match fs::read_to_string(value) {
            Ok(text) => text,
            Err(_) => "Unable to read...".to_string(),
        };

        let (num_rows, max_line_length) = Self::get_string_dimensions(&content);

        Self {
            text: content,
            n_rows: num_rows,
            max_line_length: max_line_length,
        }
    }

    fn get_string_dimensions(text: &str) -> (usize, usize) {
        let lines: Vec<&str> = text.split('\n').collect();
        let num_rows = lines.len();
        let max_line_length = lines.iter().map(|line| line.len()).max().unwrap_or(0);
        (num_rows, max_line_length)
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
    pub fn update(&mut self, input: &str) {
        self.input = input.to_string();
    }

    pub fn reset(&mut self) {
        self.input.clear();
        self.file_opened = None;
        self.file_text_info = None;
        self.setup();
    }

    pub fn setup(&mut self) {
        if self.current_directory.is_empty() {
            self.current_directory = env::current_dir().unwrap().to_string_lossy().into_owned();
        }
        // let current_directory = String::from("/");
        self.messages = self.get_directory_files(&self.current_directory);
        self.last_refresh_time = Local::now();
    }

    pub fn submit(&mut self) {
        let path_holder: Vec<FileHolder> = std::mem::take(&mut self.messages)
            .into_iter()
            .filter(|entry| self.should_select(&entry.file_name))
            .collect();
        assert!(!path_holder.is_empty());

        let filename = &path_holder[0].file_name;
        let new_entrypoint_raw = format!("{}/{}", self.current_directory, filename);
        let new_entrypoint_path = PathBuf::from(new_entrypoint_raw).canonicalize().unwrap();
        if new_entrypoint_path.is_dir() {
            let new_entrypoint = new_entrypoint_path.to_string_lossy().into_owned();
            self.messages = self.get_directory_files(&new_entrypoint);
            self.current_directory = new_entrypoint;
            self.input = String::new();
            self.last_refresh_time = Local::now();
        } else {
            self.file_text_info = Some(FileTextInfo::new(&new_entrypoint_path));
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

    pub fn draw(&mut self, area: Rect, frame: &mut Frame) {
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

        let title = format!(
            "{} {}",
            self.current_directory,
            self.last_refresh_time.format("%Y-%m-%d %H:%M")
        );
        let messages = List::new(path_holder).block(Block::bordered().title(title));
        frame.render_widget(messages, area);
    }

    fn draw_file_view(&mut self, area: Rect, frame: &mut Frame, file_path: &PathBuf) {
        let file_text_info = self.file_text_info.as_ref().unwrap();
        let file_preview = Paragraph::new(file_text_info.text.clone())
            .block(Block::bordered().title(file_path.to_string_lossy().into_owned()))
            .scroll((self.vertical_scroll as u16, self.horizontal_scroll as u16));

        self.vertical_scroll_state = self
            .vertical_scroll_state
            .content_length(file_text_info.n_rows);
        self.horizontal_scroll_state = self
            .horizontal_scroll_state
            .content_length(file_text_info.max_line_length);

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
