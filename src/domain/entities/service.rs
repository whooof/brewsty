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
}

impl Service {
    pub fn new(name: String, status: ServiceStatus) -> Self {
        Self {
            name,
            status,
            user: None,
            file: None,
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
}
