use crate::domain::{
    entities::{Service, ServiceInfo},
    repositories::ServiceRepository,
};
use anyhow::Result;
use std::sync::Arc;

macro_rules! service_use_case {
    ($name:ident, $method:ident -> $ret:ty) => {
        pub struct $name {
            repository: Arc<dyn ServiceRepository>,
        }
        impl $name {
            pub fn new(repository: Arc<dyn ServiceRepository>) -> Self {
                Self { repository }
            }
            pub async fn execute(&self) -> Result<$ret> {
                self.repository.$method().await
            }
        }
    };
    ($name:ident, $method:ident(&str) -> $ret:ty) => {
        pub struct $name {
            repository: Arc<dyn ServiceRepository>,
        }
        impl $name {
            pub fn new(repository: Arc<dyn ServiceRepository>) -> Self {
                Self { repository }
            }
            pub async fn execute(&self, name: &str) -> Result<$ret> {
                self.repository.$method(name).await
            }
        }
    };
}

service_use_case!(ListServices, list_services -> Vec<Service>);
service_use_case!(StartService, start_service(&str) -> ());
service_use_case!(StopService, stop_service(&str) -> ());
service_use_case!(RestartService, restart_service(&str) -> ());
service_use_case!(GetServiceInfo, service_info(&str) -> ServiceInfo);

pub struct GetServiceLog {
    repository: Arc<dyn ServiceRepository>,
}

impl GetServiceLog {
    pub fn new(repository: Arc<dyn ServiceRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, name: &str, tail_lines: usize) -> Result<String> {
        self.repository.service_log(name, tail_lines).await
    }
}
