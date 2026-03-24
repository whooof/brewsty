pub mod app;
pub mod brewfile;
pub mod config;
pub mod doctor;
pub mod history;
pub mod package;
pub mod package_category;
pub mod package_list;
pub mod service;

pub use app::{AppError, CommandResult, LoadState, MessageSeverity, OperationState, UserMessage};
pub use brewfile::BrewfileSyncPreview;
pub use config::{AppConfig, ThemeMode};
pub use doctor::{DoctorOutput, DoctorWarning};
pub use history::{OperationHistory, OperationRecord, OperationType};
pub use package::{CleanupItem, CleanupPreview, Package, PackageType};
#[allow(unused_imports)] // Used in tests and other modules
pub use package_category::PackageCategory;
pub use package_list::{PackageList, PackageListItem};
pub use service::{Service, ServiceInfo, ServiceStatus};
