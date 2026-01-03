use std::fs::{self};

use chrono::{DateTime, Local};
use ratatui::text::Line;
use std::path::{Path, PathBuf};

use crate::app::app_error::{AppError, AppResult};
use crate::message_holder::code_highlighter::CodeHighlighter;
const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10MB

#[derive(Debug)]
pub struct FileTextInfo {
    pub n_rows: usize,
    pub max_line_length: usize,
    pub formatted_text: Vec<Line<'static>>,
}

#[derive(Debug, Clone)]
pub struct FileHolder {
    pub parent: PathBuf,
    pub file_name: String,
    pub is_file: bool,
}

#[derive(Debug)]
pub struct FileGroupHolder {
    pub child: Vec<FileHolder>,
    pub update_time: DateTime<Local>,
}

impl FileTextInfo {
    pub fn new(value: &Path, code_highlighter: &CodeHighlighter) -> AppResult<Self> {
        let meta_data = fs::metadata(value).map_err(|e| AppError::Io(e))?;
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
        let lines: Vec<&str> = text.split('\n').collect();
        let num_rows = lines.len();
        let max_line_length = lines.iter().map(|line| line.len()).max().unwrap_or(0);
        (num_rows, max_line_length)
    }
}

impl TryFrom<PathBuf> for FileHolder {
    type Error = AppError;
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
    pub fn to_path_canonicalize(&self) -> AppResult<PathBuf> {
        let path = self.to_path();
        path.canonicalize().map_err(|_| {
            AppError::Path(format!("Unable to canonicalize {}", path.to_string_lossy()))
        })
    }

    pub fn to_path(&self) -> PathBuf {
        self.parent.join(self.file_name.clone())
    }

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
