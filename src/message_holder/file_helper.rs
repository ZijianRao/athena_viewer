use std::fs::{self};

use chrono::{DateTime, Local};
use ratatui::text::Line;
use std::path::{Path, PathBuf};

use crate::app::app_error::{AppError, AppResult};
use crate::message_holder::code_highlighter::CodeHighlighter;

/// Maximum file size allowed for viewing (10MB)
///
/// Files larger than this limit will be rejected to prevent memory exhaustion
/// and ensure responsive performance.
pub const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;

/// Holds formatted file content and metadata for display
///
/// # Fields
///
/// - `n_rows`: Number of lines in the file
/// - `max_line_length`: Length of the longest line
/// - `formatted_text`: Syntax-highlighted lines ready for display
#[derive(Debug)]
pub struct FileTextInfo {
    pub n_rows: usize,
    pub max_line_length: usize,
    pub formatted_text: Vec<Line<'static>>,
}

/// Represents a file or directory entry
///
/// # Fields
///
/// - `parent`: Parent directory path
/// - `file_name`: Name of the file/directory
/// - `is_file`: True if this is a file, false if directory
#[derive(Debug, Clone)]
pub struct FileHolder {
    pub parent: PathBuf,
    pub file_name: String,
    pub is_file: bool,
}

/// Holds a group of files/directories with metadata
///
/// # Fields
///
/// - `child`: List of file/directory entries
/// - `update_time`: When this group was last updated
#[derive(Debug)]
pub struct FileGroupHolder {
    pub child: Vec<FileHolder>,
    pub update_time: DateTime<Local>,
}

impl FileTextInfo {
    /// Creates a new FileTextInfo by loading and highlighting a file
    ///
    /// # Arguments
    ///
    /// * `value` - Path to the file to load
    /// * `code_highlighter` - Syntax highlighter for formatting
    ///
    /// # Returns
    ///
    /// Returns `AppResult<Self>` which may contain:
    /// - `AppError::Io`: If file cannot be read
    /// - `AppError::Path`: If file is too large (> 10MB)
    /// - `AppError::Parse`: If syntax highlighting fails
    pub fn new(value: &Path, code_highlighter: &CodeHighlighter) -> AppResult<Self> {
        let meta_data = fs::metadata(value).map_err(AppError::Io)?;
        if meta_data.len() > MAX_FILE_SIZE {
            return Err(AppError::Path("File too large".into()));
        }
        let content = match fs::read_to_string(value) {
            Ok(text) => text,
            Err(_) => "Unable to read...".to_string(),
        };

        let (n_rows, max_line_length) = Self::get_string_dimensions(&content);

        Ok(Self {
            n_rows,
            max_line_length,
            formatted_text: code_highlighter.highlight(&content, value)?,
        })
    }

    fn get_string_dimensions(text: &str) -> (usize, usize) {
        let mut num_rows = 0;
        let mut max_line_length = 0;
        let mut current_line_len = 0;
        for byte in text.bytes() {
            if byte == b'\n' {
                num_rows += 1;
                max_line_length = max_line_length.max(current_line_len);
                current_line_len = 0;
            } else {
                current_line_len += 1;
            }
        }

        // Handle last line
        if !text.is_empty() {
            num_rows += 1;
            max_line_length = max_line_length.max(current_line_len);
        }

        (num_rows, max_line_length)
    }
}

impl TryFrom<PathBuf> for FileHolder {
    type Error = AppError;

    /// Converts a PathBuf to a FileHolder
    ///
    /// # Arguments
    ///
    /// * `path` - The path to convert
    ///
    /// # Returns
    ///
    /// Returns `AppResult<Self>` which may contain `AppError::Path` if:
    /// - The path has no file name
    /// - The path has no parent directory
    fn try_from(path: PathBuf) -> AppResult<Self> {
        let file_name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .ok_or(AppError::Path(format!(
                "Unable to get file name for {:?}",
                path
            )))?;

        let is_file = path.is_file();
        Ok(FileHolder {
            parent: path
                .parent()
                .ok_or(AppError::Path(format!(
                    "Unable to get parent folder for {:?}",
                    path,
                )))?
                .to_path_buf(),
            file_name,
            is_file,
        })
    }
}

