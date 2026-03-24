//! # Brewsty
//!
//! A comprehensive Homebrew package manager with GUI and CLI interfaces.
//!
//! ## Features
//!
//! - **GUI Application**: Full-featured interface with 6 tabs for package management
//! - **CLI Application**: Command-line tool with 10 commands for automation
//! - **Plugin System**: Architecture for extending functionality via WASM plugins
//! - **Statistics Dashboard**: Visual charts and package analytics
//!
//! ## Architecture
//!
//! Brewsty follows a clean architecture pattern:
//!
//! - [`application`]: Use cases and business logic
//! - [`domain`]: Entities and repository traits
//! - [`infrastructure`]: Concrete repository implementations
//! - [`cli`]: Command-line interface
//! - [`plugins`]: Plugin system for extensibility
//! - [`error`]: Custom error types
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! // Create container with all use cases
//! use brewsty::application::use_case_container::UseCaseContainer;
//! use brewsty::infrastructure::brew::{BrewPackageRepository, BrewServiceRepository, BrewPackageListRepository};
//! use brewsty::infrastructure::history_repository::FileHistoryRepository;
//! use std::sync::Arc;
//!
//! let package_repo = Arc::new(BrewPackageRepository::new());
//! let service_repo = Arc::new(BrewServiceRepository::new());
//! let package_list_repo = Arc::new(BrewPackageListRepository::new());
//! let history_repo = Arc::new(FileHistoryRepository::new());
//!
//! let container = UseCaseContainer::new(package_repo, service_repo, package_list_repo, history_repo);
//! ```

pub mod application;
pub mod cli;
pub mod domain;
pub mod error;
pub mod infrastructure;
pub mod plugins;

pub use error::{BrewstyError, Result};
