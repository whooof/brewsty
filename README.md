# Brusty 🍺

A modern, GUI-based Homebrew package manager for macOS written in Rust.

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
  - Real-time status updates

## Architecture

Brusty follows clean architecture principles with SOLID design:

```
src/
├── domain/           # Core business logic
│   ├── entities/     # Domain models (Package, PackageType, CacheInfo)
│   ├── repositories/ # Repository interfaces
│   └── services/     # Domain services (validation)
├── infrastructure/   # External integrations
│   └── brew/         # Homebrew command execution
├── application/      # Use cases
│   ├── dto/          # Data transfer objects
│   └── use_cases/    # Application business logic
└── presentation/     # GUI layer
    ├── components/   # Reusable UI components
    └── ui/           # Application views
```

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run --release
```

## Requirements

- macOS
- Homebrew installed
- Rust 1.70+

## License

MIT
