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

# Run clippy for linting
cargo clippy

# Format code
cargo fmt

# Run all tests
cargo test

# Run a specific test
cargo test test_name

# Run tests for a specific module
cargo test crates_io_client
```

## Architecture

### Application Structure

The application consists of two main modules:

- **`src/main.rs`**: Main application logic, UI rendering, and event handling
- **`src/crates_io_client.rs`**: HTTP client wrapper for crates.io API

**App struct** - Main application state container:
- `client: HttpClient` - wrapper around crates_io_api AsyncClient
- `summary: Summary` - cached API response with crate lists
- `current_section: SelectedSection` - tracks which section is currently focused
- `state: HashMap<SelectedSection, ListState>` - manages list selection state per section
- `crates: HashMap<String, CrateResponse>` - cache for detailed crate info
- `exit: bool` - controls the main event loop
- `search: bool` - controls search mode (not yet implemented)
- `info: bool` - controls info popup display

**SelectedSection enum** - Represents the six dashboard sections:
- Uses `strum` derives for iteration and `FromRepr` conversion
- Order matters: defines next/previous navigation sequence
- Sequence: NewCrates → MostDownloaded → JustUpdated → RecentDownloads → PopularKeywords → PopularCategories

### HttpClient Module

Located in `src/crates_io_client.rs`, this module wraps the `crates_io_api::AsyncClient`:

- **`new()`** - Creates client with user agent and 1s timeout
- **`fetch_summary()`** - Gets dashboard data (new crates, most downloaded, etc.)
- **`fetch_crate_info(crate_name)`** - Gets detailed info for a specific crate
- **`search(query)`** - Text search across all crates
- **`search_categories(category)`** - Search by category
- **`search_keywords(keyword)`** - Search by keyword

All methods are async and return `Result<T, CratesIoError>`.

### UI Layout Pattern

The UI uses a nested layout approach:

1. **Outer container**: A `Block` with thick borders containing the title ("crates.io") and instructions
2. **Inner grid**: A 2x3 grid of sections created using nested `Layout` calls

**Key method**: `create_layout(&self, area: Rect) -> [[Rect; 3]; 2]`
- Returns a 2D array of `Rect` for positioning widgets
- First splits vertically into 2 rows (50% each)
- Then splits each row horizontally into 3 columns (33%, 34%, 33%)

**Rendering flow**:
1. `draw()` creates the title block and gets its inner area using `.inner()`
2. `create_layout()` is called with the inner area to calculate the 2x3 grid
3. Six sections are rendered into `layout[row][col]` positions
4. Each section uses a `List` widget with a `ListState` for selection highlighting

### State Management

- Each section has its own `ListState` stored in `HashMap<SelectedSection, ListState>`
- `ListState` tracks the currently selected item (highlighted in blue/bold)
- Navigation methods (`select_next()`, `select_previous()`) update the active section's state
- Section navigation wraps around (e.g., left from first section goes to last section)
- All sections initialize with the first item selected (index 0)

### Event Handling

Uses Crossterm for terminal event capture with the following key bindings:

**Section Navigation**:
- `h` / `Left` / `Shift-Tab` - Previous section
- `l` / `Right` / `Tab` - Next section

**Item Navigation (within section)**:
- `j` / `Down` - Next item
- `k` / `Up` - Previous item

**Actions**:
- `i` - Show info popup for selected crate
- `s` - Search (not yet implemented)
- `q` / `Esc` - Quit application (or close info popup if open)

### Info Popup System

Pressing `i` displays a centered popup with details about the selected crate:

- Uses `popup_area()` helper to calculate centered Rect (60% width, 60% height)
- Renders using `Clear` widget to clear background, then `Paragraph` widget
- Shows: crate name (bold), version, description, and keywords (green with # prefix)
- Currently only works for crate sections (NewCrates, MostDownloaded, JustUpdated, RecentDownloads)
- Keywords and Categories sections show "not implemented yet" message
- Pressing `q` or `Esc` closes the popup without exiting the application

### Async Runtime and Logging

- Uses Tokio async runtime (`#[tokio::main]`)
- All API calls through HttpClient are async
- Logging configured with `simplelog` crate at Debug level
- Logs written to `craters.log` in the working directory
- Use `log::debug!()` macro for debug logging

## Ratatui Usage Notes

- The edition is set to "2024" (Rust 2024 edition)
- Uses `DefaultTerminal` for terminal management (via `ratatui::init()` and `ratatui::restore()`)
- Uses `anyhow` for error handling
- Layout calculations must account for the outer title block's borders by using `.inner()` to get the usable area
- Stateful widgets (like `List`) require both the widget and a mutable `&mut ListState` to be rendered
