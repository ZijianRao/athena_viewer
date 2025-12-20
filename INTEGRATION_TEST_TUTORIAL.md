# Athena Viewer Integration Testing Tutorial

This tutorial provides a comprehensive guide for writing integration tests for the Athena Viewer TUI application. You'll learn how to test the complete user workflows from directory navigation to file viewing.

## Overview

**Project**: Athena Viewer - Terminal-based file viewer with syntax highlighting
**Framework**: Rust + Ratatui + Crossterm
**Testing Approach**: Integration tests with mock filesystem and simulated user input

## Table of Contents

1. [Test Architecture](#test-architecture)
2. [Setup & Configuration](#setup--configuration)
3. [Test Infrastructure](#test-infrastructure)
4. [Core Test Scenarios](#core-test-scenarios)
5. [Step-by-Step Implementation](#step-by-step-implementation)
6. [Advanced Testing Patterns](#advanced-testing-patterns)
7. [Best Practices](#best-practices)

## Test Architecture

### What We're Testing

The Athena Viewer has these key components that need integration testing:

```
User Input → Event Handler → State Machine → Message Holder → Terminal Output
```

**State Modes**:
- `InputMode`: Normal vs Edit
- `ViewMode`: Search, FileView, HistoryFolderView

**User Workflows**:
1. Browse directories → Select files → View content
2. Search/filter files → Navigate results
3. Mode switching (Normal ↔ Edit)
4. File operations (scroll, delete, refresh)

### Test Pyramid for TUI Apps

```
Integration Tests (80%) ← Your focus here
    ↓
Unit Tests (15%) ← Test pure functions like get_highlight_index()
    ↓
E2E Tests (5%) ← Full terminal simulation (optional)
```

## Setup & Configuration

### 1. Directory Structure

Create this structure in your project root:

```
athena_viewer/
├── src/                    # Existing source code
├── tests/                  # NEW: Integration tests
│   ├── fixtures/          # Test data files
│   ├── integration/       # Actual test files
│   └── utils/             # Test utilities
├── Cargo.toml
└── INTEGRATION_TEST_TUTORIAL.md
```

### 2. Cargo.toml Configuration

Your current `Cargo.toml` already has good start:

```toml
[dependencies]
crossterm = "0.29"
ratatui = "0.29"
tui-input = "0.14"
chrono = { version = "0.4", features = ["serde"] }
lru = "0.16"
syntect = "5.3"

[dev-dependencies]
tempfile = "3.23"  # ✅ Already present - perfect for test fixtures
```

**Optional additions for advanced testing**:

```toml
[dev-dependencies]
tempfile = "3.23"
mockall = "0.13"      # For mocking if needed
assert_cmd = "2.0"    # For CLI testing
predicates = "3.0"    # For assertions
```

## Test Infrastructure

### 1. Mock Filesystem Setup

Create `tests/utils/filesystem.rs`:

```rust
use tempfile::TempDir;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Creates a temporary directory with test files and directories
pub struct TestFilesystem {
    pub temp_dir: TempDir,
    pub root_path: PathBuf,
}

impl TestFilesystem {
    pub fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path().to_path_buf();

        Self { temp_dir, root_path }
    }

    /// Creates a file with content
    pub fn create_file(&self, path: &str, content: &str) -> PathBuf {
        let full_path = self.root_path.join(path);
        fs::create_dir_all(full_path.parent().unwrap()).unwrap();
        let mut file = File::create(&full_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        full_path
    }

    /// Creates a directory
    pub fn create_dir(&self, path: &str) -> PathBuf {
        let full_path = self.root_path.join(path);
        fs::create_dir_all(&full_path).unwrap();
        full_path
    }

    /// Creates nested directory structure for navigation tests
    pub fn create_nested_structure(&self) {
        // Root files
        self.create_file("README.md", "# Test Project\nThis is a readme.");
        self.create_file("main.rs", "fn main() { println!(\"hello\"); }");

        // Nested directories
        self.create_dir("src");
        self.create_file("src/lib.rs", "pub fn helper() {}");
        self.create_file("src/module.rs", "mod tests { /* ... */ }");

        // Deep nesting
        self.create_dir("src/nested/deep");
        self.create_file("src/nested/deep/file.txt", "deep content");

        // Empty directory
        self.create_dir("empty");
    }

    /// Get current directory path
    pub fn path(&self) -> &Path {
        &self.root_path
    }
}
```

### 2. Mock Terminal & Event Simulation

Create `tests/utils/mock_terminal.rs`:

```rust
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::io;

/// Creates a mock terminal for testing without actual TTY
pub fn create_test_terminal() -> Terminal<TestBackend> {
    let backend = TestBackend::new(80, 24); // Standard terminal size
    Terminal::new(backend).unwrap()
}

/// Helper to create key events
pub mod events {
    use super::*;

    pub fn key(code: KeyCode) -> Event {
        Event::Key(KeyEvent::new(code, KeyModifiers::empty()))
    }

    pub fn key_with_modifiers(code: KeyCode, modifiers: KeyModifiers) -> Event {
        Event::Key(KeyEvent::new(code, modifiers))
    }

    // Common key combinations
    pub fn ctrl_c() -> Event { key_with_modifiers(KeyCode::Char('c'), KeyModifiers::CONTROL) }
    pub fn ctrl_d() -> Event { key_with_modifiers(KeyCode::Char('d'), KeyModifiers::CONTROL) }
    pub fn ctrl_k() -> Event { key_with_modifiers(KeyCode::char('k'), KeyModifiers::CONTROL) }
    pub fn ctrl_z() -> Event { key_with_modifiers(KeyCode::char('z'), KeyModifiers::CONTROL) }
    pub fn tab() -> Event { key(KeyCode::Tab) }
    pub fn enter() -> Event { key(KeyCode::Enter) }
    pub fn escape() -> Event { key(KeyCode::Esc) }

    // Navigation
    pub fn down() -> Event { key(KeyCode::Down) }
    pub fn up() -> Event { key(KeyCode::Up) }
    pub fn left() -> Event { key(KeyCode::Left) }
    pub fn right() -> Event { key(KeyCode::Right) }
    pub fn page_down() -> Event { key(KeyCode::PageDown) }
    pub fn page_up() -> Event { key(KeyCode::PageUp) }
    pub fn home() -> Event { key(KeyCode::Home) }
    pub fn end() -> Event { key(KeyCode::End) }

    // Character keys
    pub fn char(c: char) -> Event { key(KeyCode::Char(c)) }
}
```

### 3. Test App Builder

Create `tests/utils/mock_app.rs`:

```rust
use athena_viewer::app::App;
use athena_viewer::state_holder::state_holder::{InputMode, ViewMode};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::Event;
use std::path::PathBuf;

/// Test wrapper for the App that provides testing utilities
pub struct TestApp {
    pub app: App,
    pub terminal: Terminal<TestBackend>,
}

impl TestApp {
    /// Create a new test app starting in a specific directory
    pub fn new(start_dir: PathBuf) -> Self {
        // Change to test directory
        std::env::set_current_dir(&start_dir).unwrap();

        let terminal = super::mock_terminal::create_test_terminal();
        let app = App::new();

        Self { app, terminal }
    }

    /// Send an event to the app and process it
    pub fn send_event(&mut self, event: Event) {
        // Simulate the event handling that happens in the main loop
        // We need to manually call the appropriate handler based on current state

        let input_mode = self.app.state_holder.borrow().input_mode;
        let view_mode = self.app.state_holder.borrow().view_mode;

        use InputMode::*;
        use ViewMode::*;

        match (input_mode, view_mode) {
            (Normal, Search) => self.app.handle_normal_search_event(event),
            (Normal, FileView) => self.app.handle_normal_file_view_event(event),
            (Edit, HistoryFolderView) => self.app.handle_edit_history_folder_view_event(event),
            (Edit, Search) => self.app.handle_edit_search_event(event),
            _ => (),
        }
    }

    /// Send a sequence of events
    pub fn send_events(&mut self, events: Vec<Event>) {
        for event in events {
            self.send_event(event);
        }
    }

    /// Get current input mode
    pub fn get_input_mode(&self) -> InputMode {
        self.app.state_holder.borrow().input_mode
    }

    /// Get current view mode
    pub fn get_view_mode(&self) -> ViewMode {
        self.app.state_holder.borrow().view_mode
    }

    /// Check if in specific modes
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

    /// Get current file opened (if any)
    pub fn get_opened_file(&self) -> Option<PathBuf> {
        self.app.message_holder.file_opened.clone()
    }

    /// Get current directory from message holder
    pub fn get_current_directory(&self) -> PathBuf {
        self.app.message_holder.folder_holder.current_directory.clone()
    }

    /// Get current search/filter input
    pub fn get_search_input(&self) -> String {
        self.app.input.value().to_string()
    }

    /// Get list of visible files/folders (for assertions)
    pub fn get_visible_items(&self) -> Vec<String> {
        self.app.message_holder.folder_holder.selected_path_holder
            .iter()
            .map(|entry| entry.relative_to(&self.app.message_holder.folder_holder.current_directory))
            .collect()
    }

    /// Get scroll positions
    pub fn get_scroll_positions(&self) -> (usize, usize) {
        (self.app.message_holder.vertical_scroll, self.app.message_holder.horizontal_scroll)
    }

    /// Render the current frame (useful for debugging)
    pub fn render_frame(&mut self) {
        let _ = self.terminal.draw(|frame| self.app.draw(frame));
    }
}
```

## Core Test Scenarios

### Scenario 1: Directory Navigation

**User Flow**: Browse → Navigate → Select → View

**Test File**: `tests/integration/navigation.rs`

```rust
#[cfg(test)]
mod navigation_tests {
    use super::utils::*;
    use athena_viewer::state_holder::state_holder::{InputMode, ViewMode};
    use ratatui::crossterm::event::KeyCode;

    #[test]
    fn test_browse_directory_and_select_file() {
        // Setup: Create test filesystem
        let fs = TestFilesystem::new();
        fs.create_nested_structure();

        // Create app in test directory
        let mut app = TestApp::new(fs.path().to_path_buf());

        // Verify initial state
        assert!(app.is_normal_mode());
        assert!(app.is_search_view());
        assert_eq!(app.get_visible_items().len(), 5); // README.md, main.rs, src/, empty/, .gitkeep

        // Navigate down to select 'src/' directory
        app.send_event(events::down());
        app.send_event(events::down());

        // Verify selection moved
        assert_eq!(app.get_visible_items().len(), 5);

        // Enter directory
        app.send_event(events::enter());

        // Should now be in src/ directory
        assert!(app.get_current_directory().ends_with("src"));
        assert_eq!(app.get_visible_items().len(), 3); // lib.rs, module.rs, nested/

        // Navigate to nested directory and enter
        app.send_event(events::down()); // select nested/
        app.send_event(events::enter());

        // Should be in nested directory
        assert!(app.get_current_directory().ends_with("nested"));

        // Navigate to deep directory and enter
        app.send_event(events::down()); // select deep/
        app.send_event(events::enter());

        // Should be in deep directory
        assert!(app.get_current_directory().ends_with("deep"));

        // Select and open file
        app.send_event(events::enter()); // select file.txt

        // Should now be in FileView mode
        assert!(app.is_file_view());
        assert!(app.get_opened_file().is_some());
        assert!(app.get_opened_file().unwrap().ends_with("file.txt"));
    }

    #[test]
    fn test_navigate_to_parent_directory() {
        let fs = TestFilesystem::new();
        fs.create_nested_structure();

        let mut app = TestApp::new(fs.path().to_path_buf());

        // Navigate into src/
        app.send_events(vec![events::down(), events::enter()]);
        assert!(app.get_current_directory().ends_with("src"));

        // Use Ctrl+K to go to parent
        app.send_event(events::ctrl_k());

        // Should be back in root
        assert_eq!(app.get_current_directory(), fs.path());
    }

    #[test]
    fn test_empty_directory_handling() {
        let fs = TestFilesystem::new();
        fs.create_dir("empty");

        let mut app = TestApp::new(fs.path().to_path_buf());

        // Navigate to empty directory
        app.send_events(vec![events::down(), events::down()]); // skip README.md, main.rs
        app.send_event(events::enter()); // select empty/

        // Should be in empty directory with no items
        assert!(app.get_current_directory().ends_with("empty"));
        assert_eq!(app.get_visible_items().len(), 0);

        // Should still be able to go back
        app.send_event(events::ctrl_k());
        assert_eq!(app.get_current_directory(), fs.path());
    }
}
```

### Scenario 2: Search Functionality

**User Flow**: Enter Edit Mode → Type Search → Filter Results → Navigate

**Test File**: `tests/integration/search.rs`

```rust
#[cfg(test)]
mod search_tests {
    use super::utils::*;
    use athena_viewer::state_holder::state_holder::{InputMode, ViewMode};
    use ratatui::crossterm::event::{Event, KeyCode};

    #[test]
    fn test_basic_search_filtering() {
        let fs = TestFilesystem::new();
        fs.create_nested_structure();

        let mut app = TestApp::new(fs.path().to_path_buf());

        // Switch to edit mode (Tab)
        app.send_event(events::tab());
        assert!(app.is_edit_mode());
        assert!(app.is_search_view());

        // Type search term "rs"
        app.send_event(Event::Key(
            ratatui::crossterm::event::KeyEvent::new(
                KeyCode::Char('r'),
                ratatui::crossterm::event::KeyModifiers::empty()
            )
        ));
        app.send_event(Event::Key(
            ratatui::crossterm::event::KeyEvent::new(
                KeyCode::Char('s'),
                ratatui::crossterm::event::KeyModifiers::empty()
            )
        ));

        // Verify search input
        assert_eq!(app.get_search_input(), "rs");

        // Verify filtered results (should show main.rs, lib.rs, module.rs)
        let items = app.get_visible_items();
        assert!(items.len() <= 3);
        assert!(items.iter().all(|s| s.contains("rs")));

        // Switch back to normal mode
        app.send_event(events::tab());
        assert!(app.is_normal_mode());

        // Search results should remain
        let items_after = app.get_visible_items();
        assert_eq!(items, items_after);
    }

    #[test]
    fn test_search_with_expand_collapse() {
        let fs = TestFilesystem::new();
        fs.create_nested_structure();

        let mut app = TestApp::new(fs.path().to_path_buf());

        // Enter edit mode and search for "nested"
        app.send_event(events::tab());
        app.send_events(vec![
            Event::Key(ratatui::crossterm::event::KeyEvent::new(
                KeyCode::Char('n'),
                ratatui::crossterm::event::KeyModifiers::empty()
            )),
            Event::Key(ratatui::crossterm::event::KeyEvent::new(
                KeyCode::Char('e'),
                ratatui::crossterm::event::KeyModifiers::empty()
            )),
        ]);

        // Should show nested directory
        let items = app.get_visible_items();
        assert!(items.iter().any(|s| s.contains("nested")));

        // Expand nested directory (E key)
        app.send_event(events::char('e'));

        // Should now show contents of nested
        let expanded_items = app.get_visible_items();
        assert!(expanded_items.len() > items.len());
        assert!(expanded_items.iter().any(|s| s.contains("deep")));

        // Collapse (C key)
        app.send_event(events::char('c'));

        // Should be back to filtered view
        let collapsed_items = app.get_visible_items();
        assert_eq!(collapsed_items.len(), items.len());
    }

    #[test]
    fn test_search_delete_file() {
        let fs = TestFilesystem::new();
        fs.create_nested_structure();

        let mut app = TestApp::new(fs.path().to_path_buf());

        // Navigate to a file
        app.send_event(events::down()); // select README.md

        // Delete with Ctrl+D
        app.send_event(events::ctrl_d());

        // File should be gone
        let items = app.get_visible_items();
        assert!(!items.iter().any(|s| s.contains("README.md")));

        // Verify file is actually deleted from filesystem
        assert!(!fs.path().join("README.md").exists());
    }

    #[test]
    fn test_search_clear_and_reset() {
        let fs = TestFilesystem::new();
        fs.create_nested_structure();

        let mut app = TestApp::new(fs.path().to_path_buf());

        // Enter edit mode and type search
        app.send_event(events::tab());
        app.send_events(vec![
            Event::Key(ratatui::crossterm::event::KeyEvent::new(
                KeyCode::Char('x'),
                ratatui::crossterm::event::KeyModifiers::empty()
            )),
        ]);

        assert_eq!(app.get_search_input(), "x");

        // Clear with Ctrl+C
        app.send_event(events::ctrl_c());

        // Should be empty and show all files
        assert_eq!(app.get_search_input(), "");
        assert_eq!(app.get_visible_items().len(), 5); // All files back
    }
}
```

### Scenario 3: State Transitions

**User Flow**: Mode switching and state restoration

**Test File**: `tests/integration/state_transitions.rs`

```rust
#[cfg(test)]
mod state_transition_tests {
    use super::utils::*;
    use athena_viewer::state_holder::state_holder::{InputMode, ViewMode};

    #[test]
    fn test_normal_edit_mode_switching() {
        let fs = TestFilesystem::new();
        let mut app = TestApp::new(fs.path().to_path_buf());

        // Start in Normal/Search
        assert!(app.is_normal_mode());
        assert!(app.is_search_view());

        // Tab to Edit/Search
        app.send_event(events::tab());
        assert!(app.is_edit_mode());
        assert!(app.is_search_view());

        // Tab back to Normal/Search
        app.send_event(events::tab());
        assert!(app.is_normal_mode());
        assert!(app.is_search_view());
    }

    #[test]
    fn test_file_view_state_restoration() {
        let fs = TestFilesystem::new();
        fs.create_file("test.txt", "content");

        let mut app = TestApp::new(fs.path().to_path_buf());

        // Open file to enter FileView
        app.send_event(events::enter()); // select test.txt

        assert!(app.is_file_view());

        // Press Q to quit and restore previous state
        app.send_event(events::char('q'));

        // Should be back to Normal/Search
        assert!(app.is_normal_mode());
        assert!(app.is_search_view());
    }

    #[test]
    fn test_history_search_transition() {
        let fs = TestFilesystem::new();
        fs.create_nested_structure();

        let mut app = TestApp::new(fs.path().to_path_buf());

        // Start in Normal/Search
        assert!(app.is_normal_mode());
        assert!(app.is_search_view());

        // Press H to go to History Search
        app.send_event(events::char('h'));

        assert!(app.is_edit_mode());
        assert!(app.is_history_view());

        // Should show history items
        let items = app.get_visible_items();
        assert!(items.len() > 0);

        // Tab should go to Normal/History
        app.send_event(events::tab());
        assert!(app.is_normal_mode());
        assert!(app.is_history_view());
    }

    #[test]
    fn test_nested_state_transitions() {
        let fs = TestFilesystem::new();
        fs.create_nested_structure();

        let mut app = TestApp::new(fs.path().to_path_buf());

        // Normal Search → Edit Search
        app.send_event(events::tab());
        assert!(app.is_edit_mode());

        // Edit Search → Normal Search
        app.send_event(events::tab());
        assert!(app.is_normal_mode());

        // Normal Search → History (Edit)
        app.send_event(events::char('h'));
        assert!(app.is_edit_mode());
        assert!(app.is_history_view());

        // History Edit → History Normal
        app.send_event(events::tab());
        assert!(app.is_normal_mode());
        assert!(app.is_history_view());

        // History Normal → Back to Search
        app.send_event(events::char('h')); // H from normal history goes back
        assert!(app.is_normal_mode());
        assert!(app.is_search_view());
    }
}
```

### Scenario 4: File Viewing & Scrolling

**User Flow**: Open file → Navigate content → Scroll → Exit

**Test File**: `tests/integration/file_operations.rs`

```rust
#[cfg(test)]
mod file_operations_tests {
    use super::utils::*;

    #[test]
    fn test_open_and_view_file() {
        let fs = TestFilesystem::new();
        let content = "line 1\nline 2\nline 3\nline 4\nline 5";
        fs.create_file("test.txt", content);

        let mut app = TestApp::new(fs.path().to_path_buf());

        // Open file
        app.send_event(events::enter());

        assert!(app.is_file_view());
        assert!(app.get_opened_file().is_some());

        // Verify file content is loaded (check through message_holder)
        let file_info = &app.app.message_holder.file_text_info;
        assert!(file_info.is_some());

        let info = file_info.as_ref().unwrap();
        assert_eq!(info.n_rows, 5); // 5 lines
        assert!(info.max_line_length > 0);
    }

    #[test]
    fn test_vertical_scrolling() {
        let fs = TestFilesystem::new();
        let content = (1..=100).map(|i| format!("line {}", i)).collect::<Vec<_>>().join("\n");
        fs.create_file("long.txt", content);

        let mut app = TestApp::new(fs.path().to_path_buf());
        app.send_event(events::enter()); // Open file

        // Initial scroll position
        let (v_scroll, _) = app.get_scroll_positions();
        assert_eq!(v_scroll, 0);

        // Scroll down with j
        app.send_event(events::char('j'));
        let (v_scroll, _) = app.get_scroll_positions();
        assert_eq!(v_scroll, 1);

        // Scroll down with Down arrow
        app.send_event(events::down());
        let (v_scroll, _) = app.get_scroll_positions();
        assert_eq!(v_scroll, 2);

        // Page down
        app.send_event(events::page_down());
        let (v_scroll, _) = app.get_scroll_positions();
        assert_eq!(v_scroll, 32); // 2 + 30

        // Scroll up with k
        app.send_event(events::char('k'));
        let (v_scroll, _) = app.get_scroll_positions();
        assert_eq!(v_scroll, 31);

        // Page up
        app.send_event(events::page_up());
        let (v_scroll, _) = app.get_scroll_positions();
        assert_eq!(v_scroll, 1);
    }

    #[test]
    fn test_horizontal_scrolling() {
        let fs = TestFilesystem::new();
        let long_line = "x".repeat(200);
        fs.create_file("wide.txt", &long_line);

        let mut app = TestApp::new(fs.path().to_path_buf());
        app.send_event(events::enter()); // Open file

        // Initial scroll
        let (_, h_scroll) = app.get_scroll_positions();
        assert_eq!(h_scroll, 0);

        // Scroll right with l
        app.send_event(events::char('l'));
        let (_, h_scroll) = app.get_scroll_positions();
        assert_eq!(h_scroll, 1);

        // Scroll right with Right arrow
        app.send_event(events::right());
        let (_, h_scroll) = app.get_scroll_positions();
        assert_eq!(h_scroll, 2);

        // Home key
        app.send_event(events::home());
        let (_, h_scroll) = app.get_scroll_positions();
        assert_eq!(h_scroll, 0);

        // End key (should scroll to end minus some padding)
        app.send_event(events::end());
        let (_, h_scroll) = app.get_scroll_positions();
        assert!(h_scroll > 150); // Near end of 200-char line
    }

    #[test]
    fn test_file_view_exit() {
        let fs = TestFilesystem::new();
        fs.create_file("test.txt", "content");

        let mut app = TestApp::new(fs.path().to_path_buf());
        app.send_event(events::enter()); // Open file

        assert!(app.is_file_view());
        assert!(app.get_opened_file().is_some());

        // Press Q to quit file view
        app.send_event(events::char('q'));

        // Should be back to search view
        assert!(app.is_search_view());
        assert!(app.get_opened_file().is_none());
    }
}
```

## Advanced Testing Patterns

### 1. Testing with Large Files

```rust
#[test]
fn test_large_file_performance() {
    use std::time::Instant;

    let fs = TestFilesystem::new();

    // Create a large file (1000 lines)
    let content: String = (1..=1000)
        .map(|i| format!("Line {}: Some content here\n", i))
        .collect();
    fs.create_file("large.txt", &content);

    let mut app = TestApp::new(fs.path().to_path_buf());

    // Measure opening time
    let start = Instant::now();
    app.send_event(events::enter());
    let duration = start.elapsed();

    // Should complete in reasonable time (< 100ms)
    assert!(duration.as_millis() < 100);

    // Verify file info loaded correctly
    let info = app.app.message_holder.file_text_info.as_ref().unwrap();
    assert_eq!(info.n_rows, 1000);
}
```

### 2. Testing Error Scenarios

```rust
#[test]
fn test_permission_denied() {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    let fs = TestFilesystem::new();
    let file = fs.create_file("readonly.txt", "content");

    // Make file read-only
    let mut perms = fs::metadata(&file).unwrap().permissions();
    perms.set_mode(0o444); // Read-only
    fs::set_permissions(&file, perms).unwrap();

    let mut app = TestApp::new(fs.path().to_path_buf());

    // Try to delete (should fail gracefully)
    app.send_event(events::down()); // Select file
    app.send_event(events::ctrl_d()); // Delete

    // File should still exist
    assert!(file.exists());
}
```

### 3. Testing Edge Cases

```rust
#[test]
fn test_unicode_file_names() {
    let fs = TestFilesystem::new();
    fs.create_file("文件.txt", "中文内容");
    fs.create_file("café.rs", "fn café() {}");
    fs.create_file("emoji_😀.txt", "emoji filename");

    let mut app = TestApp::new(fs.path().to_path_buf());

    let items = app.get_visible_items();
    assert!(items.iter().any(|s| s.contains("文件.txt")));
    assert!(items.iter().any(|s| s.contains("café.rs")));
    assert!(items.iter().any(|s| s.contains("emoji_😀.txt")));
}

#[test]
fn test_special_characters_in_search() {
    let fs = TestFilesystem::new();
    fs.create_file("test-file.rs", "");
    fs.create_file("test_file.rs", "");
    fs.create_file("test file.rs", "");

    let mut app = TestApp::new(fs.path().to_path_buf());

    // Search with hyphen
    app.send_event(events::tab());
    app.send_events(vec![
        Event::Key(ratatui::crossterm::event::KeyEvent::new(
            KeyCode::Char('-'),
            ratatui::crossterm::event::KeyModifiers::empty()
        )),
    ]);

    let items = app.get_visible_items();
    assert_eq!(items.len(), 1);
    assert!(items[0].contains("test-file"));
}
```

### 4. Integration Test Utilities

Create `tests/utils/mod.rs`:

```rust
pub mod filesystem;
pub mod mock_app;
pub mod mock_terminal;

pub use filesystem::TestFilesystem;
pub use mock_app::TestApp;
pub use mock_terminal::{create_test_terminal, events};
```

### 5. Main Integration Test Module

Create `tests/integration/mod.rs`:

```rust
pub mod file_operations;
pub mod navigation;
pub mod search;
pub mod state_transitions;
```

## Running Tests

### Basic Commands

```bash
# Run all integration tests
cargo test --test integration

# Run specific test module
cargo test --test integration navigation

# Run specific test
cargo test --test integration navigation_tests::test_browse_directory_and_select_file

# Run with output
cargo test --test integration -- --nocapture

# Run with filtering
cargo test --test integration -- --test-threads=1
```

### Test Organization

```bash
tests/
├── integration/           # All integration tests
│   ├── mod.rs            # Module declarations
│   ├── navigation.rs     # Directory navigation tests
│   ├── search.rs         # Search functionality tests
│   ├── state_transitions.rs  # Mode switching tests
│   └── file_operations.rs    # File viewing tests
├── utils/                # Test utilities
│   ├── mod.rs
│   ├── filesystem.rs
│   ├── mock_app.rs
│   └── mock_terminal.rs
└── fixtures/             # Optional: static test files
    └── sample_code.rs
```

### Continuous Integration

Create `.github/workflows/test.yml`:

```yaml
name: Integration Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v3

    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable

    - name: Run integration tests
      run: cargo test --test integration

    - name: Run with coverage
      run: |
        cargo install cargo-tarpaulin
        cargo tarpaulin --out Html --output-dir ./coverage
```

## Best Practices

### 1. Test Naming
```rust
// ✅ Good: Descriptive, what it tests
#[test]
fn test_navigate_to_parent_directory_with_ctrl_k()

// ❌ Bad: Vague
#[test]
fn test_navigation()
```

### 2. Test Isolation
```rust
// ✅ Each test creates its own temp directory
#[test]
fn test_1() {
    let fs = TestFilesystem::new();  // Fresh directory
    // ...
}

#[test]
fn test_2() {
    let fs = TestFilesystem::new();  // Fresh directory
    // ...
}
```

### 3. Assert Early and Often
```rust
// ✅ Check state at each step
app.send_event(events::down());
assert!(app.is_normal_mode());  // Verify mode

app.send_event(events::enter());
assert!(app.is_file_view());    // Verify transition
assert!(app.get_opened_file().is_some());  // Verify result
```

### 4. Clean Assertions
```rust
// ✅ Use descriptive assertions
assert_eq!(app.get_visible_items().len(), 5, "Should show 5 items initially");

// ❌ Avoid magic numbers
assert_eq!(app.get_visible_items().len(), 5);
```

### 5. Test One Thing
```rust
// ✅ Focused test
#[test]
fn test_file_view_exit() {
    // Only tests Q key exit behavior
}

// ❌ Multiple behaviors
#[test]
fn test_file_view() {
    // Tests opening, scrolling, exiting all together
}
```

## Debugging Tests

### 1. Print Debug Info
```rust
#[test]
fn test_debug() {
    let mut app = TestApp::new(fs.path().to_path_buf());

    // Debug state before
    println!("Before: {:?}", app.get_visible_items());

    app.send_event(events::down());

    // Debug state after
    println!("After: {:?}", app.get_visible_items());

    // Render frame to see visual output
    app.render_frame();

    assert!(...);
}
```

### 2. Run Single Test
```bash
cargo test --test integration test_browse_directory_and_select_file -- --nocapture
```

### 3. Use `dbg!` Macro
```rust
let items = app.get_visible_items();
dbg!(&items);  // Prints debug info
assert_eq!(items.len(), 5);
```

## Common Pitfalls

### 1. Forgetting to Reset State
```rust
// ❌ Bad: Test pollution
#[test]
fn test_1() { /* modifies global state */ }
#[test]
fn test_2() { /* might see test_1's changes */ }

// ✅ Good: Each test isolated
#[test]
fn test_1() {
    let fs = TestFilesystem::new();  // Fresh
    // ...
}
```

### 2. Not Testing Edge Cases
```rust
// ✅ Test empty, large, special cases
#[test]
fn test_empty_directory() { /* ... */ }
#[test]
fn test_large_file() { /* ... */ }
#[test]
fn test_unicode_names() { /* ... */ }
```

### 3. Over-mocking
```rust
// ❌ Don't mock what you can test
// Use real App, real filesystem, real events

// ✅ Only mock external dependencies (like terminal TTY)
```

## Next Steps

1. **Start Simple**: Begin with navigation tests (Scenario 1)
2. **Build Infrastructure**: Create `TestFilesystem` and `TestApp`
3. **Add Tests Incrementally**: One scenario at a time
4. **Refactor**: Extract common patterns into utilities
5. **Document**: Add comments explaining what each test does

## Resources

- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-01-writing-tests.html)
- [tempfile crate](https://docs.rs/tempfile/latest/tempfile/)
- [Ratatui Testing Guide](https://ratatui.rs/testing/)
- [Crossterm Event Types](https://docs.rs/crossterm/latest/crossterm/event/enum.Event.html)

---

**Happy Testing!** 🎯

This tutorial gives you everything needed to build a comprehensive integration test suite for Athena Viewer. Start with the basic infrastructure and add tests incrementally.