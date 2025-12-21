use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

pub struct TestFileSystem {
    pub temp_dir: TempDir,
    pub root_path: PathBuf,
}

impl TestFileSystem {
    pub fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path().to_path_buf();
        Self {
            temp_dir,
            root_path,
        }
    }

    pub fn create_file(&self, path: &str, content: &str) -> PathBuf {
        let full_path = self.root_path.join(path);
        fs::create_dir_all(full_path.parent().unwrap()).unwrap();
        let mut file = File::create(&full_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        full_path
    }

    pub fn create_dir(&self, path: &str) -> PathBuf {
        let full_path = self.root_path.join(path);
        fs::create_dir_all(&full_path).unwrap();
        full_path
    }

    pub fn create_nested_structure(&self) {
        // root files
        self.create_file("README.md", "# Test Project\nThis is a readme.");
        self.create_file("main.rs", "fn main() { println!(\"hello\"); }");
        self.create_file(".gitkeep", "");

        // nested directories
        self.create_dir("src");
        self.create_file("src/lib.rs", "pub fn helper() {}");
        self.create_file("src/module.rs", "mod tests { /* ... */ }");

        // deep nesting
        self.create_dir("src/nested/deep");
        self.create_file("src/nested/deep/file.txt", "deep content");

        // empty directory
        self.create_dir("empty");
    }

    pub fn path(&self) -> &Path {
        &self.root_path
    }
}
