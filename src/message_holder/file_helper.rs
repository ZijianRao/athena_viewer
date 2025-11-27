use std::fs::{self};

use chrono::{DateTime, Local};
use std::path::PathBuf;

#[derive(Debug)]
pub struct FileTextInfo {
    pub text: String,
    pub n_rows: usize,
    pub max_line_length: usize,
}

#[derive(Debug, Clone)]
pub struct FileHolder {
    pub file_name: String,
    pub is_file: bool,
}

#[derive(Debug)]
pub struct FileGroupHolder {
    pub parent: PathBuf,
    pub child: Vec<FileHolder>,
    pub update_time: DateTime<Local>,
}

impl FileTextInfo {
    pub fn new(value: &PathBuf) -> Self {
        let content = match fs::read_to_string(value) {
            Ok(text) => text,
            Err(_) => "Unable to read...".to_string(),
        };

        let (num_rows, max_line_length) = Self::get_string_dimensions(&content);

        Self {
            text: content,
            n_rows: num_rows,
            max_line_length: max_line_length,
        }
    }

    fn get_string_dimensions(text: &str) -> (usize, usize) {
        let lines: Vec<&str> = text.split('\n').collect();
        let num_rows = lines.len();
        let max_line_length = lines.iter().map(|line| line.len()).max().unwrap_or(0);
        (num_rows, max_line_length)
    }
}

impl From<PathBuf> for FileHolder {
    fn from(path: PathBuf) -> Self {
        let file_name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap();

        FileHolder {
            file_name: file_name,
            is_file: path.is_file(),
        }
    }
}

impl From<PathBuf> for FileGroupHolder {
    fn from(path: PathBuf) -> Self {
        let mut entries = Vec::new();

        // add if not at root
        if let Some(_parent) = path.parent() {
            entries.push(FileHolder {
                file_name: "..".to_string(),
                is_file: false,
            })
        }

        entries.extend(
            fs::read_dir(&path)
                .unwrap()
                .filter_map(|entry| entry.ok().map(|e| FileHolder::from(e.path()))),
        );
        Self {
            child: entries,
            parent: path,
            update_time: Local::now(),
        }
    }
}
