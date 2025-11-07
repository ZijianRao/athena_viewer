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

- `src/main.rs` - Main entry point containing the interactive character echo application
- `Cargo.toml` - Project configuration and dependencies (uses libc for terminal control)
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
- `EchoState` struct: Manages current input, output history, and display logic
- `enable_raw_mode()`/`disable_raw_mode()`: Terminal mode control functions
- `render_echo_display()`: Main rendering function with bottom-aligned layout
- `get_sliding_display_output()`: Sliding window logic for content management
- Multi-threaded input processing with mpsc channels

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