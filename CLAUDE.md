# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Craters is a terminal-based UI application for browsing crates.io, built with Ratatui (v0.29.0). It displays information about Rust crates in an interactive TUI dashboard with a 2x3 grid layout.

## Build and Run Commands

```bash
# Build the project
cargo build

# Run the application
cargo run

# Run in release mode
cargo run --release

# Check for compilation errors
cargo check

# Run tests
cargo test
```

## Architecture

### Application Structure

The application uses a single-file architecture in `src/main.rs` with the following key components:

- **App struct**: Main application state container
  - `current_section: SelectedSection` - tracks which section is currently focused
  - `exit: bool` - controls the main event loop

- **SelectedSection enum**: Represents the six dashboard sections (NewCrates, MostDownloaded, JustUpdated, RecentDownloads, Keywords, Categories)

### UI Layout Pattern

The UI uses a nested layout approach:

1. **Outer container**: A `Block` with thick borders containing the title ("crates.io") and instructions
2. **Inner grid**: A 2x3 grid of sections created using nested `Layout` calls

**Key method**: `create_layout(&self, area: Rect) -> [[Rect; 3]; 2]`
- Returns a 2D array of `Rect` for positioning widgets
- First splits vertically into 2 rows (50% each)
- Then splits each row horizontally into 3 columns (33%, 34%, 33%)

**Rendering flow**:
1. `draw()` creates the title block and gets its inner area
2. `create_layout()` is called with the inner area to calculate the 2x3 grid
3. Six sections are rendered into `layout[row][col]` positions

### Event Handling

- Uses Crossterm for terminal event capture
- Key bindings:
  - `s` - Search (placeholder)
  - `i` - Info (placeholder)
  - `q` or `Esc` - Quit

## Ratatui Usage Notes

- The edition is set to "2024" (Rust 2024 edition)
- Uses `DefaultTerminal` for terminal management
- Uses `color-eyre` for error handling
- Layout calculations must account for the outer title block's borders by using `.inner()` to get the usable area
