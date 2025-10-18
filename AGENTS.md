# Agent Guidelines for Craters

## Build Commands
- `cargo build` - Debug build
- `cargo build --release` - Release build
- `cargo run` - Run in debug mode
- `cargo run --release` - Run in release mode
- `cargo check` - Check for compilation errors

## Test Commands
- `cargo test` - Run all tests
- `cargo test TESTNAME` - Run specific test (e.g., `cargo test handle_key_event`)

## Lint and Format Commands
- `cargo clippy` - Run linter
- `cargo fmt` - Format code

## Code Style Guidelines

### Imports
- Group imports: std/prelude first, external crates, then local modules
- Use explicit imports over glob imports

### Naming Conventions
- Structs/Enums: PascalCase
- Functions/Methods/Variables: snake_case
- Constants: SCREAMING_SNAKE_CASE
- Modules: snake_case

### Error Handling
- Use `anyhow::Result` for main functions
- Custom errors with `thiserror::Error`
- Use `?` operator for error propagation

### Async Code
- Use `#[tokio::main]` for main functions
- Use `#[tokio::test]` for async tests

### Derives
- Common derives: `Debug`, `Default`, `Clone`, `Copy`
- Serde: `serde::Deserialize` for API structs
- Enums: `Display`, `FromRepr`, `EnumIter` for navigation

### Attributes
- `#[allow(dead_code)]` for unused code during development
- Use explicit lifetimes where needed

### Logging
- Use `log` crate with `simplelog` for file logging
- Debug level for development information