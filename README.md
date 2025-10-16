# Craters

A terminal-based UI application for browsing [crates.io](https://crates.io), built with [Ratatui](https://ratatui.rs/).

_NOTE_: very initial project; not even "alpha"

## Overview

Craters provides an interactive dashboard for exploring the Rust package ecosystem directly from your terminal. Browse newly published crates, discover popular packages, and stay up-to-date with the latest releases—all in a sleek TUI interface.

## Features

- **Dashboard Layout**: 2x3 grid displaying six different views of the crates ecosystem
  - New Crates
  - Just Updated
  - Most Downloaded
  - Most Recent Downloads
  - Popular Keywords
  - Popular Categories
- **Real-time Data**: Fetches live data from the crates.io API
- **Keyboard Navigation**: Vim-style keybindings for efficient navigation

## Installation

### Prerequisites

- Rust 1.80+ (uses Rust 2024 edition)

### Building from Source

```bash
git clone https://github.com/raysuliteanu/craters
cd craters
cargo build --release
```

The binary will be available at `target/release/craters`.

## Usage

Run the application:

```bash
cargo run
```

Or in release mode for better performance:

```bash
cargo run --release
```

### Keyboard Shortcuts

| Key          | Action                        |
| ------------ | ----------------------------- |
| `s`          | Search crates (coming soon)   |
| `i`          | View crate info (coming soon) |
| `q` or `Esc` | Quit application              |

## Development

### Project Structure

```
craters/
├── src/
│   ├── main.rs         # Main application logic and UI
│   └── http_client.rs  # HTTP client for crates.io API
├── Cargo.toml
└── README.md
```

### Architecture

The application follows a clean architecture with:

- **App struct**: Main application state container managing the event loop and UI rendering
- **HttpClient**: HTTP client wrapper for interacting with the crates.io API
- **2x3 Grid Layout**: Uses Ratatui's nested layout system to create a responsive dashboard

### Key Dependencies

- [ratatui](https://github.com/ratatui-org/ratatui) v0.29.0 - Terminal UI framework
- [crossterm](https://github.com/crossterm-rs/crossterm) v0.29.0 - Terminal manipulation
- [reqwest](https://github.com/seanmonstar/reqwest) v0.12.24 - HTTP client
- [serde](https://github.com/serde-rs/serde) v1.0.228 - Serialization/deserialization
- [color-eyre](https://github.com/eyre-rs/color-eyre) v0.6.5 - Error handling

### Running Tests

```bash
cargo test
```

### Code Checks

```bash
# Check for compilation errors
cargo check

# Run clippy for linting
cargo clippy

# Format code
cargo fmt
```

## Roadmap

- [ ] Implement search functionality
- [ ] Add detailed crate information view
- [ ] Add keyboard navigation between sections
- [ ] Implement scrolling within sections
- [ ] Add filtering and sorting options
- [ ] Cache API responses for better performance
- [ ] Add support for viewing crate dependencies
- [ ] Implement color themes

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## License

This project is open source and available under the MIT License.

## Acknowledgments

- Built with [Ratatui](https://ratatui.rs/) - An amazing Rust TUI framework
- Data provided by the [crates.io API](https://crates.io/data-access)
