//! State-specific event handlers and UI rendering
//!
//! Each module handles events and rendering for a specific combination of
//! `InputMode` and `ViewMode` from the state machine.
//!
//! # Modules
//!
//! - `normal_search` - Normal input mode with search view
//! - `normal_file_view` - Normal input mode with file viewing
//! - `edit_search` - Edit input mode with search view
//! - `edit_history_folder_view` - Edit input mode with history/folder view

pub mod edit_history_folder_view;
pub mod edit_search;
pub mod normal_file_view;
pub mod normal_search;
