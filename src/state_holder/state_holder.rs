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
}
