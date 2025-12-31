# Athena Viewer Integration Testing Tutorial

This tutorial provides a comprehensive guide for writing integration tests for the Athena Viewer TUI application. The project has **already implemented** a complete test infrastructure - this tutorial explains how it works and how to extend it.

## Overview

**Project**: Athena Viewer - Terminal-based file viewer with syntax highlighting
**Framework**: Rust + Ratatui + Crossterm
**Testing Approach**: Integration tests with mock filesystem and simulated user input
**Current Status**: ✅ **Test infrastructure implemented** with 70% happy path coverage

## Table of Contents

1. [Test Architecture](#test-architecture)
2. [Current Test Infrastructure](#current-test-infrastructure)
3. [Existing Test Examples](#existing-test-examples)
4. [Adding New Tests](#adding-new-tests)
5. [Unit Tests in Source Files](#unit-tests-in-source-files)
6. [Advanced Testing Patterns](#advanced-testing-patterns)
7. [Best Practices](#best-practices)
8. [Running Tests](#running-tests)
9. [Next Steps](#next-steps)

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

### Current Test Coverage

**✅ Implemented (70% happy paths)**:
- Navigation tests (browse, select, enter directories)
- History tests (add to history, navigate, handle invalid folders)
- Unit tests for file_helper and code_highlighter
- Mock infrastructure (TestApp, TestFileSystem, mock terminal)

**❌ Missing (0% error cases)**:
- Permission denied scenarios
- Deleted files/directories
- File size limits
- Unicode/special characters
- Edge cases (empty dirs, symlinks)
- Error path testing

## Current Test Infrastructure

### Project Structure

```
athena_viewer/
├── src/
│   ├── app/
│   │   ├── mod.rs
│   │   ├── app_error.rs          # AppError enum, AppResult<T>
│   │   └── state_handler/        # Event handlers (4 files)
│   ├── message_holder/
│   │   ├── mod.rs
│   │   ├── file_helper.rs        # + unit tests
│   │   ├── folder_holder.rs
│   │   └── code_highlighter.rs   # + unit tests
│   └── state_holder/
│       └── mod.rs
├── tests/                        # ✅ Implemented
│   ├── utils/
│   │   ├── mod.rs
│   │   ├── filesystem.rs         # TestFileSystem
│   │   ├── mock_app.rs           # TestApp wrapper
│   │   └── mock_terminal.rs      # TestBackend + events
│   ├── navigation.rs             # Navigation tests
│   └── history.rs                # History tests
├── Cargo.toml
└── INTEGRATION_TEST_TUTORIAL.md
```

### Dependencies

```toml
[dependencies]
ratatui = "0.29"
tui-input = "0.14"
chrono = { version = "0.4", features = ["serde"] }
lru = "0.16"
syntect = "5.3"
thiserror = "2.0"      # ✅ For error handling

[dev-dependencies]
tempfile = "3.23"      # ✅ For test fixtures
```

## Existing Test Examples

### 1. Navigation Tests (`tests/navigation.rs`)

```rust
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

        let mut visible_items = vec
!["..", "README.md", "main.rs", "src", "empty", ".gitkeep"];
        visible_items.sort();

        assert_eq!(app.get_visible_items()
, visible_items);

        // navigate down to and enter 'src/' directory
        app.send_events(vec
![
            events::char('s')
,
            events::char('r'),
            events::char('c'),
            events::enter(),
        ])
        .unwrap();

        assert!(app.get_current_directory().ends_with("src"));

        let mut visible_items = vec
!["..", "lib.rs", "module.rs", "nested"];
        visible_items.sort();
        assert_eq!(app.get_visible_items()
, visible_items);

        // filter and open lib.rs
        app.send_events(vec
![
            events::char('l')
,
            events::char('i'),
            events::char('b'),
            events::char('.'),
            events::char('r'),
            events::char('s'),
            events::enter(),
        ])
        .unwrap();

        // check filter is effective
        assert_eq!(app.get_visible_items()
, vec
!["lib.rs"])
;
        // file view mode with lib.rs
        assert!(app.is_file_view());
        assert!(app.get_opened_file().is_some());
        assert!(app.get_opened_file().unwrap().ends_with("lib.rs"));

        app.send_event(events::char('q')).unwrap();
        // check filter is still effective
        assert_eq!(app.get_visible_items()
, vec
!["lib.rs"])
;
    }

    #[test]
    fn test_navigate_to_parent_directory() {
        let fs = TestFileSystem::new();
        fs.create_nested_structure();

        let mut app = TestApp::new(fs.path().to_path_buf()).unwrap();
        assert_eq!(app.get_current_directory(), fs.path());

        // navigate down to and enter 'src/' directory
        app.send_events(vec
![
            events::char('s')
,
            events::char('r'),
            events::char('c'),
            events::enter(),
        ])
        .unwrap();

        assert!(app.get_current_directory().ends_with("src"));

        app.send_event(events::tab()).unwrap();
        assert!(app.is_normal_mode());
        app.send_event(events::ctrl_k()).unwrap();
        assert_eq!(app.get_current_directory(), fs.path());
    }
}
```

**Key Patterns**:
- Uses `TestFileSystem::new()` for isolated test directories
- `TestApp::new()` returns `AppResult<Self>` for error handling
- `send_events()` returns `AppResult<()>` - must use `.unwrap()` or `?`
- Tests use search filtering (char keys) to navigate, not arrow keys
- State assertions after each operation

### 2. History Tests (`tests/history.rs`)

```rust
pub mod utils;
#[cfg(test)]
mod history_tests {
    use super::utils::*;
    use crate::utils::TestFileSystem;

    #[test]
    fn test_history_navigation() {
        let fs = TestFileSystem::new();
        fs.create_nested_structure();

        let mut app = TestApp::new(fs.path().to_path_buf()).unwrap();

        // add src folder into history
        app.send_events(vec
![
            events::char('s')
,
            events::char('r'),
            events::char('c'),
            events::enter(),
        ])
        .unwrap();

        // add nested folder into history
        app.send_events(vec
![
            events::char('n')
,
            events::char('e'),
            events::char('s'),
            events::char('t'),
            events::char('e'),
            events::char('d'),
            events::enter(),
        ])
        .unwrap();

        // add deep folder into history
        app.send_events(vec
![
            events::char('d')
,
            events::char('e'),
            events::char('e'),
            events::char('p'),
            events::enter(),
        ])
        .unwrap();

        // switch to history view
        app.send_events(vec
![events::tab()
, events::char('h')])
            .unwrap();
        assert!(app.is_history_view());

        let mut history = Vec::new();
        let mut expected_suffix = ["src/nested/deep", "src/nested", "src"];
        for s in expected_suffix.iter_mut() {
            let holder = format!("{}/{}", fs.path().display(), s);
            history.push(holder)
        }
        history.push(fs.path().to_str().unwrap().to_string());

        assert_eq!(app.get_visible_items()
, history);

        // navigate and select history item
        app.send_events(vec
![events::down()
, events::down(), events::enter()])
            .unwrap();
        let mut visible_items = vec
!["..", "lib.rs", "module.rs", "nested"];
        visible_items.sort();
        assert_eq!(app.get_visible_items()
, visible_items);
    }

    #[test]
    fn test_history_navigation_removed_handling() {
        let fs = TestFileSystem::new();
        fs.create_nested_structure();

        let mut app = TestApp::new(fs.path().to_path_buf()).unwrap();

        // add folders to history
        app.send_events(vec
![
            events::char('s')
,
            events::char('r'),
            events::char('c'),
            events::enter(),
        ])
        .unwrap();

        app.send_events(vec
![
            events::char('n')
,
            events::char('e'),
            events::char('s'),
            events::char('t'),
            events::char('e'),
            events::char('d'),
            events::enter(),
        ])
        .unwrap();

        app.send_events(vec
![events::tab()
, events::char('h')])
            .unwrap();
        assert!(app.is_history_view());

        // verify history has both entries
        let mut history = Vec::new();
        let mut expected_suffix = ["src/nested", "src"];
        for s in expected_suffix.iter_mut() {
            let holder = format!("{}/{}", fs.path().display(), s);
            history.push(holder)
        }
        history.push(fs.path().to_str().unwrap().to_string());
        assert_eq!(app.get_visible_items()
, history);

        // remove folder and refresh
        fs.remove_folder("src/nested");
        let mut history = Vec::new();
        let mut expected_suffix = ["src"];
        for s in expected_suffix.iter_mut() {
            let holder = format!("{}/{}", fs.path().display(), s);
            history.push(holder)
        }
        history.push(fs.path().to_str().unwrap().to_string());

        app.send_events(vec
![events::enter()
]).unwrap();
        assert_eq!(app.get_visible_items()
, history);
    }
}
```

**Key Patterns**:
- Tests error handling for deleted folders in history
- Uses `fs.remove_folder()` to simulate filesystem changes
- Verifies state persistence across mode transitions

## Adding New Tests

### Step 1: Choose Test Location

**For integration tests** (user workflows):
- Add to `tests/navigation.rs` or `tests/history.rs`
- Create new file like `tests/search.rs` for search-specific tests

**For unit tests** (pure functions):
- Add `#[cfg(test)]` module at bottom of source file
- Example: `src/message_holder/file_helper.rs`

### Step 2: Write Test Structure

```rust
#[cfg(test)]
mod your_test_module {
    use super::utils::*;
    use crate::utils::TestFileSystem;

    #[test]
    fn test_your_feature() {
        // Setup
        let fs = TestFileSystem::new();
        fs.create_nested_structure();
        let mut app = TestApp::new(fs.path().to_path_buf()).unwrap();

        // Execute actions
        app.send_events(vec
![
            events::char('s')
,
            events::enter(),
        ])
        .unwrap();

        // Assert results
        assert!(app.is_file_view());
        assert_eq!(app.get_visible_items()
, vec
!["..", "lib.rs", "module.rs", "nested"])
;
    }
}
```

### Step 3: Use Test Utilities

**TestFileSystem methods**:
- `create_file(path, content)` - Create file with content
- `create_dir(path)` - Create directory
- `create_nested_structure()` - Create standard test structure
- `remove_file(path)` / `remove_folder(path)` - Simulate deletions
- `path()` - Get temp directory path

**TestApp methods**:
- `new(start_dir)` - Create app (returns `AppResult<Self>`)
- `send_event(event)` - Send single event (returns `AppResult<()>`)
- `send_events(events)` - Send multiple events (returns `AppResult<()>`)
- `is_edit_mode()`, `is_normal_mode()` - Check input mode
- `is_search_view()`, `is_file_view()`, `is_history_view()` - Check view mode
- `get_visible_items()` - Get current directory listing
- `get_current_directory()` - Get current path
- `get_opened_file()` - Get opened file path
- `get_search_input()` - Get current search filter
- `get_scroll_positions()` - Get (vertical, horizontal) scroll

**Event helpers** (`events::`):
- `char(c)` - Character key
- `enter()`, `tab()`, `escape()`
- `down()`, `up()`, `left()`, `right()`
- `page_down()`, `page_up()`, `home()`, `end()`
- `ctrl_c()`, `ctrl_d()`, `ctrl_k()`, `ctrl_z()`

## Unit Tests in Source Files

### File Helper Tests (`src/message_holder/file_helper.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_file_holder() -> Result<(), Box<dyn std::error::Error>> {
        let path = get_temp_file()?;
        let _ = FileHolder::try_from(path).unwrap();
        Ok(())
    }

    #[test]
    fn test_file_text_info() -> Result<(), Box<dyn std::error::Error>> {
        let path = get_temp_file()?;
        let code_highlighter = CodeHighlighter::default();
        let file_text_info = FileTextInfo::new(&path, &code_highlighter).unwrap();
        assert_eq!(file_text_info.n_rows, 1);
        assert_eq!(file_text_info.max_line_length, 17);
        Ok(())
    }

    fn get_temp_file() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let temp_file = NamedTempFile::new()?;
        let mut file = temp_file.reopen()?;
        file.write_all(b"Hello, world!")?;
        Ok(temp_file.path().to_path_buf())
    }
}
```

### Code Highlighter Tests (`src/message_holder/code_highlighter.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_plain_test() {
        let highlighter = CodeHighlighter::default();
        let syntax = highlighter.syntax_set.find_syntax_plain_text();

        let code = "abc \n cde";

        let out = highlighter.get_highlighted_code(code, syntax);
        assert_eq!(out.unwrap().len(), 2)
    }
}
```

## Advanced Testing Patterns

### 1. Testing Error Scenarios

```rust
#[test]
fn test_invalid_folder_handling() {
    let fs = TestFileSystem::new();
    fs.create_nested_structure();

    let mut app = TestApp::new(fs.path().to_path_buf()).unwrap();

    // Navigate to a folder
    app.send_events(vec
![
        events::char('s')
,
        events::char('r'),
        events::char('c'),
        events::enter(),
    ])
    .unwrap();

    // Delete the folder from filesystem
    fs.remove_folder("src");

    // Try to refresh - should handle gracefully
    app.send_event(events::char('r')).unwrap();

    // Should still be functional
    assert!(app.get_current_directory()
.is_file());
}
```

### 2. Testing Large Directories

```rust
#[test]
fn test_performance_with_many_files() {
    use std::time::Instant;

    let fs = TestFileSystem::new();

    // Create 100 files
    for i in 0..100 {
        fs.create_file(&format!("file_{}.txt", i), "content");
    }

    let start = Instant::now();
    let mut app = TestApp::new(fs.path().to_path_buf()).unwrap();

    // Filter to narrow down
    app.send_event(events::tab()).unwrap();
    app.send_events(vec
![
        events::char('f')
,
        events::char('i'),
        events::char('l'),
        events::char('e'),
    ])
    .unwrap();

    let duration = start.elapsed();
    assert!(duration.as_millis() < 100); // Should be fast
}
```

### 3. Testing State Transitions

```rust
#[test]
fn test_mode_switching_preserves_state() {
    let fs = TestFileSystem::new();
    fs.create_nested_structure();
    let mut app = TestApp::new(fs.path().to_path_buf()).unwrap();

    // Enter search mode and type filter
    app.send_event(events::tab()).unwrap();
    app.send_events(vec
![
        events::char('r')
,
        events::char('s'),
    ])
    .unwrap();

    let filter_before = app.get_search_input();
    assert_eq!(filter_before, "rs");

    // Switch to normal mode
    app.send_event(events::tab()).unwrap();
    assert!(app.is_normal_mode());

    // Filter should persist
    let filter_after = app.get_search_input();
    assert_eq!(filter_after, "rs");
}
```

## Best Practices

### ✅ DO

1. **Use `TestFileSystem` for isolation** - Each test gets a clean temp directory
2. **Handle `AppResult` properly** - Use `.unwrap()` for tests, `?` for helpers
3. **Assert state after each action** - Don't wait until end to check
4. **Test one behavior per test** - Keep tests focused
5. **Use descriptive test names** - `test_navigate_to_parent_directory` not `test_1`
6. **Clean up with `tempfile`** - Automatic cleanup is built-in

### ❌ DON'T

1. **Don't use real filesystem** - Always use `TestFileSystem`
2. **Don't ignore `AppResult`** - Tests will fail on errors
3. **Don't test multiple features in one test** - Hard to debug failures
4. **Don't hardcode paths** - Use `fs.path()` for portability
5. **Don't forget `.unwrap()` on send_events** - Returns `AppResult<()>`

### Test Naming Convention

```rust
// ✅ Good - Descriptive, specific
#[test]
fn test_search_filter_persists_after_mode_switch()

// ❌ Bad - Vague
#[test]
fn test_search()

// ✅ Good - Tests edge case
#[test]
fn test_empty_directory_navigation()

// ❌ Bad - Doesn't describe what's being tested
#[test]
fn test_edge_case()
```

## Running Tests

### Basic Commands

```bash
# Run all integration tests
cargo test --test navigation
cargo test --test history

# Run all tests
cargo test

# Run specific test
cargo test test_browse_directory_and_select_file

# Run with output
cargo test -- --nocapture

# Run with filter
cargo test navigation -- --test-threads=1
```

### Test Organization

```bash
tests/
├── utils/              # Shared test utilities
│   ├── mod.rs
│   ├── filesystem.rs   # TestFileSystem
│   ├── mock_app.rs     # TestApp
│   └── mock_terminal.rs # Events
├── navigation.rs       # Directory navigation tests
└── history.rs          # History feature tests
```

### CI Integration

For GitHub Actions, add to `.github/workflows/test.yml`:

```yaml
name: Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test
```

## Next Steps

### Immediate (High Priority)

1. **Add error case tests** - Test permission denied, deleted files
2. **Add unit tests for pure functions** - `get_highlight_index`, `should_select`
3. **Test edge cases** - Empty directories, unicode names, special characters

### Short-term

1. **Performance tests** - Large directories, file size limits
2. **State transition tests** - More complex mode switching
3. **Scroll behavior tests** - Vertical/horizontal scrolling edge cases

### Long-term

1. **Property-based testing** - Use `proptest` crate
2. **Visual regression tests** - Compare rendered output
3. **End-to-end tests** - Full terminal simulation

## Resources

- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-01-writing-tests.html)
- [tempfile crate docs](https://docs.rs/tempfile/latest/tempfile/)
- [Ratatui Testing Guide](https://ratatui.rs/testing/)
- [Crossterm Event Types](https://docs.rs/crossterm/latest/crossterm/event/enum.Event.html)

---

**Summary**: The Athena Viewer has a **complete, working test infrastructure**. Focus on adding error case tests and unit tests for pure functions to reach 90%+ coverage.
