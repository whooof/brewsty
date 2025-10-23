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


<img width="1312" height="940" alt="obraz" src="https://github.com/user-attachments/assets/21d89f14-b028-41bf-9022-fb9f9e8f1328" />
<img width="1312" height="940" alt="obraz" src="https://github.com/user-attachments/assets/556c4a1f-31f1-4746-a1a6-86f07ef23ee0" />
<img width="1312" height="940" alt="obraz" src="https://github.com/user-attachments/assets/46c739fb-7d1d-4a7c-8030-6b63edb992dc" />


- 📦 **Package Management**
  - View installed formulae and casks
  - Check for outdated packages
  - Install, uninstall, and update packages
  - Search for available packages
  - Pin and unpin packages to prevent updates

- 🧹 **Maintenance**
  - Clean package cache
  - Remove old package versions
  - Update all packages at once
  - Sequential updates: Update packages one at a time with progress tracking

- 🔐 **Security**
  - Password authentication modal for install/uninstall operations
  - Automatic password error detection and retry mechanism
  - Secure password input (hidden field with show/hide toggle)

- 🎨 **Modern UI**
  - Clean, intuitive interface built with egui
  - Tab-based navigation
  - Async operations with responsive UI
  - Real-time loading indicators for package operations

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
- **Sequential Operations**: Long-running updates process packages one at a time with UI feedback
- **Graceful Auth Handling**: Automatic detection of password requirements with user-friendly modal

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

### Running the Application

```bash
cargo run --release
```

### Debug Logging

By default, the application shows INFO level logs in release builds and DEBUG level logs in debug builds. To enable verbose TRACE level logging for debugging:

```bash
cargo run --features verbose-logging
```

**Feature Flags:**
- `verbose-logging` - Enables TRACE level logging (useful for debugging)
- Default (no flags) - INFO level (release) or DEBUG level (debug)

The application provides four main tabs:

- **Installed**: Browse all installed formulae and casks with version info
- **Outdated**: See packages that need updates and upgrade them
  - Select multiple packages for sequential updating
  - Each package updates one at a time with progress tracking (e.g., "Updating 2/5: package-name...")
  - Select All / Deselect All buttons for batch operations
- **Search**: Find and install new packages from Homebrew
- **Maintenance**: Clean cache and remove old versions

### Sequential Package Updates

When updating multiple packages, Brewsty processes them sequentially rather than all at once. This approach:
- Reduces system load and improves stability
- Provides clear progress feedback in the UI
- Prevents overwhelming Homebrew with concurrent requests
- Updates the package list locally after each successful update for instant feedback

### Password Authentication

If an operation (install, uninstall, update) requires administrator privileges:
1. The app attempts the operation normally first
2. If a password is required, a modal dialog appears
3. Enter your administrator password in the secure input field
4. The operation retries with the provided password
5. On failure, you can try again or cancel

The password input field includes a "Show password" toggle for verification.

## Technologies

- **[Rust](https://www.rust-lang.org/)** - Systems programming language
- **[egui](https://github.com/emilk/egui)** - Immediate mode GUI framework
- **[eframe](https://github.com/emilk/egui/tree/master/crates/eframe)** - Native GUI application framework
- **[Tokio](https://tokio.rs/)** - Async runtime for non-blocking operations
- **[Homebrew](https://brew.sh/)** - macOS package manager
- **[egui-winit](https://github.com/emilk/egui/tree/master/crates/egui-winit)** - Window integration for egui

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License (Tortilla Edition 🌯)

See [LICENSE.md](LICENSE.md) for details.

## Acknowledgments

Built with [egui](https://github.com/emilk/egui) by [@emilk](https://github.com/emilk)

