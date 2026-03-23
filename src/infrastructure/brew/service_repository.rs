use crate::domain::{
    entities::{AppError, Service, ServiceInfo, ServiceStatus},
    repositories::ServiceRepository,
};
use crate::infrastructure::brew::command::BrewCommand;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use serde::Deserialize;

pub struct BrewServiceRepository;

/// Raw JSON entry from `brew services list --json`.
#[derive(Debug, Deserialize)]
struct ServiceListEntry {
    name: String,
    status: Option<String>,
    user: Option<String>,
    file: Option<String>,
    exit_code: Option<i32>,
}

/// Raw JSON entry from `brew services info <name> --json`.
/// The command returns an array with a single element.
#[derive(Debug, Deserialize)]
struct ServiceInfoEntry {
    name: String,
    service_name: Option<String>,
    running: Option<bool>,
    loaded: Option<bool>,
    pid: Option<u32>,
    exit_code: Option<i32>,
    user: Option<String>,
    status: Option<String>,
    file: Option<String>,
    registered: Option<bool>,
    log_path: Option<String>,
    error_log_path: Option<String>,
    command: Option<String>,
}

impl BrewServiceRepository {
    pub fn new() -> Self {
        Self
    }

    fn parse_status(s: &str) -> ServiceStatus {
        match s.to_lowercase().as_str() {
            "started" => ServiceStatus::Started,
            "stopped" | "none" => ServiceStatus::Stopped,
            "error" => ServiceStatus::Error,
            _ => ServiceStatus::Unknown,
        }
    }

    fn parse_services_json(json: &str) -> Result<Vec<Service>> {
        let entries: Vec<ServiceListEntry> = serde_json::from_str(json)
            .map_err(|e| anyhow!("Failed to parse services JSON: {}", e))?;

        let services = entries
            .into_iter()
            .map(|e| {
                let status = e
                    .status
                    .as_deref()
                    .map(Self::parse_status)
                    .unwrap_or(ServiceStatus::Unknown);

                let mut svc = Service::new(e.name, status);
                if let Some(u) = e.user {
                    svc = svc.with_user(u);
                }
                if let Some(f) = e.file {
                    svc = svc.with_file(f);
                }
                if let Some(code) = e.exit_code {
                    svc = svc.with_exit_code(code);
                }
                svc
            })
            .collect();

        Ok(services)
    }

    fn parse_service_info_json(json: &str) -> Result<ServiceInfo> {
        let entries: Vec<ServiceInfoEntry> = serde_json::from_str(json)
            .map_err(|e| anyhow!("Failed to parse service info JSON: {}", e))?;

        let entry = entries
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("Empty service info response"))?;

        let status = entry
            .status
            .as_deref()
            .map(Self::parse_status)
            .unwrap_or(ServiceStatus::Unknown);

