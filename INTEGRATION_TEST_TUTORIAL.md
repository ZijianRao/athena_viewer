# Athena Viewer Integration Testing Guide

**Status**: ✅ Test infrastructure implemented (70% happy path coverage)
**Focus**: Add error case tests and unit tests for pure functions

## Quick Start

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_browse_directory_and_select_file

# Run with output
cargo test -- --nocapture
```

## Test Structure

```
tests/
├── utils/
│   ├── filesystem.rs      # TestFileSystem - isolated temp directories
│   ├── mock_app.rs        # TestApp wrapper for integration tests
│   └── mock_terminal.rs   # Mock backend + event helpers
├── navigation.rs          # Directory browsing, file selection
└── history.rs             # History feature tests

src/
├── message_holder/
│   ├── file_helper.rs     # Unit tests for FileHolder, FileTextInfo
│   ├── code_highlighter.rs # Unit tests for syntax highlighting
│   └── mod.rs             # Unit tests for get_highlight_index
```

## Existing Test Examples

### Integration Test (tests/navigation.rs)
```rust
#[test]
fn test_browse_directory_and_select_file() {
    let fs = TestFileSystem::new();
    fs.create_nested_structure();
    let mut app = TestApp::new(fs.path().to_path_buf()).unwrap();

    // Navigate: type 's' 'r' 'c' then enter
    app.send_events(vec
![
        events::char('s')
,
        events::char('r'),
        events::char('c'),
        events::enter(),
    ]).unwrap();

    assert!(app.get_current_directory().ends_with("src"));
    assert!(app.is_file_view());
}
```

### Unit Test (src/message_holder/mod.rs)
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_highlight_index_helper() {
        let act = MessageHolder::get_highlight_index_helper(1, 10).unwrap();
        assert_eq!(act, 1);
    }
}
```

## Test Utilities

### TestFileSystem
- `create_file(path, content)` - Create file with content
- `create_dir(path)` - Create directory
- `create_nested_structure()` - Standard test structure
- `remove_file(path)` / `remove_folder(path)` - Simulate deletions
- `path()` - Get temp directory path

### TestApp
- `new(start_dir)` - Create app (returns `AppResult<Self>`)
- `send_event(event)` / `send_events(events)` - Send input
- `is_edit_mode()`, `is_normal_mode()` - Check input mode
- `is_search_view()`, `is_file_view()`, `is_history_view()` - Check view mode
- `get_visible_items()` - Current directory listing
- `get_current_directory()` - Current path
- `get_opened_file()` - Opened file path
- `get_search_input()` - Current search filter

### Event Helpers (events::)
- `char(c)`, `enter()`, `tab()`, `escape()`
- `down()`, `up()`, `left()`, `right()`
- `page_down()`, `page_up()`, `home()`, `end()`
- `ctrl_c()`, `ctrl_d()`, `ctrl_k()`, `ctrl_z()`

## What's Tested (70% happy paths)

✅ **Integration Tests**:
- Browse directories and select files
- Navigate to parent directory
- Search/filter with persistence
- History navigation and invalid folder handling
- Mode switching with state preservation

✅ **Unit Tests**:
- FileHolder creation from paths
- FileTextInfo dimensions calculation
- Code highlighting for plain text
- get_highlight_index edge cases (positive, negative, zero)

## What's NOT Tested (0% error cases)

❌ **Error Paths**:
- Permission denied scenarios
- Deleted files/directories
- Malformed paths
- File size limit violations

❌ **Edge Cases**:
- Empty directories
- Symlinks
- Unicode/special characters in names
- Root directory navigation

❌ **AppError Variants**:
- All 6 error types need testing
- Error propagation in state handlers

## Adding New Tests

### Step 1: Choose Location
- **Integration**: Add to `tests/navigation.rs` or `tests/history.rs`
- **Unit**: Add `#[cfg(test)]` module at bottom of source file

### Step 2: Write Test
```rust
#[test]
fn test_your_feature() {
    let fs = TestFileSystem::new();
    fs.create_nested_structure();
    let mut app = TestApp::new(fs.path().to_path_buf()).unwrap();

    // Execute actions
    app.send_events(vec
![events::char('s')
, events::enter()]).unwrap();

    // Assert results
    assert!(app.is_file_view());
}
```

### Step 3: Add Error Case Tests
```rust
#[test]
fn test_invalid_path_handling() {
    let fs = TestFileSystem::new();
    let mut app = TestApp::new(fs.path().to_path_buf()).unwrap();

    // Navigate to folder, then delete it
    app.send_events(vec
![events::char('s')
, events::enter()]).unwrap();
    fs.remove_folder("src");

    // Should handle gracefully, not crash
    app.send_event(events::char('r')).unwrap();
    assert!(app.get_current_directory().is_file()); // Still functional
}

#[test]
fn test_file_size_limit() {
    let large_content = "x".repeat(11 * 1024 * 1024);
    let path = fs.create_file("large.txt", &large_content);

    let result = FileTextInfo::new(&path, &CodeHighlighter::default());
    assert!(result.is_err());
}
```

## Best Practices

✅ **DO**:
- Use `TestFileSystem` for isolation
- Handle `AppResult` properly (`.unwrap()` in tests, `?` in helpers)
- Assert state after each action
- Test one behavior per test
- Use descriptive test names

❌ **DON'T**:
- Use real filesystem
- Ignore `AppResult` returns
- Test multiple features in one test
- Hardcode paths (use `fs.path()`)

## Next Steps

### Immediate (High Priority)
1. **Add error case tests** - Permission denied, deleted files
2. **Add unit tests for pure functions** - `should_select` variations
3. **Test edge cases** - Empty dirs, unicode, symlinks

### Short-term
1. **Performance tests** - Large directories, timing
2. **State transition tests** - Complex mode switching
3. **AppError variant tests** - All 6 error types

### Long-term
1. **Property-based testing** - `proptest` crate
2. **Visual regression tests** - Compare rendered output
3. **End-to-end tests** - Full terminal simulation

---

**Summary**: The test infrastructure is solid. Focus on adding error case tests and unit tests for remaining pure functions to reach 90%+ coverage.
