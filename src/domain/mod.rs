//! Domain layer - core business entities and repository contracts.
//!
//! This module defines the core business model of the application, independent
//! of any framework or infrastructure concerns.
//!
//! ## Structure
//!
//! - [`entities`]: Business entities like Package, Service, Config
//! - [`repositories`]: Traits defining the data access layer contracts

pub mod entities;
pub mod repositories;