        Ok(ServiceInfo {
            name: entry.name,
            service_name: entry.service_name.unwrap_or_default(),
            running: entry.running.unwrap_or(false),
            loaded: entry.loaded.unwrap_or(false),
            pid: entry.pid,
            exit_code: entry.exit_code,
            user: entry.user,
            status,
            file: entry.file,
            registered: entry.registered.unwrap_or(false),
            log_path: entry.log_path,
            error_log_path: entry.error_log_path,
            command: entry.command,
        })
    }

    /// Fallback: parse text-based `brew services list` output (used if JSON fails).
    fn parse_services_text(output: &str) -> Result<Vec<Service>> {
        let mut services = Vec::new();

        for (index, line) in output.lines().enumerate() {
            // Skip header line
            if index == 0 || line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let name = parts[0].to_string();
                let status = Self::parse_status(parts[1]);
                let mut service = Service::new(name, status);

                if parts.len() >= 3 {
                    service = service.with_user(parts[2].to_string());
                }
                if parts.len() >= 4 {
                    service = service.with_file(parts[3].to_string());
                }

                services.push(service);
            }
        }

        Ok(services)
    }

    fn service_command_error(
        name: &str,
        error: &anyhow::Error,
        stderr: impl Into<String>,
    ) -> anyhow::Error {
        let stderr = stderr.into();
        let (command, exit_code) = match error.downcast_ref::<AppError>() {
            Some(AppError::CommandFailed {
                command, exit_code, ..
            }) => (command.clone(), *exit_code),
            _ => (format!("brew services start {name}"), None),
        };

        AppError::CommandFailed {
            command,
            exit_code,
            stderr,
        }
        .into()
    }

    fn enrich_service_command_error(name: &str, error: anyhow::Error) -> anyhow::Error {
        let message = error.to_string();
        let lowered = message.to_lowercase();

        if lowered.contains("must be run as non-root to start at user login")
            && lowered.contains("bootstrap system")
            && lowered.contains("/library/launchdaemons/homebrew.mxcl.")
        {
            return Self::service_command_error(
                name,
                &error,
                format!(
                    "Homebrew tried to register '{name}' as a system LaunchDaemon, but this formula explicitly warns that it must run as a non-root user login service.\n\
\n\
What this means:\n\
- `launchctl bootstrap system ...` tried to load `/Library/LaunchDaemons/homebrew.mxcl.{name}.plist`\n\
- Homebrew also reported that `{name}` must not run as root at login\n\
- the current service registration is therefore in the wrong scope, not just missing a password prompt\n\
\n\
Recommended recovery:\n\
1. Stop any broken registration: `brew services stop {name}`\n\
2. If the system daemon is stuck, unload it: `sudo launchctl bootout system /Library/LaunchDaemons/homebrew.mxcl.{name}.plist`\n\
3. Decide the intended mode:\n\
   - user login service: start it from your account without a system daemon\n\
   - system-wide daemon: manage it explicitly outside the Homebrew user-login flow\n\
4. If Homebrew already took `root:admin` ownership of `{name}` paths, clean those stale paths before reinstalling/upgrading as warned by Homebrew\n\
\n\
Original output:\n{message}"
                ),
            );
        }

        if message.contains("launchctl bootstrap")
            && message.contains("exited with 5")
            && message.contains("Input/output error")
        {
            return Self::service_command_error(
                name,
                &error,
                format!(
                    "Failed to start service '{}': launchctl bootstrap returned exit code 5.\n\
This usually means the LaunchAgent is in a bad state, the plist is invalid, or macOS refused to load it.\n\
Try:\n\
1. brew services stop {}\n\
2. launchctl bootout gui/$(id -u) ~/Library/LaunchAgents/homebrew.mxcl.{}.plist\n\
3. plutil -lint ~/Library/LaunchAgents/homebrew.mxcl.{}.plist\n\
4. brew services start {}\n\
If it still fails, inspect the live state with:\n\
launchctl print gui/$(id -u)/homebrew.mxcl.{}",
                    name, name, name, name, name, name
                ),
            );
        }

        error
    }
}

#[async_trait]
impl ServiceRepository for BrewServiceRepository {
    async fn list_services(&self) -> Result<Vec<Service>> {
        // Try JSON first, fall back to text parsing
        let result = tokio::task::spawn_blocking(BrewCommand::list_services_json).await?;
        match result {
            Ok(json) => Self::parse_services_json(&json),
            Err(e) => {
                tracing::warn!(
                    "brew services list --json failed ({}), falling back to text",
                    e
                );
                let text = tokio::task::spawn_blocking(BrewCommand::list_services).await??;
                Self::parse_services_text(&text)
            }
        }
    }

