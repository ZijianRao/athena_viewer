# Athena Viewer - Future Feature Ideas

**Last Updated**: 2026-01-16
**Status**: Nice-to-have enhancements for future versions

This document lists potential future enhancements for Athena Viewer. These are **not required** for production readiness but could improve the user experience and code quality.

---

## 🎯 Performance Optimizations

### 1. Syntax Highlighting Cache
**Priority**: Medium | **Impact**: High | **Complexity**: Medium

**Problem**: Syntax highlighting is re-computed every time a file is opened, even if the content hasn't changed.

**Solution**: Implement a content-addressed cache for highlighted text.

```rust
// Pseudo-code
struct HighlightCache {
    cache: LruCache<u64, Vec<Span<'static>>>,
}

impl HighlightCache {
    fn get_or_compute(&mut self, content: &str, lang: &str) -> Vec<Span<'static>> {
        let hash = calculate_hash(content);
        if let Some(cached) = self.cache.get(&hash) {
            return cached.clone();
        }
        let highlighted = highlight(content, lang);
        self.cache.put(hash, highlighted.clone());
        highlighted
    }
}
```

**Benefits**:
- Reduces CPU usage for repeated file views
- Improves responsiveness for large files
- Minimal memory overhead (hash-based keys)

**Challenges**:
- Cache invalidation strategy
- Memory management for large highlighted outputs

---

### 2. Lazy File Loading
**Priority**: Low | **Impact**: Medium | **Complexity**: Low

**Problem**: Files are loaded entirely into memory even if only viewing a portion.

**Solution**: Implement line-based lazy loading with a sliding window.

```rust
struct LazyFileLoader {
    path: PathBuf,
    loaded_lines: Vec<String>,
    total_lines: usize,
    window_size: usize,
    current_offset: usize,
}

impl LazyFileLoader {
    fn get_visible_lines(&mut self, start: usize, count: usize) -> Vec<String> {
        // Only load lines that are actually needed
        if !self.is_loaded(start, count) {
            self.load_window(start, count);
        }
        self.loaded_lines[start..start+count].to_vec()
    }
}
```

**Benefits**:
- Faster initial load for large files
- Reduced memory usage
- Better UX for multi-gigabyte files

**Challenges**:
- File seeking performance
- Handling file modifications during viewing

---

### 3. Parallel Directory Scanning
**Priority**: Medium | **Impact**: Medium | **Complexity**: High

**Problem**: Directory scanning is single-threaded, which can be slow for large directories.

**Solution**: Use rayon or tokio for parallel directory traversal.

```rust
use rayon::prelude::*;

fn scan_directory_parallel(path: &Path) -> Vec<FileHolder> {
    std::fs::read_dir(path)
        .par_bridge()
        .filter_map(|entry| entry.ok())
        .map(|entry| FileHolder::from(entry.path()))
        .collect()
}
```

**Benefits**:
- Faster directory listing for large directories
- Better CPU utilization
- Improved responsiveness

**Challenges**:
- Thread safety for shared state
- Error handling across threads
- Potential race conditions

---

## 🎨 User Experience Improvements

### 4. Better Error Display in UI
**Priority**: High | **Impact**: High | **Complexity**: Low

**Problem**: Errors are displayed in the log area at the bottom, which is easy to miss.

**Solution**: Implement a modal error dialog with more context.

```rust
struct ErrorDialog {
    title: String,
    message: String,
    details: Option<String>,
    suggested_action: Option<String>,
}

impl ErrorDialog {
    fn show(&self, frame: &mut Frame) {
        // Render centered modal with error details
        // Include: error type, message, file/line, suggestion
    }
}
```

**Benefits**:
- More visible error reporting
- Better debugging information
- Improved user guidance

**Challenges**:
- UI space management
- Modal state handling
- Accessibility considerations

---

### 5. Configuration File Support
**Priority**: Medium | **Impact**: Medium | **Complexity**: Medium

**Problem**: All settings are hardcoded, no user customization.

**Solution**: Add TOML/JSON configuration file support.

```toml
# ~/.config/athena_viewer/config.toml
[ui]
theme = "default"
show_line_numbers = true
default_file_size_limit_mb = 10

[performance]
cache_size = 500
expand_thread_count = 4

[features]
syntax_highlighting = true
file_watching = false
```

**Benefits**:
- User customization
- Persistent preferences
- Environment-specific settings

**Challenges**:
- Config file parsing
- Validation and error handling
- Default configuration management

---

### 6. File Watching (Auto-refresh)
**Priority**: Low | **Impact**: Medium | **Complexity**: High

**Problem**: Files don't auto-refresh when modified externally.

**Solution**: Use `notify` crate to watch for file changes.

```rust
use notify::{Watcher, RecursiveMode, Result as NotifyResult};

fn watch_file(path: &Path, callback: impl Fn() + Send + 'static) -> NotifyResult<()> {
    let mut watcher = notify::recommended_watcher(move |res| {
        match res {
            Ok(_) => callback(),
            Err(e) => eprintln!("Watch error: {:?}", e),
        }
    })?;
    watcher.watch(path, RecursiveMode::NonRecursive)?;
    Ok(())
}
```

