pub mod utils;
#[cfg(test)]
mod navigation_tests {
    use super::utils::*;
    use crate::utils::TestFileSystem;

    /// only test the backend, no ui involved
    #[test]
    fn test_browse_directory_and_select_file() {
        // setup: create test filesystem
        let fs = TestFileSystem::new();
        fs.create_nested_structure();

        // create app in test directory
        let mut app = TestApp::new(fs.path().to_path_buf());
        assert_eq!(app.get_current_directory(), fs.path());

        // verify initial state
        assert!(app.is_edit_mode());
        assert!(app.is_search_view());

        let mut visible_items = vec!["..", "README.md", "main.rs", "src", "empty", ".gitkeep"];
        visible_items.sort();

        assert_eq!(app.get_visible_items(), visible_items);
        // navigate down to and enter 'src/' directory
        app.send_events(vec![
            events::char('s'),
            events::char('r'),
            events::char('c'),
            events::enter(),
        ]);
        assert!(
            app.get_current_directory().ends_with("src"),
            "{}",
            app.get_current_directory().display()
        );
        let mut visible_items = vec!["..", "lib.rs", "module.rs", "nested"];
        visible_items.sort();
        assert_eq!(app.get_visible_items(), visible_items); // lib.rs, module.rs, nested/

        app.send_events(vec![
            events::char('l'),
            events::char('i'),
            events::char('b'),
            events::char('.'),
            events::char('r'),
            events::char('s'),
            events::enter(),
        ]);
        // check filter is effective
        assert_eq!(app.get_visible_items(), vec!["lib.rs"]);
        // file view mode with lib.rs
        assert!(app.is_file_view());
        assert!(app.get_opened_file().is_some());
        assert!(app.get_opened_file().unwrap().ends_with("lib.rs"));

        app.send_event(events::char('q'));
        // check filter is still effective
        assert_eq!(app.get_visible_items(), vec!["lib.rs"]);
    }

    #[test]
    fn test_navigate_to_parent_directory() {
        // setup: create test filesystem
        let fs = TestFileSystem::new();
        fs.create_nested_structure();

        // create app in test directory
        let mut app = TestApp::new(fs.path().to_path_buf());
        assert_eq!(app.get_current_directory(), fs.path());

        // navigate down to and enter 'src/' directory
        app.send_events(vec![
            events::char('s'),
            events::char('r'),
            events::char('c'),
            events::enter(),
        ]);
        assert!(
            app.get_current_directory().ends_with("src"),
            "{}",
            app.get_current_directory().display()
        );

        app.send_event(events::tab());
        assert!(app.is_normal_mode());
        app.send_event(events::ctrl_k());
        assert_eq!(app.get_current_directory(), fs.path());
    }
}
