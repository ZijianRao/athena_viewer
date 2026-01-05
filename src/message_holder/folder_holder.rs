use lru::LruCache;
use std::cell::RefCell;
use std::collections::HashSet;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::app::app_error::{AppError, AppResult};
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
    pub fn new(
        current_directory: PathBuf,
        state_holder: Rc<RefCell<StateHolder>>,
    ) -> AppResult<Self> {
        let holder = FileGroupHolder::new(current_directory.clone(), true)?;
        let current_holder: Vec<FileHolder> = holder.child.clone().into_iter().collect();
        let mut cache_holder = LruCache::new(DEFAULT_CACHE_SIZE);
        cache_holder.put(current_directory.clone(), holder);

        Ok(FolderHolder {
            state_holder,
            cache_holder,
            current_directory,
            input: Default::default(),
            selected_path_holder: current_holder.clone(),
            current_holder,
            expand_level: 0,
        })
    }

    pub fn expand(&mut self) -> AppResult<()> {
        let first_item = self.current_holder[0].clone();
        let value_path_group: Vec<PathBuf> = self
            .current_holder
            .iter()
            .skip(1) // ignore ".." case
            .filter_map(|p| p.to_path_canonicalize().ok())
            .collect();

        let mut result = Vec::new();
        for p in &value_path_group {
            if p.is_dir() {
                let group = FileGroupHolder::new(p.clone(), false)?;
                result.extend(group.child);
            } else {
                let file_holder = FileHolder::try_from(p.clone())?;
                result.push(file_holder);
            }
        }

        result.insert(0, first_item);
        self.current_holder = result;
        self.update(None)?;
        self.expand_level = self.expand_level.saturating_add(1);

        Ok(())
    }

    pub fn collapse(&mut self) -> AppResult<()> {
        if self.expand_level == 0 {
            return Ok(());
        }
        self.expand_level = self.expand_level.saturating_sub(1);

        let first_item = self.current_holder[0].clone();
        let mut new_current_holder: Vec<FileHolder> = Vec::new();
        new_current_holder.push(first_item);
        let mut selected_path_ref: HashSet<PathBuf> = HashSet::new();

        for item in self.current_holder.iter().skip(1) {
            let result = item.relative_to(&self.current_directory)?;
            let current_level = result.matches('/').count();

            let key = if current_level > self.expand_level {
                item.parent
                    .canonicalize()
                    .map_err(|_| {
                        AppError::Parse(format!(
                            "Unable to get parent of {}",
                            item.parent.to_string_lossy()
                        ))
                    })?
                    .clone()
            } else {
                item.to_path_canonicalize()?
            };

            if !selected_path_ref.contains(&key) {
                new_current_holder.push(FileHolder::try_from(key.clone())?);
                selected_path_ref.insert(key);
            }
        }
        self.current_holder = new_current_holder;
        self.update(None)?;

        Ok(())
    }

    pub fn put(&mut self, path: &Path) -> AppResult<()> {
        let holder = FileGroupHolder::new(path.to_path_buf(), true)?;
        self.cache_holder.put(path.to_path_buf(), holder);

        Ok(())
    }

    pub fn update(&mut self, input: Option<String>) -> AppResult<()> {
        if let Some(value) = input {
            self.input = value;
        }

        let mut selected_path_holder = Vec::new();
        if self.state_holder.borrow().is_history_search() {
            for (path, _) in &self.cache_holder {
                if let Some(path_str) = path.to_str() {
                    if self.should_select(path_str) {
                        let file_holder = FileHolder::try_from(path.clone())?;
                        selected_path_holder.push(file_holder);
                    }
                }
            }
        } else {
            for file_holder in &self.current_holder {
                if self.should_select(&file_holder.relative_to(&self.current_directory)?) {
                    selected_path_holder.push(file_holder.clone());
                }
            }
        }
        self.selected_path_holder = selected_path_holder;

        Ok(())
    }

    pub fn submit_new_working_directory(&mut self, path: PathBuf) -> AppResult<()> {
        if self.cache_holder.get(&path).is_none() {
            self.put(&path)?
        }

        self.current_directory = path;
        let cache_result =
            self.cache_holder
                .get(&self.current_directory)
                .ok_or(AppError::Cache(format!(
                    "Unable to get folder cache for {:?}",
                    self.current_directory
                )))?;

        self.current_holder = cache_result.child.clone();

        self.input.clear();
        self.update(None)?;
        self.expand_level = 0;

        Ok(())
    }

    pub fn refresh(&mut self) -> AppResult<()> {
        let holder = FileGroupHolder::new(self.current_directory.clone(), true)?;
        self.current_holder = holder.child.clone();
        self.update(None)?;

        self.cache_holder
            .put(self.current_directory.clone(), holder)
            .ok_or(AppError::Cache(format!(
                "Unable to insert folder cache for {:?}",
                self.current_directory
            )))?;

        Ok(())
    }

    fn should_select(&self, name: &str) -> bool {
        Self::should_select_helper(name, &self.input)
    }

    fn should_select_helper(name: &str, input: &str) -> bool {
        if input.is_empty() {
            return true;
        }

        // check if all characters in self.input appear in order (case-insensitive) in name
        let mut input_iter = input.chars();
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

    pub fn submit(&mut self, index: usize) -> AppResult<PathBuf> {
        self.selected_path_holder[index].to_path_canonicalize()
    }

    pub fn drop_invalid_folder(&mut self, index: usize) -> AppResult<()> {
        if !self.state_holder.borrow().is_history_search() {
            return Err(AppError::State("Must be in history mode".into()));
        }
        let removed = self.selected_path_holder.remove(index);
        self.cache_holder
            .pop(&removed.to_path())
            .ok_or(AppError::Cache(
                "Must contain the invalid path in cache".into(),
            ))?;
        Ok(())
    }

    pub fn peek(&self) -> AppResult<&FileGroupHolder> {
        self.cache_holder
            .peek(&self.current_directory)
            .ok_or(AppError::Cache(format!(
                "Unable to get cache for {:?}",
                self.current_directory
            )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_select() {
        assert!(FolderHolder::should_select_helper("abc", "c"));
        assert!(FolderHolder::should_select_helper("abc", ""));
        assert!(!FolderHolder::should_select_helper("abc", "d"));
        assert!(!FolderHolder::should_select_helper("abc", "abcd"));
    }
}