**Benefits**:
- Real-time file updates
- Better for log viewing
- Improved development workflow

**Challenges**:
- Cross-platform file watching
- Performance overhead
- Debouncing rapid changes

---

### 7. Bookmark System
**Priority**: Low | **Impact**: Medium | **Complexity**: Medium

**Problem**: No way to quickly navigate to frequently accessed files/directories.

**Solution**: Implement persistent bookmarks with quick access.

```rust
struct Bookmark {
    name: String,
    path: PathBuf,
    created: DateTime<Utc>,
    last_accessed: DateTime<Utc>,
}

struct BookmarkManager {
    bookmarks: Vec<Bookmark>,
    storage_path: PathBuf,
}

impl BookmarkManager {
    fn add_bookmark(&mut self, name: String, path: PathBuf) -> AppResult<()> {
        // Add to list and persist to disk
    }

    fn get_bookmarks(&self) -> Vec<&Bookmark> {
        // Return sorted by last_accessed (most recent first)
    }
}
```

**Benefits**:
- Quick navigation to important files
- Persistent across sessions
- Organized by user-defined names

**Challenges**:
- Storage format and persistence
- UI for managing bookmarks
- Conflict resolution for duplicate names

---

## 🔧 Code Quality & Testing

### 8. Property-Based Testing
**Priority**: Low | **Impact**: Medium | **Complexity**: High

**Problem**: Unit tests only cover specific cases, not edge cases.

**Solution**: Use `proptest` crate for property-based testing.

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_get_highlight_index_helper_properties(
        raw_index in 0..100i32,
        group_len in 1..20usize,
    ) {
        let result = MessageHolder::get_highlight_index_helper(raw_index, group_len);
        prop_assert!(result.is_ok());
        let index = result.unwrap();
        prop_assert!(index < group_len);
    }
}
```

**Benefits**:
- Finds edge cases automatically
- Better test coverage
- Catches subtle bugs

**Challenges**:
- Learning curve for property-based testing
- Test performance
- Debugging failing properties

---

### 9. Performance Benchmarks
**Priority**: Low | **Impact**: Low | **Complexity**: Medium

**Problem**: No way to measure performance improvements.

**Solution**: Add `criterion` benchmarks for critical paths.

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_should_select_helper(c: &mut Criterion) {
    c.bench_function("should_select_helper", |b| {
        b.iter(|| {
            should_select_helper(
                black_box("main.rs"),
                black_box("rs"),
            )
        })
    });
}

criterion_group!(benches, bench_should_select_helper);
criterion_main!(benches);
```

**Benefits**:
- Track performance over time
- Identify bottlenecks
- Validate optimizations

**Challenges**:
- Benchmark maintenance
- CI integration
- Statistical significance

---

### 10. Integration with External Tools
**Priority**: Low | **Impact**: Low | **Complexity**: Medium

**Problem**: No integration with external editors or tools.

**Solution**: Add support for opening files in external editors.

```rust
struct ExternalEditor {
    editor_command: String,
    editor_args: Vec<String>,
}

impl ExternalEditor {
    fn open_file(&self, path: &Path) -> AppResult<()> {
        let mut cmd = Command::new(&self.editor_command);
        cmd.args(&self.editor_args);
        cmd.arg(path);
        cmd.status()?;
        Ok(())
    }
}
```

**Benefits**:
- Seamless workflow with existing tools
- User choice of editor
- Better for complex file operations

**Challenges**:
- Cross-platform command execution
- Error handling for external processes
- Configuration management

---

## 🌐 Platform-Specific Features

### 11. Windows Terminal Integration
**Priority**: Low | **Impact**: Low | **Complexity**: Medium

**Problem**: Windows Terminal integration could be improved.

**Solution**: Add Windows-specific features like:
- Windows Terminal profile integration
- Windows file system metadata display
- Windows-specific keyboard shortcuts

**Benefits**:
- Better Windows user experience
- Platform-specific optimizations

**Challenges**:
- Platform-specific code maintenance
- Testing on multiple platforms

---

### 12. macOS Integration
**Priority**: Low | **Impact**: Low | **Complexity**: Medium

**Problem**: macOS-specific features could be added.

**Solution**: Add macOS-specific features like:
- Spotlight integration for file search
- Quick Look preview support
- macOS keyboard shortcuts (Cmd, Option)

**Benefits**:
- Better macOS user experience
- Platform-specific optimizations

**Challenges**:
- Platform-specific code maintenance
- Testing on multiple platforms

---

## 📊 Monitoring & Analytics

### 13. Usage Analytics (Optional)
**Priority**: Low | **Impact**: Low | **Complexity**: Medium

**Problem**: No visibility into how users interact with the application.

**Solution**: Add optional, privacy-preserving analytics.

