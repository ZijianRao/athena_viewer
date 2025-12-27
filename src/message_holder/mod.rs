pub mod code_highlighter;
pub mod file_helper;
pub mod folder_holder;

use ratatui::style::Stylize;
use ratatui::symbols::scrollbar;
use ratatui::{
    layout::{Margin, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};
use std::fs;

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::app::app_error::AppResult;
use crate::message_holder::code_highlighter::CodeHighlighter;
use crate::message_holder::file_helper::{FileHolder, FileTextInfo};
use crate::message_holder::folder_holder::FolderHolder;
use crate::state_holder::StateHolder;

#[derive(Debug)]
pub struct MessageHolder {
    state_holder: Rc<RefCell<StateHolder>>,
    pub folder_holder: FolderHolder,
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
    pub fn new(current_directory: PathBuf, state_holder: Rc<RefCell<StateHolder>>) -> Self {
        let state_holder_ref = Rc::clone(&state_holder);
        MessageHolder {
            state_holder,
            code_highlighter: CodeHighlighter::default(),
            folder_holder: FolderHolder::new(current_directory, state_holder_ref),
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
    pub fn update(&mut self, input: Option<String>) {
        self.folder_holder.update(input);
        self.reset_index();
    }
    pub fn expand(&mut self) {
        self.folder_holder.expand();
    }
    pub fn collapse(&mut self) {
        self.folder_holder.collapse();
    }

    pub fn to_parent(&mut self) -> AppResult<()> {
        self.raw_highlight_index = 0;
        self.update(None);
        self.submit()?;
        Ok(())
    }

    pub fn delete(&mut self) {
        let path_holder = &self.folder_holder.selected_path_holder;
        if path_holder.is_empty() {
            return;
        }

        let highlight_index = self.get_highlight_index(path_holder.len());
        if let Ok(path) = self.folder_holder.submit(highlight_index) {
            if path.is_dir() {
                let _ = fs::remove_dir_all(path);
            } else {
                let _ = fs::remove_file(path);
            }
            self.folder_holder.refresh();
        }
    }

    pub fn refresh_current_folder_cache(&mut self) {
        self.folder_holder.refresh();
    }

    pub fn reset(&mut self) {
        self.folder_holder.input.clear();
        self.folder_holder.update(None);
        self.reset_file_view();
        self.reset_index();
    }

    pub fn reset_file_view(&mut self) {
        self.file_opened = None;
        self.file_text_info = None;
    }

    fn get_highlight_index(&self, group_len: usize) -> usize {
        let divisor: i32 = group_len
            .try_into()
            .expect("Cannot convert group len of path_group");
        let remainder = self.raw_highlight_index.rem_euclid(divisor);
        remainder.try_into().expect("Unexpected!")
    }

    pub fn submit(&mut self) -> AppResult<()> {
        let path_holder = &self.folder_holder.selected_path_holder;
        if path_holder.is_empty() {
            return Ok(());
        }

        let highlight_index = self.get_highlight_index(path_holder.len());
        let new_entrypoint_canonicalized_result = self.folder_holder.submit(highlight_index);
        match new_entrypoint_canonicalized_result {
            Ok(new_entrypoint) => {
                if new_entrypoint.is_dir() {
                    if self.state_holder.borrow().is_history_search() {
                        self.state_holder.borrow_mut().to_search();
                    }
                    self.folder_holder
                        .submit_new_working_directory(new_entrypoint);
                } else {
                    self.file_text_info =
                        Some(FileTextInfo::new(&new_entrypoint, &self.code_highlighter)?);
                    self.file_opened = Some(new_entrypoint);
                    self.state_holder.borrow_mut().to_file_view();
                }
            }
            Err(_) => {
                if self.state_holder.borrow().is_history_search() {
                    self.folder_holder.drop_invalid_folder(highlight_index);
                } else {
                    self.refresh_current_folder_cache();
                }
            }
        }

        Ok(())
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
            .filter_map(|entry| {
                self.get_text(entry).ok().map(|text| {
                    ListItem::new(Line::from(text).style(if entry.is_file {
                        Style::default()
                    } else {
                        Color::LightCyan.into()
                    }))
                })
            })
            .collect();
        if path_holder.is_empty() {
            return;
        }

        let highlight_index = self.get_highlight_index(path_holder.len());
        if let Some(path) = path_holder.get_mut(highlight_index) {
            *path = path.clone().add_modifier(Modifier::REVERSED);
        };

        let block = if self.state_holder.borrow().is_history_search() {
            Block::default().title(format!("History: {} items", path_holder.len()))
        } else {
            Block::default()
                .title(self.folder_holder.current_directory.display().to_string())
                .title_bottom(
                    self.folder_holder
                        .peek()
                        .update_time
                        .format("%Y-%m-%d %H:%M:%S")
                        .to_string(),
                )
        };

        let messages = List::new(path_holder).block(block);
        frame.render_widget(messages, area);
    }

    fn get_text(&self, entry: &FileHolder) -> Result<String, std::io::Error> {
        if self.state_holder.borrow().is_history_search() {
            Ok(entry.to_path_canonicalize()?.to_string_lossy().into_owned())
        } else {
            Ok(entry.relative_to(&self.folder_holder.current_directory))
        }
    }

    fn draw_file_view(&mut self, area: Rect, frame: &mut Frame, file_path: &Path) {
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
