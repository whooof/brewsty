use crate::domain::{
    entities::{Service, ServiceStatus},
    repositories::ServiceRepository,
};
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
        let output = tokio::task::spawn_blocking(BrewCommand::list_services).await??;
        self.parse_services_list(&output)
    }

    async fn start_service(&self, name: &str) -> Result<()> {
        let name = name.to_string();
        let output =
            tokio::task::spawn_blocking(move || BrewCommand::start_service(&name)).await??;

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
        let output =
            tokio::task::spawn_blocking(move || BrewCommand::stop_service(&name)).await??;

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
        let output =
            tokio::task::spawn_blocking(move || BrewCommand::restart_service(&name)).await??;

        if !output.stdout.is_empty() {
            tracing::info!("restart_service output: {}", output.stdout);
        }
        if !output.stderr.is_empty() {
            tracing::info!("restart_service stderr: {}", output.stderr);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo() -> BrewServiceRepository {
        BrewServiceRepository::new()
    }

    #[test]
    fn parse_service_status_started() {
        assert_eq!(
            BrewServiceRepository::parse_service_status("started"),
            ServiceStatus::Started
        );
        assert_eq!(
            BrewServiceRepository::parse_service_status("Started"),
            ServiceStatus::Started
        );
    }

    #[test]
    fn parse_service_status_stopped() {
        assert_eq!(
            BrewServiceRepository::parse_service_status("stopped"),
            ServiceStatus::Stopped
        );
        assert_eq!(
            BrewServiceRepository::parse_service_status("none"),
            ServiceStatus::Stopped
        );
    }

    #[test]
    fn parse_service_status_error() {
        assert_eq!(
            BrewServiceRepository::parse_service_status("error"),
            ServiceStatus::Error
        );
    }

    #[test]
    fn parse_service_status_unknown() {
        assert_eq!(
            BrewServiceRepository::parse_service_status("something"),
            ServiceStatus::Unknown
        );
    }

    #[test]
    fn parse_services_list_full() {
        let output = "Name       Status  User   File\npostgresql started whooof /usr/local/opt/postgresql/homebrew.postgresql.service.plist\nredis      stopped\nnginx      started root   /Library/LaunchDaemons/homebrew.mxcl.nginx.plist\n";
        let result = repo().parse_services_list(output).unwrap();
        assert_eq!(result.len(), 3);

        assert_eq!(result[0].name, "postgresql");
        assert_eq!(result[0].status, ServiceStatus::Started);
        assert_eq!(result[0].user.as_deref(), Some("whooof"));
        assert!(result[0].file.is_some());

        assert_eq!(result[1].name, "redis");
        assert_eq!(result[1].status, ServiceStatus::Stopped);
        assert!(result[1].user.is_none());

        assert_eq!(result[2].name, "nginx");
        assert_eq!(result[2].status, ServiceStatus::Started);
        assert_eq!(result[2].user.as_deref(), Some("root"));
    }

    #[test]
    fn parse_services_list_empty() {
        let output = "Name  Status  User  File\n";
        let result = repo().parse_services_list(output).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn parse_services_list_header_only() {
        let output = "Name  Status  User  File\n\n";
        let result = repo().parse_services_list(output).unwrap();
        assert!(result.is_empty());
    }
}