    async fn start_service(&self, name: &str) -> Result<()> {
        let name = name.to_string();
        let output = match tokio::task::spawn_blocking({
            let name = name.clone();
            move || BrewCommand::start_service(&name)
        })
        .await?
        {
            Ok(output) => output,
            Err(error) => return Err(Self::enrich_service_command_error(&name, error)),
        };

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
        let output = match tokio::task::spawn_blocking({
            let name = name.clone();
            move || BrewCommand::restart_service(&name)
        })
        .await?
        {
            Ok(output) => output,
            Err(error) => return Err(Self::enrich_service_command_error(&name, error)),
        };

        if !output.stdout.is_empty() {
            tracing::info!("restart_service output: {}", output.stdout);
        }
        if !output.stderr.is_empty() {
            tracing::info!("restart_service stderr: {}", output.stderr);
        }

        Ok(())
    }

    async fn service_info(&self, name: &str) -> Result<ServiceInfo> {
        let name = name.to_string();
        let json =
            tokio::task::spawn_blocking(move || BrewCommand::service_info_json(&name)).await??;
        Self::parse_service_info_json(&json)
    }

    async fn service_log(&self, name: &str, tail_lines: usize) -> Result<String> {
        // First get the service info to find the log path
        let info = self.service_info(name).await?;

        let log_path = info
            .log_path
            .or(info.error_log_path)
            .ok_or_else(|| anyhow!("No log file found for service '{}'", name))?;

        let lines = tail_lines;
        tokio::task::spawn_blocking(move || BrewCommand::read_service_log(&log_path, lines)).await?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enriches_launchctl_bootstrap_exit_5_error() {
        let error = anyhow!(
            "Brew command failed: Bootstrap failed: 5: Input/output error\n\
Try re-running the command as root for richer errors.\n\
Error: Failure while executing; `/bin/launchctl bootstrap gui/501 /Users/test/Library/LaunchAgents/homebrew.mxcl.caddy.plist` exited with 5."
        );

        let enriched = BrewServiceRepository::enrich_service_command_error("caddy", error);
        let details = enriched
            .downcast_ref::<AppError>()
            .and_then(AppError::details)
            .unwrap_or_else(|| enriched.to_string());

        assert!(details.contains("launchctl bootstrap returned exit code 5"));
        assert!(details.contains("launchctl bootout gui/$(id -u)"));
        assert!(details.contains("plutil -lint"));
    }

    #[test]
    fn enriches_non_root_user_login_service_conflict() {
        let error = anyhow!(
            "Warning: Taking root:admin ownership of some caddy paths:\n\
  /opt/homebrew/Cellar/caddy/2.11.2/bin\n\
Warning: caddy must be run as non-root to start at user login!\n\
Bootstrap failed: 5: Input/output error\n\
Error: Failure while executing; `/bin/launchctl bootstrap system /Library/LaunchDaemons/homebrew.mxcl.caddy.plist` exited with 5."
        );

        let enriched = BrewServiceRepository::enrich_service_command_error("caddy", error);
        let details = enriched
            .downcast_ref::<AppError>()
            .and_then(AppError::details)
            .unwrap_or_else(|| enriched.to_string());

        assert!(details.contains("must run as a non-root user login service"));
        assert!(details.contains("sudo launchctl bootout system"));
        assert!(details.contains("wrong scope"));
    }

    #[test]
    fn parse_services_json_basic() {
        let json = r#"[
            {"name": "postgresql", "status": "started", "user": "whooof", "file": "/usr/local/opt/postgresql/homebrew.postgresql.service.plist", "exit_code": 0},
            {"name": "redis", "status": "stopped", "user": null, "file": null, "exit_code": null},
            {"name": "nginx", "status": "error", "user": "root", "file": "/Library/LaunchDaemons/homebrew.mxcl.nginx.plist", "exit_code": 1}
        ]"#;

        let services = BrewServiceRepository::parse_services_json(json).unwrap();
        assert_eq!(services.len(), 3);

        assert_eq!(services[0].name, "postgresql");
        assert_eq!(services[0].status, ServiceStatus::Started);
        assert_eq!(services[0].user.as_deref(), Some("whooof"));
        assert!(services[0].file.is_some());
        assert_eq!(services[0].exit_code, Some(0));

        assert_eq!(services[1].name, "redis");
        assert_eq!(services[1].status, ServiceStatus::Stopped);
        assert!(services[1].user.is_none());

        assert_eq!(services[2].name, "nginx");
        assert_eq!(services[2].status, ServiceStatus::Error);
        assert_eq!(services[2].exit_code, Some(1));
    }

