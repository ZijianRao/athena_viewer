use lru::LruCache;
use std::cell::RefCell;
use std::collections::HashSet;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::message_holder::file_helper::{FileGroupHolder, FileHolder};
use crate::state_holder::StateHolder;

const DEFAULT_CACHE_SIZE: NonZeroUsize = match NonZeroUsize::new(100) {
    Some(size) => size,
    None => panic!("DEFAULT_CACHE_SIZE must be non-zero"),
};

#[derive(Debug)]
pub struct FolderHolder {
    state_holder: Rc<RefCell<StateHolder>>,
    cache_holder: LruCache<PathBuf, FileGroupHolder>,
    pub input: String,
    pub selected_path_holder: Vec<FileHolder>,
    pub current_directory: PathBuf,
    current_holder: Vec<FileHolder>,
    expand_level: usize,
}

impl FolderHolder {
    pub fn new(current_directory: PathBuf, state_holder: Rc<RefCell<StateHolder>>) -> Self {
        let holder = FileGroupHolder::new(current_directory.clone(), true);
        let current_holder: Vec<FileHolder> = holder.child.clone().into_iter().collect();
        let mut cache_holder = LruCache::new(DEFAULT_CACHE_SIZE);
        cache_holder.put(current_directory.clone(), holder);

        FolderHolder {
            state_holder,
            cache_holder,
            current_directory,
            input: Default::default(),
            selected_path_holder: current_holder.clone(),
            current_holder,
            expand_level: 0,
        }
    }

    pub fn expand(&mut self) {
        let first_item = self.current_holder[0].clone();
        let value_path_group: Vec<PathBuf> = self
            .current_holder
            .iter()
            .skip(1) // ignore ".." case
            .filter_map(|p| p.to_path_canonicalize().ok())
            .collect();

        let mut result: Vec<FileHolder> = value_path_group
            .iter()
            .flat_map(|p| {
                if p.is_dir() {
                    FileGroupHolder::new(p.clone(), false).child
                } else {
                    vec![FileHolder::from(p.clone())]
                }
            })
            .collect();
        result.insert(0, first_item);
        self.current_holder = result;
        self.update(None);
        self.expand_level = self.expand_level.saturating_add(1);
    }

    pub fn collapse(&mut self) {
        if self.expand_level == 0 {
            return;
        }
        self.expand_level = self.expand_level.saturating_sub(1);

        let first_item = self.current_holder[0].clone();
        let mut new_current_holder: Vec<FileHolder> = Vec::new();
        new_current_holder.push(first_item);
        let mut selected_path_ref: HashSet<PathBuf> = HashSet::new();

        for item in self.current_holder.iter().skip(1) {
            let result = item.relative_to(&self.current_directory);
            let current_level = result.matches('/').count();

            let key = if current_level > self.expand_level {
                item.parent
                    .canonicalize()
                    .expect("Cannot canonicalize?")
                    .clone()
            } else {
                item.to_path_canonicalize()
                    .expect("Expect to have valid path")
                    .canonicalize()
                    .expect("Cannot canonicalize?")
            };

            if !selected_path_ref.contains(&key) {
                new_current_holder.push(FileHolder::from(key.clone()));
                selected_path_ref.insert(key);
            }
        }
        self.current_holder = new_current_holder;
        self.update(None);
    }

    pub fn put(&mut self, path: &Path) {
        let holder = FileGroupHolder::new(path.to_path_buf(), true);
        self.cache_holder.put(path.to_path_buf(), holder);
    }

    pub fn update(&mut self, input: Option<String>) {
        if let Some(value) = input {
            self.input = value;
        }

        if self.state_holder.borrow().is_history_search() {
            self.selected_path_holder = self
                .cache_holder
                .iter()
                .filter(|(path, _)| {
                    self.should_select(
                        path.to_str()
                            .unwrap_or_else(|| panic!("Unable to get path {:?}", path)),
                    )
                })
                .map(|(path, _)| FileHolder::from(path.clone()))
                .collect();
        } else {
            self.selected_path_holder = self
                .current_holder
                .clone()
                .into_iter()
                .filter(|entry| self.should_select(&entry.relative_to(&self.current_directory)))
                .collect();
        }
    }

    pub fn submit_new_working_directory(&mut self, path: PathBuf) {
        if self.cache_holder.get(&path).is_none() {
            self.put(&path)
        }

        self.current_directory = path;
        self.current_holder = self
            .cache_holder
            .get(&self.current_directory)
            .unwrap_or_else(|| {
                panic!(
                    "Unable to get folder cache for {:?}",
                    self.current_directory
                )
            })
            .child
            .clone();
        self.input.clear();
        self.update(None);
        self.expand_level = 0;
    }

    pub fn refresh(&mut self) {
        let holder = FileGroupHolder::new(self.current_directory.clone(), true);
        self.current_holder = holder.child.clone();
        self.update(None);

        self.cache_holder
            .put(self.current_directory.clone(), holder);
    }

    fn should_select(&self, name: &str) -> bool {
        if self.input.is_empty() {
            return true;
        }

        // check if all charactoer in self.input appear in order (case-insensitive) in name
        let mut input_iter = self.input.chars();
        let mut next_to_match = input_iter.next();

        for name_char in name.chars() {
            match next_to_match {
                Some(input_char) if name_char.eq_ignore_ascii_case(&input_char) => {
                    next_to_match = input_iter.next();
                }
                None => return true,
                _ => (),
            }
        }

        next_to_match.is_none()
    }

    pub fn submit(&mut self, index: usize) -> Result<PathBuf, std::io::Error> {
        self.selected_path_holder[index].to_path_canonicalize()
    }

    pub fn drop_invalid_folder(&mut self, index: usize) {
        assert!(self.state_holder.borrow().is_history_search());
        let removed = self.selected_path_holder.remove(index);
        self.cache_holder
            .pop(&removed.to_path())
            .expect("Must contain the invalid path in cache");
    }

    pub fn peek(&self) -> &FileGroupHolder {
        self.cache_holder
            .peek(&self.current_directory)
            .unwrap_or_else(|| panic!("Unable to get cache for {:?}", self.current_directory))
    }
}
