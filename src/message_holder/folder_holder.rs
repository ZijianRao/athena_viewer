use std::env;

use lru::LruCache;
use std::cell::RefCell;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::rc::Rc;

use crate::message_holder::file_helper::{FileGroupHolder, FileHolder};
use crate::state_holder::state_holder::StateHolder;

const DEFAULT_CACHE_SIZE: NonZeroUsize = match NonZeroUsize::new(100) {
    Some(size) => size,
    None => panic!("DEFAULT_CACHE_SIZE must be non-zero"),
};

#[derive(Debug)]
pub struct FolderHolder {
    state_holder: Rc<RefCell<StateHolder>>,
    cache_holder: LruCache<PathBuf, FileGroupHolder>,
    input: String,
    pub selected_path_holder: Vec<FileHolder>,
    pub current_directory: PathBuf,
}

impl FolderHolder {
    pub fn new(state_holder: Rc<RefCell<StateHolder>>) -> Self {
        let current_directory = env::current_dir().expect("Unable to get current directory!");
        let holder = FileGroupHolder::from(current_directory.clone());
        let selected_path_holder: Vec<FileHolder> = holder.child.clone().into_iter().collect();
        let mut cache_holder = LruCache::new(DEFAULT_CACHE_SIZE);
        cache_holder.put(current_directory.clone(), holder);

        FolderHolder {
            state_holder: state_holder,
            cache_holder: cache_holder,
            current_directory: current_directory,
            input: Default::default(),
            selected_path_holder: selected_path_holder,
        }
    }

    pub fn put(&mut self, path: &PathBuf) {
        let holder = FileGroupHolder::from(path.clone());
        self.cache_holder.put(path.clone(), holder);
    }

    pub fn update(&mut self, input: &str) {
        self.input = input.to_string();
        let messages: Vec<FileHolder>;
        if self.state_holder.borrow().is_history_search() {
            self.selected_path_holder = self
                .cache_holder
                .iter()
                .filter(|(path, _)| {
                    self.should_select(
                        path.to_str()
                            .expect(&format!("Unable to get path {:?}", path)),
                    )
                })
                .map(|(path, _)| FileHolder::from(path.clone()))
                .collect();
        } else {
            messages = self
                .cache_holder
                .get(&self.current_directory)
                .expect(&format!(
                    "Unable to get folder cache for {:?}",
                    self.current_directory
                ))
                .child
                .clone();
            self.selected_path_holder = messages
                .into_iter()
                .filter(|entry| self.should_select(&entry.file_name))
                .collect();
        }
    }

    pub fn submit_new_working_directory(&mut self, path: PathBuf) {
        if self.cache_holder.get(&path).is_none() {
            self.put(&path)
        }

        self.current_directory = path;
        self.input.clear();
        self.update("");
    }

    pub fn refresh(&mut self) {
        let holder = FileGroupHolder::from(self.current_directory.clone());
        self.cache_holder
            .put(self.current_directory.clone(), holder);
        self.update(&self.input.clone());
    }

    fn should_select(&self, name: &str) -> bool {
        if self.input.is_empty() {
            return true;
        }

        let mut counter = 0;
        for char in name.chars() {
            if char.eq_ignore_ascii_case(
                &self
                    .input
                    .chars()
                    .nth(counter)
                    .expect("Should not reach out of bounds"),
            ) {
                counter += 1;
            }
            if counter == self.input.len() {
                return true;
            }
        }

        false
    }

    pub fn submit(&mut self, index: usize) -> Result<PathBuf, std::io::Error> {
        self.selected_path_holder[index].to_path()
    }

    pub fn peek(&self) -> &FileGroupHolder {
        self.cache_holder
            .peek(&self.current_directory)
            .expect(&format!(
                "Unable to get cache for {:?}",
                self.current_directory
            ))
    }
}
