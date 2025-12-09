use std::fs::{self};

use chrono::{DateTime, Local};
use ratatui::text::Line;
use std::path::PathBuf;

use crate::message_holder::code_highlighter::CodeHighlighter;

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
    pub fn new(value: &PathBuf, code_highlighter: &CodeHighlighter) -> Self {
        let content = match fs::read_to_string(value) {
            Ok(text) => text,
            Err(_) => "Unable to read...".to_string(),
        };

        let (num_rows, max_line_length) = Self::get_string_dimensions(&content);

        Self {
            n_rows: num_rows,
            max_line_length: max_line_length,
            formatted_text: code_highlighter.highlight(&content, value),
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
            .expect(&format!("Unable to get file name for {:?}", path));

        let is_file = path.is_file();
        FileHolder {
            parent: path
                .parent()
                .expect("Must have a valid parent folder")
                .to_path_buf(),
            file_name: file_name,
            is_file: is_file,
        }
    }
}

impl FileHolder {
    pub fn to_path(&self) -> Result<PathBuf, std::io::Error> {
        self.parent.join(self.file_name.clone()).canonicalize()
    }
}

impl From<PathBuf> for FileGroupHolder {
    fn from(path: PathBuf) -> Self {
        let mut entries = Vec::new();

        // add if not at root
        if let Some(_parent) = path.parent() {
            entries.push(FileHolder {
                parent: path.clone(),
                file_name: "..".to_string(),
                is_file: false,
            })
        }

        entries.extend(
            fs::read_dir(&path)
                .expect(&format!("Unable to read directory for {:?}", path))
                .filter_map(|entry| entry.ok().map(|e| FileHolder::from(e.path()))),
        );
        Self {
            child: entries,
            update_time: Local::now(),
        }
    }
}
