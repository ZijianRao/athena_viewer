use InputMode::*;
use ViewMode::*;

#[derive(Debug, Default, PartialEq, Clone)]
pub enum InputMode {
    #[default]
    Normal,
    Edit,
}

#[derive(Debug, Default, PartialEq, Clone)]
pub enum ViewMode {
    #[default]
    Search,
    FileView,
    HistoryFolderView,
}

#[derive(Debug, Default, PartialEq)]
pub struct StateHolder {
    pub input_mode: InputMode,
    pub view_mode: ViewMode,
    prev_input_mode: InputMode,
    prev_view_mode: ViewMode,
}

impl StateHolder {
    pub fn to_search(&mut self) {
        self.save_previous_state();
        self.input_mode = Normal;
        self.view_mode = Search;
    }
    pub fn to_search_edit(&mut self) {
        self.save_previous_state();
        self.input_mode = Edit;
        self.view_mode = Search;
    }
    pub fn to_history_view(&mut self) {
        self.save_previous_state();
        self.input_mode = Normal;
        self.view_mode = HistoryFolderView;
    }
    pub fn to_file_view(&mut self) {
        self.save_previous_state();
        self.input_mode = Normal;
        self.view_mode = FileView;
    }

    fn save_previous_state(&mut self) {
        self.prev_input_mode = self.input_mode.clone();
        self.prev_view_mode = self.view_mode.clone();
    }
    pub fn restore_previous_state(&mut self) {
        self.input_mode = self.prev_input_mode.clone();
        self.view_mode = self.prev_view_mode.clone();
    }
}
