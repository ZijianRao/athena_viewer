use ratatui::style::Stylize;
use ratatui::symbols::scrollbar;
use ratatui::{
    layout::{Margin, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::message_holder::code_highlighter::CodeHighlighter;
use crate::message_holder::file_helper::{FileHolder, FileTextInfo};
use crate::message_holder::folder_holder::FolderHolder;
use crate::state_holder::state_holder::StateHolder;

#[derive(Debug)]
pub struct MessageHolder {
    state_holder: Rc<RefCell<StateHolder>>,
    folder_holder: FolderHolder,
    code_highlighter: CodeHighlighter,
    pub raw_highlight_index: i32,
    pub file_opened: Option<PathBuf>,
    pub file_text_info: Option<FileTextInfo>,
    pub vertical_scroll_state: ScrollbarState,
    pub horizontal_scroll_state: ScrollbarState,
    pub vertical_scroll: usize,
    pub horizontal_scroll: usize,
}

impl MessageHolder {
    pub fn new(state_holder: Rc<RefCell<StateHolder>>) -> Self {
        let state_holder_ref = Rc::clone(&state_holder);
        MessageHolder {
            state_holder,
            code_highlighter: CodeHighlighter::new(),
            folder_holder: FolderHolder::new(state_holder_ref),
            raw_highlight_index: 0,
            file_opened: Default::default(),
            file_text_info: Default::default(),
            vertical_scroll_state: Default::default(),
            horizontal_scroll_state: Default::default(),
            vertical_scroll: Default::default(),
            horizontal_scroll: Default::default(),
        }
    }

    pub fn reset_index(&mut self) {
        self.raw_highlight_index = 0;
    }
    pub fn move_up(&mut self) {
        self.raw_highlight_index = self.raw_highlight_index.saturating_sub(1);
    }
    pub fn move_down(&mut self) {
        self.raw_highlight_index = self.raw_highlight_index.saturating_add(1);
    }
    pub fn update(&mut self, input: &str) {
        self.folder_holder.update(input);
    }

    pub fn refresh_current_folder_cache(&mut self) {
        self.folder_holder.refresh();
    }

    pub fn reset(&mut self) {
        self.folder_holder.update("");
        self.file_opened = None;
        self.file_text_info = None;
        self.reset_index();
    }

    fn get_highlight_index(&self, group_len: usize) -> usize {
        let divisor: i32 = group_len
            .try_into()
            .expect("Cannot convert group len of path_group");
        let remainder = self.raw_highlight_index.rem_euclid(divisor);
        remainder.try_into().expect("Unexpected!")
    }

    pub fn submit(&mut self) {
        let path_holder = &self.folder_holder.selected_path_holder;
        if path_holder.is_empty() {
            return;
        }

        let highlight_index = self.get_highlight_index(path_holder.len());
        let new_entrypoint_canonicalized_result = self.folder_holder.submit(highlight_index);
        match new_entrypoint_canonicalized_result {
            Ok(new_entrypoint) => {
                if new_entrypoint.is_dir() {
                    self.folder_holder
                        .submit_new_working_directory(new_entrypoint);
                } else {
                    self.file_text_info =
                        Some(FileTextInfo::new(&new_entrypoint, &self.code_highlighter));
                    self.file_opened = Some(new_entrypoint);
                    self.state_holder.borrow_mut().to_file_view();
                }
            }
            Err(_) => {}
        }
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame) {
        match self.file_opened.clone() {
            None => self.draw_folder_view(area, frame),
            Some(file_path) => self.draw_file_view(area, frame, &file_path),
        }
    }

    fn draw_folder_view(&mut self, area: Rect, frame: &mut Frame) {
        let mut path_holder: Vec<ListItem> = self
            .folder_holder
            .selected_path_holder
            .iter()
            .map(|entry| {
                ListItem::new(Line::from(self.get_text(entry)).style(if entry.is_file {
                    Style::default()
                } else {
                    Color::LightCyan.into()
                }))
            })
            .collect();
        if path_holder.is_empty() {
            return;
        }

        let highlight_index = self.get_highlight_index(path_holder.len());
        if let Some(path) = path_holder.get_mut(highlight_index) {
            *path = path.clone().add_modifier(Modifier::REVERSED);
        };

        let title;
        if self.state_holder.borrow().is_history_search() {
            title = format!("History: {} items", path_holder.len());
        } else {
            title = format!(
                "{} {}",
                self.folder_holder.current_directory.display(),
                self.folder_holder
                    .peek()
                    .update_time
                    .format("%Y-%m-%d %H:%M:%S")
            );
        }

        let messages = List::new(path_holder).block(Block::default().title(title));
        frame.render_widget(messages, area);
    }

    fn get_text(&self, entry: &FileHolder) -> String {
        if self.state_holder.borrow().is_history_search() {
            entry
                .to_path()
                .expect("Unable to get history item")
                .to_string_lossy()
                .into_owned()
        } else {
            entry.file_name.clone()
        }
    }

    fn draw_file_view(&mut self, area: Rect, frame: &mut Frame, file_path: &PathBuf) {
        let file_text_info = self
            .file_text_info
            .as_ref()
            .expect("Unable to get text file info!");
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

        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::HorizontalBottom).symbols(scrollbar::HORIZONTAL),
            area.inner(Margin {
                vertical: 0,
                horizontal: 1,
            }),
            &mut self.horizontal_scroll_state,
        );
    }
}
