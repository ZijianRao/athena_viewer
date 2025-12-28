use std::io::{self};
use thiserror::{self};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Path error: {0}")]
    Path(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("State error: {0}")]
    State(String),
    #[error("Terminal error: {0}")]
    Terminal(String),
    #[error("Cache error: {0}")]
    Cache(String),
}

pub type AppResult<T> = Result<T, AppError>;
