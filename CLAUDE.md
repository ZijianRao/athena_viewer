# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Athena Viewer is a terminal-based file viewer application built in Rust using the `ratatui` TUI framework. It provides syntax-highlighted file viewing, directory navigation, and search functionality with a modal interface (Normal/Edit modes).

**Current Status**: Beta Candidate (v0.1.0) - **80% to production**. Error handling, performance, and test infrastructure are solid. **5 critical panics remain** before production readiness.

## How We Use Claude Code

Claude Code is used **primarily as a code coach** in this project:
- **Code review and feedback**: Analyzing Rust code patterns and suggesting improvements
- **Learning guidance**: Providing structured feedback on Rust idioms and best practices
- **Architecture advice**: Suggesting better design patterns and organization
- **Error handling guidance**: Helping implement proper Rust error handling patterns
- **Test planning**: Identifying which functions need tests and how to structure them

**Important**: Focus on teaching Rust concepts through code review rather than direct code generation. Guide through the learning process.

## Development Commands

### Building and Running
- `cargo build` - Build the project in debug mode
- `cargo run` - Build and run the application
- `cargo build --release` - Build optimized release version

### Development Tools
- `cargo check` - Quick compile check without producing executable
- `cargo clippy` - Run Rust linter for code quality
- `cargo fmt` - Format code according to Rust style guidelines
- `cargo test` - Run all tests

## Project Structure

```
athena_viewer/
├── src/
│   ├── main.rs              # Entry point - proper error handling
│   ├── lib.rs               # Library exports
│   ├── app/                 # Main application logic
│   │   ├── mod.rs          # App struct and main loop
│   │   ├── app_error.rs    # Error types (6 variants)
│   │   └── state_handler/  # State-specific event handlers (4 files)
│   │       ├── normal_search.rs
│   │       ├── normal_file_view.rs
│   │       ├── edit_search.rs
│   │       └── edit_history_folder_view.rs
│   ├── message_holder/      # File viewing and message display
│   │   ├── mod.rs          # MessageHolder + unit tests
│   │   ├── file_helper.rs  # File I/O (+tests)
│   │   ├── folder_holder.rs # Directory navigation
│   │   └── code_highlighter.rs # Syntax highlighting (+tests)
│   └── state_holder/        # Application state management
│       └── mod.rs          # State machine (consolidated)
├── tests/                   # Integration tests
│   ├── utils/
│   │   ├── filesystem.rs   # TestFileSystem
│   │   ├── mock_app.rs     # TestApp wrapper
│   │   └── mock_terminal.rs # Mock backend
│   ├── navigation.rs       # Directory browsing tests
│   └── history.rs          # History feature tests
├── Cargo.toml               # Dependencies (thiserror = "2.0")
├── README.md                # User documentation
├── CLAUDE.md                # This file (developer guide)
├── RUST_CODE_REVIEW.md      # Detailed code analysis
├── INTEGRATION_TEST_TUTORIAL.md # Testing guide
└── target/                  # Build output (gitignored)
```

**Recent Progress** (Jan 4, 2026):
- ✅ Added `thiserror = "2.0"` and `AppError` enum (6 variants)
- ✅ Fixed O(n²) → O(n) algorithm (100x speedup)
- ✅ Added file size limits (10MB max)
- ✅ Added integration tests + unit tests (70% happy paths)
- ✅ Consolidated module structure

**Documentation Files**:
- `README.md` - User-facing documentation
- `CLAUDE.md` - This file - development guidance
- `RUST_CODE_REVIEW.md` - Comprehensive analysis (updated Jan 4, 2026)
- `INTEGRATION_TEST_TUTORIAL.md` - Testing guide (concise)

## Current Status Summary

### ✅ Completed (80% to Production)
1. **Error Handling**: Complete `thiserror` integration with 6 error variants
2. **Performance**: O(n²) → O(n) algorithm (100x speedup for large directories)
3. **Safety**: File size limits (10MB) implemented
4. **Tests**: Integration tests + unit tests for core functions
5. **Architecture**: Clean module consolidation

