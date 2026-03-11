use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceStatus {
    Started,
    Stopped,
    Error,
    Unknown,
}

impl ServiceStatus {
    pub fn is_running(&self) -> bool {
        matches!(self, ServiceStatus::Started)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub name: String,
    pub status: ServiceStatus,
    pub user: Option<String>,
    pub file: Option<String>,
    pub exit_code: Option<i32>,
    pub pid: Option<u32>,
    pub registered: bool,
    pub loaded: bool,
    pub log_path: Option<String>,
    pub error_log_path: Option<String>,
    pub command: Option<String>,
}

impl Service {
    pub fn new(name: String, status: ServiceStatus) -> Self {
        Self {
            name,
            status,
            user: None,
            file: None,
            exit_code: None,
            pid: None,
            registered: false,
            loaded: false,
            log_path: None,
            error_log_path: None,
            command: None,
        }
    }

    pub fn with_user(mut self, user: String) -> Self {
        self.user = Some(user);
        self
    }

    pub fn with_file(mut self, file: String) -> Self {
        self.file = Some(file);
        self
    }

    pub fn with_exit_code(mut self, code: i32) -> Self {
        self.exit_code = Some(code);
        self
    }

    /// Returns a boot status label for display.
    pub fn boot_label(&self) -> &str {
        if self.registered {
            "Login"
        } else if self.status.is_running() {
            "Manual"
        } else {
            "None"
        }
    }
}

/// Detailed info from `brew services info --json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub service_name: String,
    pub running: bool,
    pub loaded: bool,
    pub pid: Option<u32>,
    pub exit_code: Option<i32>,
    pub user: Option<String>,
    pub status: ServiceStatus,
    pub file: Option<String>,
    pub registered: bool,
    pub log_path: Option<String>,
    pub error_log_path: Option<String>,
    pub command: Option<String>,
}
