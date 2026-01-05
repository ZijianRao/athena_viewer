//! File viewing, directory navigation, and message display
//!
//! This module handles:
//! - Loading and displaying files with syntax highlighting
//! - Directory navigation and search filtering
//! - File/folder caching for performance
//! - Scroll state management
//!
//! # Core Types
//!
//! - [`MessageHolder`]: Main controller for file/directory operations
//! - [`FolderHolder`]: Directory navigation and caching
//! - [`FileHolder`]: Individual file/folder metadata
//! - [`FileTextInfo`]: File content with formatting
//! - [`CodeHighlighter`]: Syntax highlighting using syntect

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

use crate::app::app_error::{AppError, AppResult};
use crate::message_holder::code_highlighter::CodeHighlighter;
use crate::message_holder::file_helper::{FileHolder, FileTextInfo};
use crate::message_holder::folder_holder::FolderHolder;
use crate::state_holder::StateHolder;

/// Main controller for file viewing and directory navigation
///
/// Coordinates between the UI, file system operations, and state management.
/// Handles file loading, directory browsing, search filtering, and rendering.
///
/// # Fields
///
/// - `state_holder`: Shared state machine reference
/// - `folder_holder`: Directory navigation and caching
/// - `code_highlighter`: Syntax highlighting engine
/// - `raw_highlight_index`: Current selection index (before wrapping)
/// - `file_opened`: Currently open file path (if any)
/// - `file_text_info`: Loaded file content and metadata (if file open)
/// - `vertical_scroll_state`: Scrollbar state for vertical scrolling
/// - `horizontal_scroll_state`: Scrollbar state for horizontal scrolling
/// - `vertical_scroll`: Current vertical scroll position
/// - `horizontal_scroll`: Current horizontal scroll position
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
    /// Creates a new MessageHolder instance
    ///
    /// # Arguments
    ///
    /// * `current_directory` - Starting directory for navigation
    /// * `state_holder` - Shared state machine reference
    ///
    /// # Returns
    ///
    /// Returns `AppResult<Self>` which may contain:
    /// - `AppError::Io`: If directory cannot be read
    /// - `AppError::Path`: If path resolution fails
    /// - `AppError::Cache`: If cache initialization fails
    pub fn new(
        current_directory: PathBuf,
        state_holder: Rc<RefCell<StateHolder>>,
    ) -> AppResult<Self> {
        let state_holder_ref = Rc::clone(&state_holder);
        Ok(MessageHolder {
            state_holder,
            code_highlighter: CodeHighlighter::default(),
            folder_holder: FolderHolder::new(current_directory, state_holder_ref)?,
            raw_highlight_index: 0,
            file_opened: Default::default(),
            file_text_info: Default::default(),
            vertical_scroll_state: Default::default(),
            horizontal_scroll_state: Default::default(),
            vertical_scroll: Default::default(),
            horizontal_scroll: Default::default(),
        })
    }

    /// Resets the selection index to 0
    pub fn reset_index(&mut self) {
        self.raw_highlight_index = 0;
    }

    /// Moves the selection up by one item
    ///
    /// Uses saturating subtraction to prevent underflow
    pub fn move_up(&mut self) {
        self.raw_highlight_index = self.raw_highlight_index.saturating_sub(1);
    }

    /// Moves the selection down by one item
    ///
    /// Uses saturating addition to prevent overflow
    pub fn move_down(&mut self) {
        self.raw_highlight_index = self.raw_highlight_index.saturating_add(1);
    }

    /// Updates the filtered view based on search input
    ///
    /// # Arguments
    ///
    /// * `input` - Optional search filter string
    ///
    /// # Returns
    ///
    /// Returns `AppResult<()>` which may contain `AppError::State` if update fails
    pub fn update(&mut self, input: Option<String>) -> AppResult<()> {
        self.folder_holder.update(input)?;
        self.reset_index();
        Ok(())
    }

    /// Expands all directories recursively
    ///
    /// # Returns
    ///
    /// Returns `AppResult<()>` which may contain:
    /// - `AppError::Io`: If directory reading fails
    /// - `AppError::Path`: If path resolution fails
    pub fn expand(&mut self) -> AppResult<()> {
        self.folder_holder.expand()?;
        Ok(())
    }

    /// Collapses expanded directories
    ///
    /// # Returns
    ///
    /// Returns `AppResult<()>` which may contain `AppError::State` on invalid state
    pub fn collapse(&mut self) -> AppResult<()> {
        self.folder_holder.collapse()?;
        Ok(())
    }

    /// Navigates to the parent directory
    ///
    /// Resets the selection index and refreshes the view
    ///
    /// # Returns
    ///
    /// Returns `AppResult<()>` which may contain directory navigation errors
    pub fn to_parent(&mut self) -> AppResult<()> {
        self.raw_highlight_index = 0;
        self.update(Some("".into()))?;
        self.submit()?;
        Ok(())
    }

    /// Deletes the currently selected file or directory
    ///
    /// # Returns
    ///
    /// Returns `AppResult<()>` which may contain:
    /// - `AppError::Path`: If path resolution fails
    /// - `AppError::State`: If selection is invalid
    pub fn delete(&mut self) -> AppResult<()> {
        let path_holder = &self.folder_holder.selected_path_holder;
        if path_holder.is_empty() {
            return Ok(());
        }

        let highlight_index = self.get_highlight_index(path_holder.len())?;
        if let Ok(path) = self.folder_holder.submit(highlight_index) {
            if path.is_dir() {
                let _ = fs::remove_dir_all(path);
            } else {
                let _ = fs::remove_file(path);
            }
            self.folder_holder.refresh()?;
        }
        Ok(())
    }

    /// Refreshes the current folder's cache
    ///
    /// # Returns
    ///
    /// Returns `AppResult<()>` which may contain `AppError::Cache` or `AppError::Io`
    pub fn refresh_current_folder_cache(&mut self) -> AppResult<()> {
        self.folder_holder.refresh()?;
        Ok(())
    }

    /// Resets the message holder to initial state
    ///
    /// Clears input, resets file view, and resets selection index
    ///
    /// # Returns
    ///
    /// Returns `AppResult<()>` which may contain directory update errors
    pub fn reset(&mut self) -> AppResult<()> {
        self.folder_holder.input.clear();
        self.folder_holder.update(None)?;
        self.reset_file_view();
        self.reset_index();
        Ok(())
    }

    /// Resets the file view state
    ///
    /// Clears the currently opened file and its text info
    pub fn reset_file_view(&mut self) {
        self.file_opened = None;
        self.file_text_info = None;
    }

    /// Converts raw highlight index to wrapped index within bounds
    ///
    /// # Arguments
    ///
    /// * `group_len` - The number of items in the current selection
    ///
    /// # Returns
    ///
    /// Returns the wrapped index or `AppError::Parse` if conversion fails
    fn get_highlight_index(&self, group_len: usize) -> AppResult<usize> {
        Self::get_highlight_index_helper(self.raw_highlight_index, group_len)
    }

    /// Helper function for index wrapping with Euclidean division
    ///
    /// Handles negative indices by wrapping around using modulo arithmetic
    fn get_highlight_index_helper(raw_highlight_index: i32, group_len: usize) -> AppResult<usize> {
        let divisor: i32 = group_len
            .try_into()
            .map_err(|_| AppError::Parse("Cannot convert group len of path_group".into()))?;
        let remainder = raw_highlight_index.rem_euclid(divisor);
        let out: usize = remainder
            .try_into()
            .map_err(|_| AppError::Parse("Cannot convert group len of path_group".into()))?;
        Ok(out)
    }

    /// Submits the current selection (navigates into directory or opens file)
    ///
    /// # Returns
    ///
    /// Returns `AppResult<()>` which may contain:
    /// - `AppError::Path`: If path resolution fails
    /// - `AppError::Parse`: If file parsing fails
    /// - `AppError::Cache`: If cache operations fail
    pub fn submit(&mut self) -> AppResult<()> {
        let path_holder = &self.folder_holder.selected_path_holder;
        if path_holder.is_empty() {
            return Ok(());
        }

        let highlight_index = self.get_highlight_index(path_holder.len())?;
        let new_entrypoint_canonicalized_result = self.folder_holder.submit(highlight_index);
        match new_entrypoint_canonicalized_result {
            Ok(new_entrypoint) => {
                if new_entrypoint.is_dir() {
                    if self.state_holder.borrow().is_history_search() {
                        self.state_holder.borrow_mut().to_search();
                    }
                    self.folder_holder
                        .submit_new_working_directory(new_entrypoint)?;
                } else {
                    self.file_text_info =
                        Some(FileTextInfo::new(&new_entrypoint, &self.code_highlighter)?);
                    self.file_opened = Some(new_entrypoint);
                    self.state_holder.borrow_mut().to_file_view();
                }
            }
            Err(_) => {
                if self.state_holder.borrow().is_history_search() {
                    self.folder_holder.drop_invalid_folder(highlight_index)?;
                } else {
                    self.refresh_current_folder_cache()?;
                }
            }
        }

        Ok(())
    }

    /// Renders the current view to the terminal
    ///
    /// # Arguments
    ///
    /// * `area` - The rectangular area to render in
    /// * `frame` - The ratatui frame to render to
    ///
    /// # Returns
    ///
    /// Returns `AppResult<()>` which may contain rendering errors
    pub fn draw(&mut self, area: Rect, frame: &mut Frame) -> AppResult<()> {
        match self.file_opened.clone() {
            None => self.draw_folder_view(area, frame),
            Some(file_path) => self.draw_file_view(area, frame, &file_path),
        }
    }

    fn draw_folder_view(&mut self, area: Rect, frame: &mut Frame) -> AppResult<()> {
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
            return Ok(());
        }

        let highlight_index = self.get_highlight_index(path_holder.len())?;
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
                        .peek()?
                        .update_time
                        .format("%Y-%m-%d %H:%M:%S")
                        .to_string(),
                )
        };

        let messages = List::new(path_holder).block(block);
        frame.render_widget(messages, area);
        Ok(())
    }

    fn get_text(&self, entry: &FileHolder) -> AppResult<String> {
        if self.state_holder.borrow().is_history_search() {
            Ok(entry.to_path_canonicalize()?.to_string_lossy().into_owned())
        } else {
            entry.relative_to(&self.folder_holder.current_directory)
        }
    }

    fn draw_file_view(&mut self, area: Rect, frame: &mut Frame, file_path: &Path) -> AppResult<()> {
        let file_text_info = self
            .file_text_info
            .as_ref()
            .ok_or(AppError::Parse("Unexpected, file should be opened".into()))?;
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
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_get_highlight_index_helper_common() {
        let act = MessageHolder::get_highlight_index_helper(1, 10).unwrap();
        let exp = 1;
        assert_eq!(act, exp);

        let act = MessageHolder::get_highlight_index_helper(0, 100).unwrap();
        let exp = 0;
        assert_eq!(act, exp);
    }

    #[test]
    fn test_get_highlight_index_helper_neg() {
        let act = MessageHolder::get_highlight_index_helper(-1, 10).unwrap();
        let exp = 9;
        assert_eq!(act, exp);
    }

    #[test]
    fn test_get_highlight_index_helper_large() {
        let act = MessageHolder::get_highlight_index_helper(5, 3).unwrap();
        let exp = 2;
        assert_eq!(act, exp);
    }
}
