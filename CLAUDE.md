# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Athena Viewer is an interactive terminal character echo application built in Rust. It provides a Claude Code-like interface with immediate character feedback, sliding window display, and clean input/output separation. The application demonstrates advanced terminal manipulation techniques and real-time character processing.

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

- `src/main.rs` - Main entry point containing the interactive character echo application (289 lines)
- `Cargo.toml` - Project configuration and dependencies (uses libc v0.2 for terminal control)
- `Cargo.lock` - Dependency lock file for reproducible builds
- `CLAUDE.md` - This documentation file
- `target/` - Build output directory (gitignored)

## Current Features

### Core Functionality
- **Immediate Character Echo**: Characters appear as you type without Enter confirmation
- **Sliding Window Display**: Most recent content appears closest to input area
- **Bottom-Aligned Output**: Content positions itself naturally at the bottom of the output area
- **Input Box Boundaries**: Clean visual separation with separator lines above and below input

### Terminal Features
- **Raw Terminal Mode**: Direct character input without line buffering
- **Multi-threaded Architecture**: Separate input thread for responsive character handling
- **ANSI Escape Sequences**: Advanced terminal control for positioning and formatting
- **Cross-Platform Support**: Uses libc for Unix-like system terminal control

### User Interface
- **Fixed Layout**: 24-line terminal layout with header, output area, and input box
- **Visual Separators**: ═ lines create clear boundaries between sections
- **Cursor Positioning**: Absolute positioning for consistent layout
- **Clean Exit**: Proper terminal mode restoration

## Technical Implementation

### Key Components
- `EchoState` struct: Manages current input, output history, and display logic (max_lines: 5)
- `enable_raw_mode()`/`disable_raw_mode()`: Terminal mode control functions using libc termios
- `render_echo_display()`: Main rendering function with bottom-aligned layout and ANSI escape sequences
- `get_sliding_display_output()`: Sliding window logic for content management (15 output lines max)
- Multi-threaded input processing with mpsc channels for responsive character handling
- Input thread handles raw character reading and special key detection (ESC, Ctrl+C, Enter, Backspace)

### Dependencies
- `libc`: Low-level system calls for terminal control (tcgetattr, tcsetattr)
- `std::io::Write`: Output handling and flushing
- `std::sync::mpsc`: Inter-thread communication
- `std::thread`: Concurrent input processing

## Development Notes

### Architecture Decisions
- Uses raw terminal mode for immediate character input
- Implements sliding window to handle content overflow gracefully
- Bottom-aligned display creates natural reading flow
- Simple exit behavior leaves content in scrollback buffer
- Multi-threaded design ensures responsive character processing

### Terminal Requirements
- Requires a Unix-like terminal environment
- Needs raw terminal mode support
- 24-line minimum terminal height recommended
- ANSI escape sequence support required

### Known Limitations
- Scrollback buffer clearing is terminal-dependent and not implemented
- Terminal size is assumed to be 24 lines minimum
- Currently designed for Unix-like systems only

## Recent Development & Code Quality

### Latest Improvements
Based on recent commit history, the project has undergone several refinements:

- **Code Quality**: Cleaned up code formatting and improved overall code quality
- **Documentation**: Updated and simplified documentation with clearer explanations
- **Output Handling**: Fixed immediate character echo and implemented bottom-aligned sliding window
- **Visual Design**: Added two-line separator to properly bound the input area
- **Performance**: Reduced gap between input and output areas for better UX

### Development Best Practices
- Code follows Rust style guidelines (`cargo fmt` compliant)
- Linter passes with clean results (`cargo clippy`)
- Error handling with proper Result types and user-friendly error messages
- Thread-safe communication using mpsc channels
- Proper resource cleanup with terminal mode restoration

### Build Status
- Compiles cleanly on Rust stable toolchain
- Minimal external dependencies (only libc for system calls)
- Efficient binary size and memory footprint