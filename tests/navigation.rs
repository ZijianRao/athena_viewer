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
        let mut app = TestApp::new(fs.path().to_path_buf()).unwrap();
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
        ])
        .unwrap();
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
        ])
        .unwrap();
        // check filter is effective
        assert_eq!(app.get_visible_items(), vec!["lib.rs"]);
        // file view mode with lib.rs
        assert!(app.is_file_view());
        assert!(app.get_opened_file().is_some());
        assert!(app.get_opened_file().unwrap().ends_with("lib.rs"));

        app.send_event(events::char('q')).unwrap();
        // check filter is still effective
        assert_eq!(app.get_visible_items(), vec!["lib.rs"]);
    }

    #[test]
    fn test_navigate_to_parent_directory() {
        // setup: create test filesystem
        let fs = TestFileSystem::new();
        fs.create_nested_structure();

        // create app in test directory
        let mut app = TestApp::new(fs.path().to_path_buf()).unwrap();
        assert_eq!(app.get_current_directory(), fs.path());

        // navigate down to and enter 'src/' directory
        app.send_events(vec![
            events::char('s'),
            events::char('r'),
            events::char('c'),
            events::enter(),
        ])
        .unwrap();
        assert!(
            app.get_current_directory().ends_with("src"),
            "{}",
            app.get_current_directory().display()
        );

        app.send_event(events::tab()).unwrap();
        assert!(app.is_normal_mode());
        app.send_event(events::ctrl_k()).unwrap();
        assert_eq!(app.get_current_directory(), fs.path());

        // test when the folder list is filtered case
        app.send_event(events::tab()).unwrap();
        assert!(app.is_edit_mode());
        app.send_events(vec![
            events::char('s'),
            events::char('r'),
            events::char('c'),
            events::enter(),
            events::char('l'),
        ])
        .unwrap();
        app.send_events(vec![events::tab(), events::ctrl_k()])
            .unwrap();
        assert_eq!(app.get_current_directory(), fs.path());
    }

    #[test]
    fn test_browse_directory_permission_error() {
        use athena_viewer::app::app_error::AppError;
        use std::fs;
        use std::os::unix::fs::PermissionsExt;
        // setup: create test filesystem
        let fs = TestFileSystem::new();
        fs.create_nested_structure();
        let no_permission_folder_name = "no_permission";
        let no_permission_path = fs.create_dir(no_permission_folder_name);
        fs::set_permissions(
            no_permission_path.clone(),
            fs::Permissions::from_mode(0o111),
        )
        .unwrap();

        // create app in test directory
        let mut app = TestApp::new(fs.path().to_path_buf()).unwrap();
        assert_eq!(app.get_current_directory(), fs.path());

        // verify initial state
        assert!(app.is_edit_mode());
        assert!(app.is_search_view());

        let mut visible_items = vec![
            "..",
            "README.md",
            "main.rs",
            "src",
            "empty",
            ".gitkeep",
            no_permission_folder_name,
        ];
        visible_items.sort();

        assert_eq!(app.get_visible_items(), visible_items);
        // navigate down to and enter 'src/' directory
        let result = app.send_events(vec![
            events::char('n'),
            events::char('o'),
            events::char('_'),
            events::char('p'),
            events::enter(),
        ]);
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, AppError::Parse(_)));
        }
        fs::set_permissions(no_permission_path, fs::Permissions::from_mode(0o755)).unwrap();
        assert_eq!(app.get_current_directory(), fs.path().to_path_buf());
    }

    #[test]
    fn test_browse_directory_traveral_parent_check() {
        use athena_viewer::app::app_error::AppError;

        // setup: create test filesystem
        let fs = TestFileSystem::new();
        fs.create_nested_structure();

        // create app in test directory
        let mut app = TestApp::new(fs.path().to_path_buf()).unwrap();
        assert_eq!(app.get_current_directory(), fs.path());
        let result = app.send_events(vec![events::tab(), events::ctrl_k()]);
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, AppError::Path(_)));
        }
    }

    #[test]
    fn test_folder_expand() {
        // setup: create test filesystem
        let fs = TestFileSystem::new();
        fs.create_nested_structure();

        // create app in test directory
        let mut app = TestApp::new(fs.path().to_path_buf()).unwrap();
        assert_eq!(app.get_current_directory(), fs.path());

        app.send_event(events::tab()).unwrap();
        assert!(app.is_normal_mode());
        app.send_event(events::char('e')).unwrap();

        let mut visible_items_exp = vec![
            "..",
            ".gitkeep",
            "README.md",
            "main.rs",
            "src/lib.rs",
            "src/module.rs",
            "src/nested",
        ];
        visible_items_exp.sort();
        let visible_items_act = app.get_visible_items();

        assert_eq!(visible_items_act, visible_items_exp);
        // navigate down to and enter 'src/' directory
    }
}
