//! Application state machine
//!
//! This module provides a state machine for managing the application's
//! input and view modes. It supports state transitions and state restoration.
//!
//! # State Model
//!
//! The application has two independent state dimensions:
//!
//! ## Input Mode
//! - `Normal`: Read-only mode, keyboard shortcuts for navigation
//! - `Edit`: Input mode, typing search/filter queries
//!
//! ## View Mode
//! - `Search`: Browsing current directory with search filter
//! - `FileView`: Viewing a file's contents
//! - `HistoryFolderView`: Browsing cached directories (history)
//!
//! # State Transitions
//!
//! ```text
//! [Normal+Search] <---> [Edit+Search]
//!      |                     |
//!      v                     v
//! [Normal+FileView]   [Edit+HistoryFolderView]
//! ```

use InputMode::*;
use ViewMode::*;

/// Input mode: controls how keyboard input is handled
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum InputMode {
    /// Read-only mode, navigation shortcuts
    Normal,
    /// Edit mode, typing search queries
    #[default]
    Edit,
}

/// View mode: controls what content is displayed
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum ViewMode {
    /// Directory browsing with search filter
    #[default]
    Search,
    /// File content viewing
    FileView,
    /// History/cached directory browsing
    HistoryFolderView,
}

/// Application state holder with state restoration support
///
/// # Fields
///
/// - `input_mode`: Current input mode
/// - `view_mode`: Current view mode
/// - `prev_input_mode`: Previous input mode (for restoration)
/// - `prev_view_mode`: Previous view mode (for restoration)
#[derive(Debug, Default, PartialEq)]
pub struct StateHolder {
    pub input_mode: InputMode,
    pub view_mode: ViewMode,
    prev_input_mode: InputMode,
    prev_view_mode: ViewMode,
}

impl StateHolder {
    /// Transitions to Normal+Search mode
    ///
    /// Used for browsing current directory with keyboard navigation
    pub fn to_search(&mut self) {
        self.save_previous_state();
        self.input_mode = Normal;
        self.view_mode = Search;
    }
    /// Transitions to Edit+Search mode
    ///
    /// Used for typing search/filter queries
    pub fn to_search_edit(&mut self) {
        self.save_previous_state();
        self.input_mode = Edit;
        self.view_mode = Search;
    }

    /// Transitions to Edit+HistoryFolderView mode
    ///
    /// Used for searching through cached directory history
    pub fn to_history_search(&mut self) {
        self.save_previous_state();
        self.input_mode = Edit;
        self.view_mode = HistoryFolderView;
    }

    /// Transitions to Normal+FileView mode
    ///
    /// Used for viewing file contents
    pub fn to_file_view(&mut self) {
        self.save_previous_state();
        self.input_mode = Normal;
        self.view_mode = FileView;
    }

    /// Checks if currently in Edit mode
    pub fn is_edit(&self) -> bool {
        self.input_mode == Edit
    }

    /// Checks if currently viewing history
    pub fn is_history_search(&self) -> bool {
        self.view_mode == HistoryFolderView
    }

    /// Checks if currently viewing a file
    pub fn is_file_view(&self) -> bool {
        self.view_mode == FileView
    }

    /// Saves the current state for later restoration
    fn save_previous_state(&mut self) {
        self.prev_input_mode = self.input_mode;
        self.prev_view_mode = self.view_mode;
    }

    /// Restores the previous state
    ///
    /// Useful for returning from file view to the previous mode
    pub fn restore_previous_state(&mut self) {
        self.input_mode = self.prev_input_mode;
        self.view_mode = self.prev_view_mode;
    }
}
