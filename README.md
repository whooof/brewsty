# Brewsty 🍺

A modern, GUI-based Homebrew package manager for macOS written in Rust.

## ⚠️ Warning: AI-Generated Learning Project

This codebase was largely pair-programmed with AI models (Claude, ChatGPT, etc.). Treat it as:
- A sandbox for learning and architecture experiments
- A case study in what AIs get right—and almost right
- A living demo of clean architecture in Rust

Disclaimer: It compiles, it runs, and it manages Homebrew, but parts were authored by statistically enthusiastic robots. Expect the occasional off-by-one joke, an overconfident TODO, or a function that’s suspiciously clever at 3am.

Meta-note: Yes, an AI wrote this warning about AI-written code. Self-reference acknowledged; recursion depth limited; stack intact.

Use, fork, and learn freely. If you find a bug, blame the robots, keep the credit.

## Features

- 📦 **Package Management**
  - View installed formulae and casks
  - Check for outdated packages
  - Install, uninstall, and update packages
  - Search for available packages

- 🧹 **Maintenance**
  - Clean package cache
  - Remove old package versions
  - Update all packages at once

- 🎨 **Modern UI**
  - Clean, intuitive interface built with egui
  - Tab-based navigation
  - Async operations with responsive UI

## Architecture

Brewsty follows clean architecture principles with SOLID design:

```
src/
├── domain/           # Core business logic
│   ├── entities/     # Domain models (Package, PackageType, CacheInfo)
│   └── repositories/ # Repository interfaces
├── infrastructure/   # External integrations
│   └── brew/         # Homebrew command execution
├── application/      # Use cases
│   ├── dto/          # Data transfer objects
│   └── use_cases/    # Application business logic
└── presentation/     # GUI layer
    ├── components/   # Reusable UI components
    ├── services/     # Async task management
    └── ui/           # Application views
```

### Key Design Principles

- **Single Responsibility**: Each module has one clear purpose
- **Dependency Inversion**: Use cases depend on repository abstractions
- **Interface Segregation**: Clean, focused trait definitions
- **Testability**: Repository pattern allows easy mocking and testing

## Prerequisites

- macOS
- [Homebrew](https://brew.sh/) installed
- Rust toolchain (install via [rustup](https://rustup.rs/))
- Xcode Command Line Tools (`xcode-select --install`)

## Installation

```bash
git clone https://github.com/yourusername/brewsty.git
cd brewsty
cargo build --release
```

## Usage

```bash
cargo run --release
```

The application provides four main tabs:

- **Installed**: Browse all installed formulae and casks with version info
- **Outdated**: See packages that need updates and upgrade them
- **Search**: Find and install new packages from Homebrew
- **Maintenance**: Clean cache and remove old versions

## Technologies

- **[Rust](https://www.rust-lang.org/)** - Systems programming language
- **[egui](https://github.com/emilk/egui)** - Immediate mode GUI framework
- **[eframe](https://github.com/emilk/egui/tree/master/crates/eframe)** - Native GUI application framework
- **[Tokio](https://tokio.rs/)** - Async runtime for non-blocking operations
- **[Homebrew](https://brew.sh/)** - macOS package manager

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License (Tortilla Edition 🌯)

See [LICENSE.md](LICENSE.md) for details.

## Acknowledgments

Built with [egui](https://github.com/emilk/egui) by [@emilk](https://github.com/emilk)

