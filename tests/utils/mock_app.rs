use athena_viewer::app::app_error::AppResult;
use athena_viewer::app::App;
use athena_viewer::state_holder::{InputMode, ViewMode};
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::Event;
use ratatui::Terminal;
use std::path::PathBuf;

pub struct TestApp {
    pub app: App,
    pub terminal: Terminal<TestBackend>,
}

impl TestApp {
    /// create a new test app starting in a specific directory
    pub fn new(start_dir: PathBuf) -> Self {
        // change to test directory

        let terminal = super::mock_terminal::create_test_terminal();
        let app = App::new(start_dir);

        Self { app, terminal }
    }

    /// send an event to the app and process it
    pub fn send_event(&mut self, event: Event) -> AppResult<()> {
        // simulate the event handling that happens in the main loop
        // we need to manually call the appropriate handler based on the current state

        let input_mode = self.app.state_holder.borrow().input_mode;
        let view_mode = self.app.state_holder.borrow().view_mode;

        use InputMode::*;
        use ViewMode::*;

        match (input_mode, view_mode) {
            (Normal, Search) => self.app.handle_normal_search_event(event)?,
            (Normal, FileView) => self.app.handle_normal_file_view_event(event)?,
            (Edit, HistoryFolderView) => self.app.handle_edit_history_folder_view_event(event)?,
            (Edit, Search) => self.app.handle_edit_search_event(event)?,
            _ => (),
        }
        Ok(())
    }

    /// send a sequence of events
    pub fn send_events(&mut self, events: Vec<Event>) -> AppResult<()> {
        for event in events {
            self.send_event(event)?;
        }

        Ok(())
    }

    /// get current input mode
    pub fn get_input_mode(&self) -> InputMode {
        self.app.state_holder.borrow().input_mode
    }

    /// get current view mode
    pub fn get_view_mode(&self) -> ViewMode {
        self.app.state_holder.borrow().view_mode
    }

    /// check if in specific modes
    pub fn is_normal_mode(&self) -> bool {
        self.get_input_mode() == InputMode::Normal
    }

    pub fn is_edit_mode(&self) -> bool {
        self.get_input_mode() == InputMode::Edit
    }

    pub fn is_search_view(&self) -> bool {
        self.get_view_mode() == ViewMode::Search
    }

    pub fn is_file_view(&self) -> bool {
        self.get_view_mode() == ViewMode::FileView
    }

    pub fn is_history_view(&self) -> bool {
        self.get_view_mode() == ViewMode::HistoryFolderView
    }

    /// get current file opened (if any)
    pub fn get_opened_file(&self) -> Option<PathBuf> {
        self.app.message_holder.file_opened.clone()
    }

    /// get current directory from message holder
    pub fn get_current_directory(&self) -> PathBuf {
        self.app
            .message_holder
            .folder_holder
            .current_directory
            .clone()
    }

    /// get current search/filter input
    pub fn get_search_input(&self) -> String {
        self.app.input.value().to_string()
    }

    /// get list of visible files/folders (for assertions)
    pub fn get_visible_items(&self) -> Vec<String> {
        let is_history_view = self.is_history_view();
        self.app
            .message_holder
            .folder_holder
            .selected_path_holder
            .iter()
            .map(|entry| {
                if is_history_view {
                    entry
                        .to_path_canonicalize()
                        .unwrap()
                        .to_string_lossy()
                        .into_owned()
                } else {
                    entry.relative_to(&self.app.message_holder.folder_holder.current_directory)
                }
            })
            .collect()
    }

    pub fn get_scroll_positions(&self) -> (usize, usize) {
        (
            self.app.message_holder.vertical_scroll,
            self.app.message_holder.horizontal_scroll,
        )
    }

    /// render the current frame (useful for debugging)
    pub fn render_frame(&mut self) {
        let _ = self.terminal.draw(|frame| self.app.draw(frame));
    }
}