### ❌ Remaining (Critical)
1. **5 Critical Panics** (1-2 hours to fix):
   - `app/mod.rs:53` - Terminal draw error
   - `folder_holder.rs:14` - Const panic
   - `folder_holder.rs:220` - Cache panic
   - `message_holder/mod.rs:269,273,280,287` - Test code

2. **Documentation**: Zero Rustdoc comments
3. **Error Testing**: 0% coverage for error paths
4. **Path Security**: No traversal protection

## Key Components

### Core Architecture
- `App`: Main application state and rendering
- `StateHolder`: Enum-driven state machine (`InputMode`, `ViewMode`)
- `MessageHolder`: File loading, caching, highlighting
- `FolderHolder`: Directory navigation and search
- `AppError`: 6 error variants (Io, Path, Parse, State, Terminal, Cache)

### State Machine
```
[Normal+Search] <---> [Edit+Search]
     |                     |
     v                     v
[Normal+FileView]   [Edit+HistoryFolderView]
```

## Code Review Focus Areas

### 1. Error Handling (Priority #1)
- **Current**: 12 unwrap() calls (5 critical, 7 safe)
- **Goal**: All production code must use `?` operator
- **Pattern**: `value.map_err(|_| AppError::Type("msg".into()))?`

### 2. Safety & Production Readiness
- **File size limits**: ✅ Implemented (10MB)
- **Path traversal**: ❌ Needs protection
- **Bounds checking**: ✅ get_highlight_index fixed
- **Unicode handling**: ⚠️ Mixed results

### 3. Performance
- **Search algorithm**: ✅ O(n²) → O(n) (100x speedup)
- **String allocations**: ✅ Reduced in hot paths
- **Cache operations**: ⚠️ Needs error handling

### 4. Testing Strategy
- **Happy paths**: ✅ 70% coverage
- **Error cases**: ❌ 0% coverage
- **Unit tests**: ✅ For pure functions
- **Integration**: ✅ Navigation, history

### 5. Documentation
- **Rustdoc comments**: ❌ Zero (all public items need docs)
- **Function comments**: ⚠️ Some exist
- **Module docs**: ❌ None

## Current Priorities

### Immediate (1-2 hours) - **CRITICAL FOR PRODUCTION**
1. Fix 5 critical panics in `app/mod.rs`, `folder_holder.rs`, `message_holder/mod.rs`
2. Replace remaining unwrap() with proper error handling

### Short-term (1-2 days)
1. Add error case tests (permission denied, deleted files)
2. Add path traversal protection
3. Add edge case tests (empty dirs, unicode, symlinks)

### Medium-term (2-3 days)
1. Add Rustdoc comments to all public items
2. Refactor large functions (< 50 lines)
3. Add constants for magic numbers

## Learning Path for Rust

### ✅ What You've Learned
1. **Error handling**: `thiserror`, `AppResult<T>`, `?` operator
2. **Algorithm optimization**: O(n²) → O(n) analysis
3. **Performance**: Allocation awareness in hot paths
4. **Module consolidation**: Clean architecture patterns
5. **Test infrastructure**: Mock TUI and filesystem
6. **State machines**: Enum-driven design with `Copy` + `Default`

### 📚 Next Steps (Documentation & Safety)
1. **Rustdoc conventions**: API documentation
2. **Safety patterns**: Input validation, bounds checking
3. **Traits**: Abstraction and code reuse
4. **Lifetime annotations**: More explicit types

## Important Links

- **Code Review**: `RUST_CODE_REVIEW.md` - Detailed analysis (Jan 4, 2026)
- **Testing Guide**: `INTEGRATION_TEST_TUTORIAL.md` - How to write tests
- **Quick Wins**: Fix 5 panics → Add tests → Add docs

## Git Commit Pattern

Recent progression:
- Features → Tests → Bug fixes → Refactoring → Error handling
- Focus: Module consolidation and production readiness

**Current**: Ready for final error handling fixes and documentation pass.