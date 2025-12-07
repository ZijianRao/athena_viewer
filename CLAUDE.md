# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Athena Viewer is a terminal-based file viewer application built in Rust using the `ratatui` TUI framework. It provides syntax-highlighted file viewing, directory navigation, and search functionality with a modal interface (Normal/Edit modes).

## How We Use Claude Code

Claude Code is used **primarily as a code coach** in this project:
- **Code review and feedback**: Analyzing Rust code patterns and suggesting improvements
- **Learning guidance**: Providing structured feedback on Rust idioms and best practices
- **Architecture advice**: Suggesting better design patterns and organization
- **Error handling guidance**: Helping implement proper Rust error handling patterns

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
- **Shared mutable state**: Uses `Rc<RefCell<T>>` for shared state
- **Enum-driven state machine**: Clear mode transitions
- **Modular organization**: Separated by concern (app, state, messages)

### Known Limitations
- **Error handling**: Needs improvement (multiple `unwrap()` calls)
- **Testing**: No unit or integration tests
- **Documentation**: Missing Rustdoc comments
- **Safety**: No path traversal protection or file size limits

## Code Review Focus Areas

When reviewing code in this project, focus on:
1. **Rust idioms**: Ownership, borrowing, error handling patterns
2. **Error handling**: Replace `unwrap()` with proper error propagation
3. **Code organization**: Module structure and separation of concerns
4. **Performance**: String allocations, caching efficiency
5. **Safety**: File system access, bounds checking
6. **Testing**: Adding unit tests for core logic

## Recent Development

Based on commit history, recent improvements include:
- State management refactoring
- Input/output state separation
- Code quality improvements
- Documentation cleanup

See `RUST_CODE_REVIEW.md` for detailed code review feedback and improvement suggestions.