impl FileHolder {
    /// Returns the canonicalized absolute path of this file/directory
    ///
    /// # Returns
    ///
    /// Returns `AppResult<PathBuf>` which may contain `AppError::Path` if
    /// the path cannot be canonicalized (e.g., doesn't exist)
    pub fn to_path_canonicalize(&self) -> AppResult<PathBuf> {
        let path = self.to_path();
        path.canonicalize().map_err(|_| {
            AppError::Path(format!("Unable to canonicalize {}", path.to_string_lossy()))
        })
    }

    /// Constructs the full path from parent and file name
    pub fn to_path(&self) -> PathBuf {
        self.parent.join(self.file_name.clone())
    }

    /// Returns the path relative to a reference directory
    ///
    /// # Arguments
    ///
    /// * `ref_path` - The reference directory path
    ///
    /// # Returns
    ///
    /// Returns `AppResult<String>` which may contain `AppError::Path` if
    /// the reference path is not a prefix of this file's path
    pub fn relative_to(&self, ref_path: &PathBuf) -> AppResult<String> {
        let rel_path = self.parent.strip_prefix(ref_path).map_err(|_| {
            AppError::Path(format!(
                "Can not get path prefix from {} for {}",
                self.parent.to_string_lossy(),
                ref_path.to_string_lossy()
            ))
        })?;
        let prefix = rel_path.to_string_lossy();
        if prefix.is_empty() {
            Ok(self.file_name.clone())
        } else {
            Ok(format!("{}/{}", prefix, self.file_name))
        }
    }
}

impl FileGroupHolder {
    /// Creates a new FileGroupHolder by reading a directory
    ///
    /// # Arguments
    ///
    /// * `path` - The directory path to read
    /// * `adding_parent_shortcut` - If true, adds ".." entry for parent navigation
    ///
    /// # Returns
    ///
    /// Returns `AppResult<Self>` which may contain:
    /// - `AppError::Parse`: If the directory cannot be read
    /// - `AppError::Path`: If individual entries cannot be processed
    pub fn new(path: PathBuf, adding_parent_shortcut: bool) -> AppResult<Self> {
        let mut entries = Vec::new();

        // add if not at root
        if adding_parent_shortcut {
            if let Some(_parent) = path.parent() {
                entries.push(FileHolder {
                    parent: path.clone(),
                    file_name: "..".to_string(),
                    is_file: false,
                })
            }
        }

        let read_dir_result = fs::read_dir(&path)
            .map_err(|_| AppError::Parse(format!("Unable to read {}", path.to_string_lossy())))?;

        for entry in read_dir_result.flatten() {
            let file_holder = FileHolder::try_from(entry.path())?;
            entries.push(file_holder);
        }

        entries.sort_by_key(|f| f.file_name.clone());
        Ok(Self {
            child: entries,
            update_time: Local::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_file_holder() {
        let temp_file = get_temp_file();
        let path = temp_file.path();
        let _ = FileHolder::try_from(path.to_path_buf()).unwrap();
    }

    #[test]
    fn test_file_text_info() {
        let temp_file = get_temp_file();
        let path = temp_file.path();
        let code_highlighter = CodeHighlighter::default();
        let file_text_info = FileTextInfo::new(&path, &code_highlighter).unwrap();
        assert_eq!(file_text_info.n_rows, 1);
        assert_eq!(file_text_info.max_line_length, 13);
    }

    fn get_temp_file() -> NamedTempFile {
        let temp_file = NamedTempFile::new().unwrap();

        let mut file = temp_file.reopen().unwrap();
        file.write_all(b"Hello, world!").unwrap();
        temp_file
    }
}
