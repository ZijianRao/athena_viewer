# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Athena Viewer is a terminal-based file viewer application built in Rust using the `ratatui` TUI framework. It provides syntax-highlighted file viewing, directory navigation, and search functionality with a modal interface (Normal/Edit modes).

**Current Status**: Prototype (v0.1.0) - Working features, but requires error handling and testing improvements for production readiness.

## How We Use Claude Code

Claude Code is used **primarily as a code coach** in this project:
- **Code review and feedback**: Analyzing Rust code patterns and suggesting improvements
- **Learning guidance**: Providing structured feedback on Rust idioms and best practices
- **Architecture advice**: Suggesting better design patterns and organization
- **Error handling guidance**: Helping implement proper Rust error handling patterns
- **Test planning**: Identifying which functions need tests and how to structure them

**Important**: Focus on teaching Rust concepts through code review rather than direct code generation. When the user wants to make changes, guide them through the learning process.

The focus is on **learning Rust through code review** rather than automated code generation.

## Development Commands

### Building and Running
- `cargo build` - Build the project in debug mode
- `cargo run` - Build and run the application
- `cargo build --release` - Build optimized release version

### Development Tools
- `cargo check` - Quick compile check without producing executable
- `cargo clippy` - Run Rust linter for code quality
- `cargo fmt` - Format code according to Rust style guidelines
- `cargo test` - Run tests (when tests are added)

## Project Structure

```
src/
├── main.rs              # Application entry point
├── app/                 # Main application logic
│   ├── mod.rs          # App struct and main loop
│   └── state_handler/  # State-specific event handlers
├── message_holder/      # File viewing and message display
│   ├── mod.rs
│   ├── message_holder.rs
│   ├── file_helper.rs
│   └── code_highlighter.rs
└── state_holder/       # Application state management
    ├── mod.rs
    └── state_holder.rs
```

- `Cargo.toml` - Project configuration and dependencies
- `Cargo.lock` - Dependency lock file for reproducible builds
- `CLAUDE.md` - This documentation file
- `RUST_CODE_REVIEW.md` - Code review feedback and improvement suggestions
- `target/` - Build output directory (gitignored)

## Current Features

### Core Functionality
- **Syntax-highlighted file viewing**: Uses `syntect` crate for code highlighting
- **Modal interface**: Normal mode (navigation) and Edit mode (search/input)
- **Directory navigation**: Browse and select files in current directory
- **File search**: Search through files with highlighting
- **LRU caching**: Efficient file caching with configurable size
- **State machine**: Enum-driven state management (`InputMode`, `ViewMode`)

### User Interface
- **TUI framework**: Built with `ratatui` and `crossterm`
- **Input handling**: Uses `tui-input` for text input
- **Help display**: Context-sensitive help in different modes
- **Visual feedback**: Clear mode indicators and status

## Technical Implementation

### Key Components
- `App` struct: Main application state and rendering
- `StateHolder`: Manages application mode (`InputMode`, `ViewMode`)
- `MessageHolder`: Handles file loading, caching, and display
- `CodeHighlighter`: Syntax highlighting using syntect
- State handlers: Mode-specific event handling in `app/state_handler/`

### Dependencies
- `ratatui`: Terminal user interface framework
- `crossterm`: Cross-platform terminal operations
- `tui-input`: Text input handling
- `syntect`: Syntax highlighting
- `lru`: LRU caching for files
- `chrono`: Date/time handling

## Development Notes

### Architecture Decisions
- **Single-threaded event loop**: Simple blocking event reading
- **Shared mutable state**: Uses `Rc<RefCell<T>>` for shared state (idiomatic for single-threaded TUI)
- **Enum-driven state machine**: Clear mode transitions (`InputMode`, `ViewMode`)
- **Modular organization**: Separated by concern (app, state, messages)

### Known Limitations & Priority Improvements

#### Critical (Production-blocking)
1. **Error handling**: 20+ `unwrap()` calls - app crashes on errors
2. **Zero tests**: No safety net for refactoring

#### High Priority
3. **Safety**: No path traversal protection, file size limits
4. **Documentation**: Zero Rustdoc comments

#### Medium Priority
5. **Performance**: Unnecessary allocations in hot paths
6. **Code duplication**: Draw handlers and event handlers have repetitive patterns

### Learning Path for Rust
1. **Week 1-2**: Error handling with `thiserror` crate
2. **Week 3-4**: Add unit tests (start with math logic)
3. **Week 5-6**: Refactor large functions, extract patterns
4. **Week 7-8**: Add safety checks, documentation

## Code Review Focus Areas

When reviewing code in this project, focus on:

### 1. Rust Idioms & Learning Points
- **Ownership**: Identify unnecessary `to_string()` calls and allocations
- **Error Handling**: Replace all `unwrap()` with `?` and proper error types
- **Traits**: `Rc<RefCell<T>>` usage, trait bounds in `ratatui` and `syntect`
- **Enums**: State machine patterns with `#[derive(Copy, Default)]`

### 2. Safety & Production Readiness
- **No panics**: Every `unwrap()` is a crash waiting to happen
- **Bounds checking**: Array access, scroll positions, index calculations
- **Input validation**: Path traversal, file size limits
- **Error propagation**: Use `Result<T, E>` and `?` operator

### 3. Code Quality
- **Function size**: Keep < 50 lines (refactor `handle_normal_file_view_event`)
- **Duplication**: Extract repeated patterns in draw handlers
- **Performance**: Minimize allocations in hot paths (keystroke handling)
- **Constants**: Remove magic numbers

### 4. Testing Strategy
- **Unit tests**: Pure functions like `get_highlight_index`, `should_select`
- **Integration tests**: Navigation flow, file opening
- **Mock filesystem**: Use `tempfile` for test fixtures

## Project Documentation

### Key Files
- `RUST_CODE_REVIEW.md` - Comprehensive code review (updated 2025-12-17)
- `CLAUDE.md` - This file
- `Cargo.toml` - Project dependencies

### Important Links
- **Code review**: See `RUST_CODE_REVIEW.md` for detailed analysis
- **Quick wins**: Steps 1-5 in the review for immediate improvements

## Recent Development History

Based on git commits:
- `e578cf4` feat: clear input line
- `12c59a9` feat: normal search: delete, go up
- `fe12125` fix: update current holder as well in refresh
- `592a9f1` feat: duration logging of each operation
- `db43da3` feat: collapse support for folder search

**Pattern**: Good feature iteration. Recent focus on UX and debugging capabilities.