    #[test]
    fn parse_services_json_empty_array() {
        let json = "[]";
        let services = BrewServiceRepository::parse_services_json(json).unwrap();
        assert!(services.is_empty());
    }

    #[test]
    fn parse_services_json_missing_optional_fields() {
        let json = r#"[{"name": "foo", "status": "stopped"}]"#;
        let services = BrewServiceRepository::parse_services_json(json).unwrap();
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].name, "foo");
        assert_eq!(services[0].status, ServiceStatus::Stopped);
        assert!(services[0].user.is_none());
        assert!(services[0].file.is_none());
        assert!(services[0].exit_code.is_none());
    }

    #[test]
    fn parse_service_info_json_full() {
        let json = r#"[{
            "name": "postgresql",
            "service_name": "homebrew.mxcl.postgresql",
            "running": true,
            "loaded": true,
            "pid": 12345,
            "exit_code": 0,
            "user": "whooof",
            "status": "started",
            "file": "/usr/local/opt/postgresql/homebrew.postgresql.service.plist",
            "registered": true,
            "log_path": "/usr/local/var/log/postgresql.log",
            "error_log_path": "/usr/local/var/log/postgresql.error.log",
            "command": "/usr/local/opt/postgresql/bin/postgres -D /usr/local/var/postgres"
        }]"#;

        let info = BrewServiceRepository::parse_service_info_json(json).unwrap();
        assert_eq!(info.name, "postgresql");
        assert_eq!(info.service_name, "homebrew.mxcl.postgresql");
        assert!(info.running);
        assert!(info.loaded);
        assert_eq!(info.pid, Some(12345));
        assert_eq!(info.exit_code, Some(0));
        assert!(info.registered);
        assert_eq!(
            info.log_path.as_deref(),
            Some("/usr/local/var/log/postgresql.log")
        );
        assert!(info.command.is_some());
    }

    #[test]
    fn parse_service_info_json_minimal() {
        let json = r#"[{"name": "bar"}]"#;
        let info = BrewServiceRepository::parse_service_info_json(json).unwrap();
        assert_eq!(info.name, "bar");
        assert!(!info.running);
        assert!(!info.loaded);
        assert!(!info.registered);
        assert_eq!(info.status, ServiceStatus::Unknown);
    }

    #[test]
    fn parse_service_info_json_empty_errors() {
        let json = "[]";
        let result = BrewServiceRepository::parse_service_info_json(json);
        assert!(result.is_err());
    }

    #[test]
    fn parse_services_text_fallback() {
        let output = "Name       Status  User   File\npostgresql started whooof /usr/local/opt/postgresql/homebrew.postgresql.service.plist\nredis      stopped\nnginx      started root   /Library/LaunchDaemons/homebrew.mxcl.nginx.plist\n";
        let result = BrewServiceRepository::parse_services_text(output).unwrap();
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
    fn parse_services_text_empty() {
        let output = "Name  Status  User  File\n";
        let result = BrewServiceRepository::parse_services_text(output).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn parse_status_variants() {
        assert_eq!(
            BrewServiceRepository::parse_status("started"),
            ServiceStatus::Started
        );
        assert_eq!(
            BrewServiceRepository::parse_status("Started"),
            ServiceStatus::Started
        );
        assert_eq!(
            BrewServiceRepository::parse_status("stopped"),
            ServiceStatus::Stopped
        );
        assert_eq!(
            BrewServiceRepository::parse_status("none"),
            ServiceStatus::Stopped
        );
        assert_eq!(
            BrewServiceRepository::parse_status("error"),
            ServiceStatus::Error
        );
        assert_eq!(
            BrewServiceRepository::parse_status("something"),
            ServiceStatus::Unknown
        );
    }
}
