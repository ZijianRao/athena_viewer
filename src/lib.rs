//! Athena Viewer - A terminal-based file viewer with syntax highlighting
//!
//! This library provides the core functionality for the Athena Viewer application,
//! including application state management, file/directory navigation, message display,
//! and syntax highlighting.
//!
//! # Architecture
//!
//! - [`app`]: Main application logic and event handling
//! - [`message_holder`]: File viewing, directory navigation, and syntax highlighting
//! - [`state_holder`]: State machine for managing application modes
//!
//! # Core Types
//!
//! - [`app::App`]: Main application struct
//! - [`app::app_error::AppError`]: Error types with `thiserror`
//! - [`state_holder::StateHolder`]: State machine for input/view modes

pub mod app;
pub mod message_holder;
pub mod state_holder;
