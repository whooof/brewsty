use crate::domain::{
    entities::Service,
    repositories::ServiceRepository,
};
use anyhow::Result;
use std::sync::Arc;

macro_rules! service_use_case {
    ($name:ident, $method:ident -> $ret:ty) => {
        pub struct $name { repository: Arc<dyn ServiceRepository> }
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
        pub struct $name { repository: Arc<dyn ServiceRepository> }
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
