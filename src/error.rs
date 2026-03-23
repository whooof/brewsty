//! Custom error types for Brewsty
//!
//! This module provides type-safe error handling throughout the application,
//! replacing generic `anyhow::Error` with specific error types.

use std::io;
use thiserror::Error;

/// Main error type for Brewsty
#[derive(Error, Debug)]
pub enum BrewstyError {
    /// Error when a package operation fails
    #[error("Package operation failed: {0}")]
    PackageError(String),

    /// Error when a service operation fails
    #[error("Service operation failed: {0}")]
    ServiceError(String),

    /// Error when configuration is invalid or missing
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Error when history operations fail
    #[error("History operation failed: {0}")]
    HistoryError(String),

    /// Error when plugin operations fail
    #[error("Plugin error: {0}")]
    PluginError(String),

    /// Error when WASM plugin loading fails
    #[error("WASM plugin error: {0}")]
    WasmPluginError(String),

    /// Error when file operations fail
    #[error("File operation failed: {0}")]
    FileError(#[from] io::Error),

    /// Error when JSON serialization/deserialization fails
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Error when YAML serialization/deserialization fails
    #[error("YAML error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    /// Error when HTTP requests fail
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    /// Error when a package is not found
    #[error("Package not found: {0}")]
    PackageNotFound(String),

    /// Error when an operation is not supported
    #[error("Operation not supported: {0}")]
    UnsupportedOperation(String),

    /// Generic error for cases not covered by specific variants
    #[error("Error: {0}")]
    Other(String),
}

impl BrewstyError {
    /// Create a package error
    pub fn package(msg: impl Into<String>) -> Self {
        Self::PackageError(msg.into())
    }

    /// Create a service error
    pub fn service(msg: impl Into<String>) -> Self {
        Self::ServiceError(msg.into())
    }

    /// Create a config error
    pub fn config(msg: impl Into<String>) -> Self {
        Self::ConfigError(msg.into())
    }

    /// Create a history error
    pub fn history(msg: impl Into<String>) -> Self {
        Self::HistoryError(msg.into())
    }

    /// Create a plugin error
    pub fn plugin(msg: impl Into<String>) -> Self {
        Self::PluginError(msg.into())
    }

    /// Create a WASM plugin error
    pub fn wasm_plugin(msg: impl Into<String>) -> Self {
        Self::WasmPluginError(msg.into())
    }

    /// Create a file error
    pub fn file(msg: impl Into<String>) -> Self {
        Self::FileError(io::Error::other(msg.into()))
    }

    /// Create a not found error
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::PackageNotFound(msg.into())
    }

    /// Create an unsupported operation error
    pub fn unsupported(msg: impl Into<String>) -> Self {
        Self::UnsupportedOperation(msg.into())
    }
}

/// Result type alias using BrewstyError
pub type Result<T> = std::result::Result<T, BrewstyError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_error() {
        let err = BrewstyError::package("test error");
        assert!(err.to_string().contains("Package operation failed"));
        assert!(err.to_string().contains("test error"));
    }

    #[test]
    fn test_service_error() {
        let err = BrewstyError::service("service down");
        assert!(err.to_string().contains("Service operation failed"));
    }

    #[test]
    fn test_config_error() {
        let err = BrewstyError::config("invalid config");
        assert!(err.to_string().contains("Configuration error"));
    }

    #[test]
    fn test_history_error() {
        let err = BrewstyError::history("history corrupted");
        assert!(err.to_string().contains("History operation failed"));
    }

    #[test]
    fn test_plugin_error() {
        let err = BrewstyError::plugin("plugin crashed");
        assert!(err.to_string().contains("Plugin error"));
    }

    #[test]
    fn test_wasm_plugin_error() {
        let err = BrewstyError::wasm_plugin("WASM load failed");
        assert!(err.to_string().contains("WASM plugin error"));
    }

    #[test]
    fn test_not_found_error() {
        let err = BrewstyError::not_found("git");
        assert!(err.to_string().contains("Package not found"));
        assert!(err.to_string().contains("git"));
    }

    #[test]
    fn test_unsupported_error() {
        let err = BrewstyError::unsupported("rollback");
        assert!(err.to_string().contains("Operation not supported"));
    }

    #[test]
    fn test_result_alias() {
        let result: Result<()> = Ok(());
        assert!(result.is_ok());

        let result: Result<()> = Err(BrewstyError::package("test"));
        assert!(result.is_err());
    }

    #[test]
    fn test_error_display() {
        let err = BrewstyError::Other("generic error".to_string());
        assert_eq!(err.to_string(), "Error: generic error");
    }
}
