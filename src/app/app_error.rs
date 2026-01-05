use std::io::{self};
use thiserror::{self};

/// Application error types with detailed context
///
/// All errors implement `std::error::Error` via `thiserror` for proper error handling
/// and display. Use `AppResult<T>` for functions that may return these errors.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// Filesystem I/O errors (from `std::io::Error`)
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// Path resolution and validation errors
    ///
    /// Examples: invalid paths, missing parents, file too large
    #[error("Path error: {0}")]
    Path(String),

    /// Parsing and conversion errors
    ///
    /// Examples: string parsing, type conversion failures
    #[error("Parse error: {0}")]
    Parse(String),

    /// Invalid application state errors
    ///
    /// Examples: wrong mode for operation, missing required state
    #[error("State error: {0}")]
    State(String),

    /// Terminal/rendering errors
    ///
    /// Examples: terminal initialization, draw failures, event polling
    #[error("Terminal error: {0}")]
    Terminal(String),

    /// Cache operation errors
    ///
    /// Examples: missing cache entries, failed insertions
    #[error("Cache error: {0}")]
    Cache(String),
}

/// Result type for application operations
///
/// Shorthand for `Result<T, AppError>` used throughout the codebase
pub type AppResult<T> = Result<T, AppError>;
