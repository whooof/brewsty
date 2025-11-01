use crate::domain::{entities::{Service, ServiceStatus}, repositories::ServiceRepository};
use crate::infrastructure::brew::command::BrewCommand;
use anyhow::Result;
use async_trait::async_trait;

pub struct BrewServiceRepository;

impl BrewServiceRepository {
    pub fn new() -> Self {
        Self
    }

    fn parse_service_status(status_str: &str) -> ServiceStatus {
        let status_lower = status_str.to_lowercase();
        if status_lower.contains("started") {
            ServiceStatus::Started
        } else if status_lower.contains("stopped") || status_lower.contains("none") {
            ServiceStatus::Stopped
        } else if status_lower.contains("error") {
            ServiceStatus::Error
        } else {
            ServiceStatus::Unknown
        }
    }

    fn parse_services_list(&self, output: &str) -> Result<Vec<Service>> {
        let mut services = Vec::new();

        for (index, line) in output.lines().enumerate() {
            // Skip header line
            if index == 0 || line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let name = parts[0].to_string();
                let status_str = parts[1];
                let status = Self::parse_service_status(status_str);

                let mut service = Service::new(name, status);

                // Try to extract user if present (format: name status user file)
                if parts.len() >= 3 {
                    service = service.with_user(parts[2].to_string());
                }

                // Try to extract file if present
                if parts.len() >= 4 {
                    service = service.with_file(parts[3].to_string());
                }

                services.push(service);
            }
        }

        Ok(services)
    }
}

#[async_trait]
impl ServiceRepository for BrewServiceRepository {
    async fn list_services(&self) -> Result<Vec<Service>> {
        let output = tokio::task::spawn_blocking(|| BrewCommand::list_services()).await??;
        self.parse_services_list(&output)
    }

    async fn start_service(&self, name: &str) -> Result<()> {
        let name = name.to_string();
        let output = tokio::task::spawn_blocking(move || BrewCommand::start_service(&name)).await??;

        if !output.stdout.is_empty() {
            tracing::info!("start_service output: {}", output.stdout);
        }
        if !output.stderr.is_empty() {
            tracing::info!("start_service stderr: {}", output.stderr);
        }

        Ok(())
    }

    async fn stop_service(&self, name: &str) -> Result<()> {
        let name = name.to_string();
        let output = tokio::task::spawn_blocking(move || BrewCommand::stop_service(&name)).await??;

        if !output.stdout.is_empty() {
            tracing::info!("stop_service output: {}", output.stdout);
        }
        if !output.stderr.is_empty() {
            tracing::info!("stop_service stderr: {}", output.stderr);
        }

        Ok(())
    }

    async fn restart_service(&self, name: &str) -> Result<()> {
        let name = name.to_string();
        let output = tokio::task::spawn_blocking(move || BrewCommand::restart_service(&name)).await??;

        if !output.stdout.is_empty() {
            tracing::info!("restart_service output: {}", output.stdout);
        }
        if !output.stderr.is_empty() {
            tracing::info!("restart_service stderr: {}", output.stderr);
        }

        Ok(())
    }
}