```rust
struct UsageAnalytics {
    session_start: Instant,
    files_viewed: usize,
    directories_browsed: usize,
    searches_performed: usize,
}

impl UsageAnalytics {
    fn record_file_view(&mut self) {
        self.files_viewed += 1;
    }

    fn export_summary(&self) -> String {
        format!(
            "Session: {:?}\nFiles viewed: {}\nDirectories browsed: {}\nSearches: {}",
            self.session_start.elapsed(),
            self.files_viewed,
            self.directories_browsed,
            self.searches_performed
        )
    }
}
```

**Benefits**:
- Understand user behavior
- Identify popular features
- Guide future development

**Challenges**:
- Privacy concerns
- Data storage and transmission
- User opt-in/opt-out

---

## 🎓 Learning Resources

### 14. Built-in Tutorial
**Priority**: Low | **Impact**: Low | **Complexity**: High

**Problem**: New users may not know how to use all features.

**Solution**: Add an interactive tutorial mode.

```rust
struct Tutorial {
    step: usize,
    steps: Vec<TutorialStep>,
    completed: bool,
}

struct TutorialStep {
    title: String,
    description: String,
    hint: String,
    required_action: Action,
}
```

**Benefits**:
- Better onboarding for new users
- Discover hidden features
- Improved user retention

**Challenges**:
- Tutorial state management
- Balancing tutorial vs. normal mode
- Maintaining tutorial content

---

## 📝 Documentation Improvements

### 15. Man Page Generation
**Priority**: Low | **Impact**: Low | **Complexity**: Low

**Problem**: No man page for Linux/Unix systems.

**Solution**: Generate man page from Rustdoc comments.

```bash
# Generate man page
cargo install cargo-mangen
cargo mangen --output athena_viewer.1
```

**Benefits**:
- Standard Unix documentation
- Better integration with system
- Professional appearance

**Challenges**:
- Maintaining man page format
- Keeping in sync with code

---

### 16. Shell Completion Scripts
**Priority**: Low | **Impact**: Low | **Complexity**: Low

**Problem**: No shell completion for command-line arguments.

**Solution**: Generate completion scripts for bash, zsh, fish.

```bash
# Generate completions
cargo install cargo-completion
cargo completion bash > /etc/bash_completion.d/athena_viewer
```

**Benefits**:
- Better CLI experience
- Discoverable command-line options
- Professional tooling

**Challenges**:
- Maintaining completion scripts
- Shell-specific syntax

---

## 🎯 Priority Matrix

| Feature | Priority | Impact | Complexity | Effort |
|---------|----------|--------|------------|--------|
| **Syntax Highlighting Cache** | Medium | High | Medium | ⭐⭐⭐ |
| **Better Error Display** | High | High | Low | ⭐⭐ |
| **Configuration File** | Medium | Medium | Medium | ⭐⭐⭐ |
| **Property-Based Testing** | Low | Medium | High | ⭐⭐⭐⭐ |
| **Performance Benchmarks** | Low | Low | Medium | ⭐⭐ |
| **File Watching** | Low | Medium | High | ⭐⭐⭐⭐ |
| **Bookmark System** | Low | Medium | Medium | ⭐⭐⭐ |
| **External Editor Integration** | Low | Low | Medium | ⭐⭐ |
| **Platform Integration** | Low | Low | Medium | ⭐⭐ |
| **Usage Analytics** | Low | Low | Medium | ⭐⭐ |
| **Built-in Tutorial** | Low | Low | High | ⭐⭐⭐⭐ |
| **Man Page** | Low | Low | Low | ⭐ |
| **Shell Completions** | Low | Low | Low | ⭐ |

---

## 🚀 Recommended Roadmap

### Phase 1: Core Polish (Post-Production)
1. **Better Error Display** - High impact, low complexity
2. **Configuration File** - Medium impact, medium complexity
3. **Syntax Highlighting Cache** - High impact, medium complexity

### Phase 2: Advanced Features
4. **Bookmark System** - Medium impact, medium complexity
5. **File Watching** - Medium impact, high complexity
6. **External Editor Integration** - Low impact, medium complexity

### Phase 3: Testing & Quality
7. **Property-Based Testing** - Medium impact, high complexity
8. **Performance Benchmarks** - Low impact, medium complexity

### Phase 4: Platform & Polish
9. **Platform Integration** - Low impact, medium complexity
10. **Shell Completions** - Low impact, low complexity
11. **Man Page** - Low impact, low complexity

---

## 📚 References

- **proptest**: https://github.com/proptest-rs/proptest
- **criterion**: https://github.com/bheisler/criterion.rs
- **notify**: https://github.com/notify-rs/notify
- **serde**: https://serde.rs/ (for config serialization)
- **rayon**: https://github.com/rayon-rs/rayon (for parallel processing)

---

## 🎯 Conclusion

These features are **nice-to-have** enhancements that could improve Athena Viewer in the future. The current codebase is already production-ready (after fixing 4 critical panics), so these features should be prioritized based on user feedback and development resources.

**Start with**: Better error display and configuration file support - these provide the best ROI for user experience improvements.
