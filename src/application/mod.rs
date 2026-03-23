//! Application layer - business logic and use cases.
//!
//! This module contains the application's business logic, organized as use cases.
//! Each use case represents a specific operation that can be performed by the application.
//!
//! ## Structure
//!
//! - [`dto`]: Data transfer objects for inter-layer communication
//! - [`use_case_container`]: DI container that wires up all use cases
//! - [`use_cases`]: Individual use case implementations

pub mod dto;
pub mod use_case_container;
pub mod use_cases;

pub use use_case_container::UseCaseContainer;
