use std::env;

use crate::message_holder::code_highlighter::CodeHighlighter;
use crate::message_holder::file_helper::{FileGroupHolder, FileHolder, FileTextInfo};
use lru::LruCache;
use ratatui::style::Stylize;
use ratatui::symbols::scrollbar;
use ratatui::{
    layout::{Margin, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};
use std::num::NonZeroUsize;
use std::path::PathBuf;

#[derive(Debug)]
pub struct MessageHolder {
    cache_holder: LruCache<PathBuf, FileGroupHolder>,
    current_directory: PathBuf,
    input: String,
    code_highlighter: CodeHighlighter,
    pub highlight_index: usize,
    pub file_opened: Option<PathBuf>,
    pub file_text_info: Option<FileTextInfo>,
    pub vertical_scroll_state: ScrollbarState,
    pub horizontal_scroll_state: ScrollbarState,
    pub vertical_scroll: usize,
    pub horizontal_scroll: usize,
}

impl Default for MessageHolder {
    fn default() -> Self {
        Self {
            cache_holder: LruCache::new(NonZeroUsize::new(100).unwrap()),
            current_directory: Default::default(),
            input: Default::default(),
            code_highlighter: CodeHighlighter::new(),
            highlight_index: Default::default(),
            file_opened: Default::default(),
            file_text_info: Default::default(),
            vertical_scroll_state: Default::default(),
            horizontal_scroll_state: Default::default(),
            vertical_scroll: Default::default(),
            horizontal_scroll: Default::default(),
        }
    }
}
impl MessageHolder {
    pub fn update(&mut self, input: &str) {
        self.input = input.to_string();
    }

    pub fn refresh_current_folder_cache(&mut self) {
        let holder = FileGroupHolder::from(self.current_directory.clone());
        self.cache_holder
            .put(self.current_directory.clone(), holder);
    }

    pub fn reset(&mut self) {
        self.input.clear();
        self.file_opened = None;
        self.file_text_info = None;
        self.setup();
    }

    pub fn setup(&mut self) {
        if self.current_directory.as_os_str().is_empty() {
            self.current_directory = env::current_dir().unwrap();
        }

        let holder = FileGroupHolder::from(self.current_directory.clone());
        self.cache_holder
            .put(self.current_directory.clone(), holder);
    }

    pub fn submit(&mut self) {
        let mut messages = self
            .cache_holder
            .get(&self.current_directory)
            .unwrap()
            .child
            .clone();
        let path_holder: Vec<FileHolder> = std::mem::take(&mut messages)
            .into_iter()
            .filter(|entry| self.should_select(&entry.file_name))
            .collect();
        assert!(!path_holder.is_empty());

        let filename = &path_holder[self.highlight_index].file_name;
        let new_entrypoint_canonicalized_result =
            self.current_directory.join(filename).canonicalize();
        match new_entrypoint_canonicalized_result {
            Ok(new_entrypoint) => {
                if new_entrypoint.is_dir() {
                    self.current_directory = new_entrypoint;
                    if self.cache_holder.get(&self.current_directory).is_none() {
                        let holder = FileGroupHolder::from(self.current_directory.clone());
                        self.cache_holder
                            .put(self.current_directory.clone(), holder);
                    }
                    self.input = String::new();
                } else {
                    self.file_text_info =
                        Some(FileTextInfo::new(&new_entrypoint, &self.code_highlighter));
                    self.file_opened = Some(new_entrypoint);
                }
            }
            Err(_) => {
                self.setup();
            }
        }
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame) {
        match self.file_opened.clone() {
            None => {
                if !self.current_directory.as_os_str().is_empty() {
                    self.draw_file_view_search(area, frame);
                }
            }
            Some(file_path) => {
                self.draw_file_view(area, frame, &file_path);
            }
        }
    }

    fn draw_file_view_search(&mut self, area: Rect, frame: &mut Frame) {
        let current_file_holder = self.cache_holder.peek(&self.current_directory).unwrap();

        let mut path_holder: Vec<ListItem> = current_file_holder
            .child
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

        if !path_holder.is_empty() {
            if path_holder.len() <= self.highlight_index {
                self.highlight_index = path_holder.len() - 1;
            }
        }
        if let Some(path) = path_holder.get_mut(self.highlight_index) {
            *path = path.clone().add_modifier(Modifier::REVERSED);
        };

        let title = format!(
            "{} {}",
            self.current_directory.display(),
            current_file_holder.update_time.format("%Y-%m-%d %H:%M:%S")
        );
        let messages = List::new(path_holder).block(Block::default().title(title));
        frame.render_widget(messages, area);
    }

    fn draw_file_view(&mut self, area: Rect, frame: &mut Frame, file_path: &PathBuf) {
        let file_text_info = self.file_text_info.as_ref().unwrap();
        let file_preview = Paragraph::new(file_text_info.formatted_text.clone())
            .block(Block::default().title(file_path.to_string_lossy().into_owned()))
            .scroll((self.vertical_scroll as u16, self.horizontal_scroll as u16));

        self.vertical_scroll_state = self
            .vertical_scroll_state
            .content_length(file_text_info.n_rows);
        self.horizontal_scroll_state = self
            .horizontal_scroll_state
            .content_length(file_text_info.max_line_length);

        frame.render_widget(file_preview, area);
        // frame.render_stateful_widget(
        //     Scrollbar::new(ScrollbarOrientation::VerticalRight).symbols(scrollbar::VERTICAL),
        //     area.inner(Margin {
        //         vertical: 1,
        //         horizontal: 0,
        //     }),
        //     &mut self.vertical_scroll_state,
        // );
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
