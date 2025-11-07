# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Athena Viewer is a Rust project currently in early development. The name suggests it may be intended as a visualization or viewing application, possibly related to Athena (which could refer to analytics, data processing, or database systems).

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

This is a minimal Rust project with standard structure:
- `src/main.rs` - Main entry point (currently contains "Hello, world!" program)
- `Cargo.toml` - Project configuration and dependencies
- `target/` - Build output directory (gitignored)

## Current State

The project is in initial setup phase with only a basic "Hello, world!" implementation. No dependencies are currently defined in Cargo.toml, and the architecture has not yet been established.

## Development Notes

Since this is an early-stage Rust project, common next steps would typically include:
- Defining dependencies in Cargo.toml
- Setting up module structure in src/
- Adding error handling and logging
- Implementing the core viewer functionality based on the project requirements