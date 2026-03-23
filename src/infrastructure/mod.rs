//! Infrastructure layer - concrete implementations of repositories and external services.
//!
//! This module provides concrete implementations of the repository traits defined
//! in the domain layer, handling all external I/O and framework-specific concerns.
//!
//! ## Structure
//!
//! - [`brew`]: Homebrew-specific repository implementations
//! - [`config_repository`]: Application configuration storage
//! - [`history_repository`]: Operation history tracking

pub mod brew;
pub mod config_repository;
pub mod history_repository;
