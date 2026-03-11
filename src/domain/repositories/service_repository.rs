use crate::domain::entities::{Service, ServiceInfo};
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait ServiceRepository: Send + Sync {
    async fn list_services(&self) -> Result<Vec<Service>>;
    async fn start_service(&self, name: &str) -> Result<()>;
    async fn stop_service(&self, name: &str) -> Result<()>;
    async fn restart_service(&self, name: &str) -> Result<()>;
    async fn service_info(&self, name: &str) -> Result<ServiceInfo>;
    async fn service_log(&self, name: &str, tail_lines: usize) -> Result<String>;
}
