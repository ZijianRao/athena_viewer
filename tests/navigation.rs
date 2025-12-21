pub mod utils;
#[cfg(test)]
mod navigation_tests {
    use super::utils::*;
    use athena_viewer::state_holder::state_holder::{InputMode, ViewMode};

    use crate::utils::TestFileSystem;
    // use ratatui::crossterm::event::KeyCode;

    /// only test the backend, no ui involved
    #[test]
    fn test_browse_directory_and_select_file() {
        // setup: create test filesystem
        let fs = TestFileSystem::new();
        fs.create_nested_structure();

        // create app in test directory
        let mut app = TestApp::new(fs.path().to_path_buf());

        // verify initial state
        assert!(app.is_edit_mode());
        assert!(app.is_search_view());

        let mut visible_items = vec!["..", "README.md", "main.rs", "src", "empty", ".gitkeep"];
        visible_items.sort();

        assert_eq!(app.get_visible_items(), visible_items);
        // navigate down to and enter 'src/' directory
        app.send_event(events::char('s'));
        app.send_event(events::char('r'));
        app.send_event(events::char('c'));
        app.send_event(events::enter());
        assert!(
            app.get_current_directory().ends_with("src"),
            "{}",
            app.get_current_directory().display()
        );
        let mut visible_items = vec!["..", "lib.rs", "module.rs", "nested"];
        visible_items.sort();
        assert_eq!(app.get_visible_items(), visible_items); // lib.rs, module.rs, nested/
    }
}
