pub mod utils;
#[cfg(test)]
mod history_tests {
    use super::utils::*;
    use crate::utils::TestFileSystem;

    /// only test the backend, no ui involved
    #[test]
    fn test_history_navigation() {
        // setup: create test filesystem
        let fs = TestFileSystem::new();
        fs.create_nested_structure();

        // create app in test directory
        let mut app = TestApp::new(fs.path().to_path_buf());

        // add src folder into history
        app.send_events(vec![
            events::char('s'),
            events::char('r'),
            events::char('c'),
            events::enter(),
        ]);

        // add nested folder into history
        app.send_events(vec![
            events::char('n'),
            events::char('e'),
            events::char('s'),
            events::char('t'),
            events::char('e'),
            events::char('d'),
            events::enter(),
        ]);

        // add nested folder into history
        app.send_events(vec![
            events::char('d'),
            events::char('e'),
            events::char('e'),
            events::char('p'),
            events::enter(),
        ]);

        app.send_events(vec![events::tab(), events::char('h')]);
        assert!(app.is_history_view());
        let mut history = Vec::new();
        let mut expected_suffix = ["src/nested/deep", "src/nested", "src"];
        for s in expected_suffix.iter_mut() {
            let holder = format!("{}/{}", fs.path().display(), s);
            history.push(holder)
        }
        history.push(fs.path().to_str().unwrap().to_string());

        assert_eq!(app.get_visible_items(), history);

        app.send_events(vec![events::down(), events::down(), events::enter()]);
        let mut visible_items = vec!["..", "lib.rs", "module.rs", "nested"];
        visible_items.sort();
        assert_eq!(app.get_visible_items(), visible_items);
    }

    #[test]
    fn test_history_navigation_removed_handling() {
        // setup: create test filesystem
        let fs = TestFileSystem::new();
        fs.create_nested_structure();

        // create app in test directory
        let mut app = TestApp::new(fs.path().to_path_buf());

        // add src folder into history
        app.send_events(vec![
            events::char('s'),
            events::char('r'),
            events::char('c'),
            events::enter(),
        ]);

        // add nested folder into history
        app.send_events(vec![
            events::char('n'),
            events::char('e'),
            events::char('s'),
            events::char('t'),
            events::char('e'),
            events::char('d'),
            events::enter(),
        ]);

        app.send_events(vec![events::tab(), events::char('h')]);
        assert!(app.is_history_view());
        let mut history = Vec::new();

        let mut expected_suffix = ["src/nested", "src"];
        for s in expected_suffix.iter_mut() {
            let holder = format!("{}/{}", fs.path().display(), s);
            history.push(holder)
        }
        history.push(fs.path().to_str().unwrap().to_string());

        assert_eq!(app.get_visible_items(), history);
        history.clear();
        fs.remove_folder("src/nested"); // remove folder to handle error case
        let mut expected_suffix = ["src"];
        for s in expected_suffix.iter_mut() {
            let holder = format!("{}/{}", fs.path().display(), s);
            history.push(holder)
        }
        history.push(fs.path().to_str().unwrap().to_string());

        app.send_events(vec![events::enter()]);
        assert_eq!(app.get_visible_items(), history);
    }
}
