use crate::domain::entities::PackageType;
use crate::domain::entities::{AppError, CommandResult};
use anyhow::{Result, anyhow};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct BrewOutput {
    pub stdout: String,
    pub stderr: String,
}

pub struct BrewCommand;

struct TempAskpassHelper {
    dir_path: PathBuf,
    script_path: PathBuf,
}

impl TempAskpassHelper {
    fn create(prompt: &str) -> Result<Self> {
        let unique_id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let dir_path = std::env::temp_dir().join(format!(
            "brewsty-askpass-{}-{unique_id}",
            std::process::id()
        ));
        fs::create_dir(&dir_path)?;
        fs::set_permissions(&dir_path, fs::Permissions::from_mode(0o700))?;

        let script_path = dir_path.join("askpass.sh");
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .mode(0o700)
            .open(&script_path)?;
        file.write_all(Self::script_contents(prompt).as_bytes())?;
        file.sync_all()?;

        Ok(Self {
            dir_path,
            script_path,
        })
    }

    fn path(&self) -> &std::path::Path {
        &self.script_path
    }

    fn escape_applescript_string(value: &str) -> String {
        value.replace('\\', "\\\\").replace('"', "\\\"")
    }

    fn script_contents(prompt: &str) -> String {
        let escaped_prompt = Self::escape_applescript_string(prompt);

        format!(
            r#"#!/bin/sh
set -eu
PATH=/usr/bin:/bin:/usr/sbin:/sbin

exec /usr/bin/osascript <<'APPLESCRIPT'
try
    set promptText to "{escaped_prompt}"
    set passwordText to text returned of (display dialog promptText with title "Brewsty" default answer "" with hidden answer buttons {{"Cancel", "OK"}} default button "OK")
    return passwordText
on error number -128
    error "Password prompt cancelled" number 1
end try
APPLESCRIPT
"#
        )
    }
}

impl Drop for TempAskpassHelper {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.script_path);
        let _ = fs::remove_dir(&self.dir_path);
    }
}

impl BrewCommand {
    fn brew_program_path() -> String {
        std::process::Command::new("which")
            .arg("brew")
            .output()
            .ok()
            .and_then(|output| {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if path.is_empty() { None } else { Some(path) }
            })
            .unwrap_or_else(|| "brew".to_string())
    }

    fn password_prompt(action: &str) -> String {
        format!("Brewsty needs your administrator password to {action}.")
    }

    fn get_package_type_arg(package_type: PackageType) -> &'static str {
        match package_type {
            PackageType::Formula => "--formula",
            PackageType::Cask => "--cask",
        }
    }

    fn command_string(args: &[&str]) -> String {
        if args.is_empty() {
            "brew".to_string()
        } else {
            format!("brew {}", args.join(" "))
        }
    }

    fn from_spawn_error(command: String, error: std::io::Error) -> AppError {
        if error.kind() == std::io::ErrorKind::NotFound {
            AppError::BrewMissing
        } else {
            AppError::Io(format!("{command}: {error}"))
        }
    }

    fn classify_failure(result: CommandResult) -> AppError {
        let combined = format!("{} {}", result.stdout, result.stderr).to_lowercase();

        if combined.contains("password prompt cancelled")
            || combined.contains("no password was provided")
            || combined.contains("user canceled")
            || combined.contains("user cancelled")
        {
            return AppError::AuthCancelled;
        }

        if combined.contains("incorrect password")
            || combined.contains("incorrect password attempt")
            || combined.contains("sorry, try again")
        {
            return AppError::AuthFailed;
        }

        AppError::CommandFailed {
            command: result.command,
            exit_code: result.exit_code,
            stderr: result.stderr.trim().to_string(),
        }
    }

    fn execute_command(
        args: &[&str],
        configure: impl FnOnce(Command) -> Command,
    ) -> std::result::Result<CommandResult, AppError> {
        let command_string = Self::command_string(args);
        let command = Command::new("brew");
        let output = configure(command)
            .args(args)
            .output()
            .map_err(|error| Self::from_spawn_error(command_string.clone(), error))?;

        Ok(CommandResult {
            command: command_string,
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            exit_code: output.status.code(),
        })
    }

    fn run_brew(args: &[&str]) -> std::result::Result<String, AppError> {
        let result = Self::execute_command(args, |command| command)?;
        if !result.succeeded() {
            return Err(Self::classify_failure(result));
        }
        Ok(result.stdout)
    }

    pub fn execute_brew(args: &[&str]) -> Result<String> {
        Ok(Self::run_brew(args)?)
    }

    fn run_brew_output(args: &[&str]) -> std::result::Result<BrewOutput, AppError> {
        let result = Self::execute_command(args, |command| command)?;
        if !result.succeeded() {
            return Err(Self::classify_failure(result));
        }
        Ok(BrewOutput {
            stdout: result.stdout,
            stderr: result.stderr,
        })
    }

    fn run_brew_with_output(
        args: &[&str],
        prompt: &str,
    ) -> std::result::Result<BrewOutput, AppError> {
        let askpass = TempAskpassHelper::create(prompt).map_err(|error| {
            AppError::Io(format!("Failed to create secure askpass helper: {}", error))
        })?;

        tracing::debug!("Executing brew command with secure macOS askpass helper");

        let result = Self::execute_command(args, |mut command| {
            command
                .env("SUDO_ASKPASS", askpass.path())
                .env("SUDO_ASKPASS_REQUIRE", "force")
                .env("SUDO_PROMPT", prompt)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            command
        })?;

        if !result.succeeded() {
            return Err(Self::classify_failure(result));
        }

        Ok(BrewOutput {
            stdout: result.stdout,
            stderr: result.stderr,
        })
    }

    fn run_sudo_brew_with_output(
        args: &[&str],
        prompt: &str,
    ) -> std::result::Result<BrewOutput, AppError> {
        let askpass = TempAskpassHelper::create(prompt).map_err(|error| {
            AppError::Io(format!("Failed to create secure askpass helper: {}", error))
        })?;

        let brew_path = Self::brew_program_path();
        let command_string = format!("sudo -A {} {}", brew_path, args.join(" "));
        let output = Command::new("sudo")
            .arg("-A")
            .arg(&brew_path)
            .args(args)
            .env("SUDO_ASKPASS", askpass.path())
            .env("SUDO_PROMPT", prompt)
            .env("PATH", std::env::var("PATH").unwrap_or_default())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|error| Self::from_spawn_error(command_string.clone(), error))?;

        let result = CommandResult {
            command: command_string,
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            exit_code: output.status.code(),
        };

        if !result.succeeded() {
            return Err(Self::classify_failure(result));
        }

        Ok(BrewOutput {
            stdout: result.stdout,
            stderr: result.stderr,
        })
    }

    fn should_retry_service_with_sudo(error: &AppError) -> bool {
        match error {
            AppError::CommandFailed { stderr, .. } => {
                let lowered = stderr.to_lowercase();
                let user_login_non_root_conflict = lowered
                    .contains("must be run as non-root to start at user login")
                    || lowered.contains("bootstrap system")
                    || lowered.contains("/library/launchdaemons/homebrew.mxcl.");
                let retryable_permission_issue = lowered
                    .contains("try re-running the command as root")
                    || lowered.contains("bootstrap failed: 5")
                    || lowered.contains("permission denied")
                    || lowered.contains("operation not permitted");

                retryable_permission_issue && !user_login_non_root_conflict
            }
            _ => false,
        }
    }

    pub fn get_installed_sizes() -> Result<std::collections::HashMap<String, u64>> {
        tracing::debug!("Running: brew --prefix to get Cellar and Caskroom paths");
        let prefix = Self::run_brew(&["--prefix"])?.trim().to_string();

        let cellar = format!("{}/Cellar/*", prefix);
        let caskroom = format!("{}/Caskroom/*", prefix);

        let output = Command::new("sh")
            .arg("-c")
            .arg(format!("du -sk {} {} 2>/dev/null", cellar, caskroom))
            .output()?;

        let mut sizes = std::collections::HashMap::new();
        let stdout = String::from_utf8_lossy(&output.stdout);

        for line in stdout.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() == 2
                && let Ok(size_kb) = parts[0].parse::<u64>() {
                    let path = parts[1];
                    if let Some(name) = path.split('/').next_back() {
                        sizes.insert(name.to_string(), size_kb * 1024);
                    }
                }
        }

        Ok(sizes)
    }

    pub fn list_packages(package_type: PackageType) -> Result<String> {
        let type_arg = match package_type {
            PackageType::Formula => "--formula",
            PackageType::Cask => "--cask",
        };
        tracing::debug!("Running: brew list {} --versions", type_arg);
        let result = Self::run_brew(&["list", type_arg, "--versions"])?;
        tracing::debug!("brew list {} returned {} bytes", type_arg, result.len());
        Ok(result)
    }

    pub fn get_package_info(name: &str, package_type: PackageType) -> Result<String> {
        let type_arg = Self::get_package_type_arg(package_type);
        tracing::debug!("Running: brew info --json=v2 {} {}", type_arg, name);

        let result = Self::run_brew(&["info", "--json=v2", type_arg, name])?;
        tracing::debug!(
            "brew info {} {} returned {} bytes",
            type_arg,
            name,
            result.len()
        );
        Ok(result)
    }

    pub fn outdated_packages(package_type: PackageType) -> Result<String> {
        let type_arg = Self::get_package_type_arg(package_type);
        Ok(Self::run_brew(&["outdated", type_arg, "--json=v2"])?)
    }

    pub fn install_package(name: &str, package_type: PackageType) -> Result<BrewOutput> {
        let type_arg = Self::get_package_type_arg(package_type);
        let prompt = Self::password_prompt(&format!("install {name}"));
        Ok(Self::run_brew_with_output(
            &["install", type_arg, name],
            &prompt,
        )?)
    }

    pub fn uninstall_package(name: &str, package_type: PackageType) -> Result<BrewOutput> {
        let type_arg = Self::get_package_type_arg(package_type);
        let prompt = Self::password_prompt(&format!("uninstall {name}"));
        Ok(Self::run_brew_with_output(
            &["uninstall", type_arg, name],
            &prompt,
        )?)
    }

    pub fn upgrade_package(name: &str, package_type: PackageType) -> Result<BrewOutput> {
        let type_arg = Self::get_package_type_arg(package_type);
        Ok(Self::run_brew_output(&["upgrade", type_arg, name])?)
    }

    pub fn upgrade_all() -> Result<BrewOutput> {
        Ok(Self::run_brew_output(&["upgrade"])?)
    }

    pub fn cleanup_dry_run() -> Result<String> {
        Ok(Self::run_brew(&["cleanup", "-s", "--dry-run"])?)
    }

    pub fn cleanup() -> Result<BrewOutput> {
        Ok(Self::run_brew_output(&["cleanup", "-s"])?)
    }

    pub fn cleanup_old_versions_dry_run() -> Result<String> {
        Ok(Self::run_brew(&["cleanup", "--prune=all", "--dry-run"])?)
    }

    pub fn cleanup_old_versions() -> Result<BrewOutput> {
        Ok(Self::run_brew_output(&["cleanup", "--prune=all"])?)
    }

    pub fn autoremove_dry_run() -> Result<String> {
        Ok(Self::run_brew(&["autoremove", "-n"])?)
    }

    pub fn autoremove() -> Result<BrewOutput> {
        Ok(Self::run_brew_output(&["autoremove"])?)
    }

    pub fn search_packages(query: &str, package_type: PackageType) -> Result<String> {
        let type_arg = Self::get_package_type_arg(package_type);
        Ok(Self::run_brew(&["search", type_arg, query])?)
    }

    pub fn list_pinned() -> Result<String> {
        Ok(Self::run_brew(&["list", "--pinned"])?)
    }

    pub fn pin_package(name: &str) -> Result<BrewOutput> {
        Ok(Self::run_brew_output(&["pin", name])?)
    }

    pub fn unpin_package(name: &str) -> Result<BrewOutput> {
        Ok(Self::run_brew_output(&["unpin", name])?)
    }

    // Services management
    pub fn list_services() -> Result<String> {
        Ok(Self::run_brew(&["services", "list"])?)
    }

    pub fn list_services_json() -> Result<String> {
        Ok(Self::run_brew(&["services", "list", "--json"])?)
    }

    pub fn service_info_json(name: &str) -> Result<String> {
        Ok(Self::run_brew(&["services", "info", name, "--json"])?)
    }

    /// Read the last `tail_lines` lines from a log file at `path`.
    pub fn read_service_log(path: &str, tail_lines: usize) -> Result<String> {
        use std::io::{BufRead, BufReader};

        let file = std::fs::File::open(path)
            .map_err(|e| anyhow!("Cannot open log file '{}': {}", path, e))?;

        let reader = BufReader::new(file);
        let all_lines: Vec<String> = reader.lines().collect::<std::io::Result<Vec<_>>>()?;

        let start = all_lines.len().saturating_sub(tail_lines);
        Ok(all_lines[start..].join("\n"))
    }

    pub fn start_service(name: &str) -> Result<BrewOutput> {
        let prompt = Self::password_prompt(&format!("start service {name}"));
        match Self::run_brew_with_output(&["services", "start", name], &prompt) {
            Ok(output) => Ok(output),
            Err(error) if Self::should_retry_service_with_sudo(&error) => Ok(
                Self::run_sudo_brew_with_output(&["services", "start", name], &prompt)?,
            ),
            Err(error) => Err(error.into()),
        }
    }

    pub fn stop_service(name: &str) -> Result<BrewOutput> {
        let prompt = Self::password_prompt(&format!("stop service {name}"));
        Ok(Self::run_brew_with_output(
            &["services", "stop", name],
            &prompt,
        )?)
    }

    pub fn restart_service(name: &str) -> Result<BrewOutput> {
        let prompt = Self::password_prompt(&format!("restart service {name}"));
        match Self::run_brew_with_output(&["services", "restart", name], &prompt) {
            Ok(output) => Ok(output),
            Err(error) if Self::should_retry_service_with_sudo(&error) => Ok(
                Self::run_sudo_brew_with_output(&["services", "restart", name], &prompt)?,
            ),
            Err(error) => Err(error.into()),
        }
    }

    // Export package list with versions
    pub fn export_installed() -> Result<String> {
        // Get list of formulae and casks with versions
        let formulae = Self::run_brew(&["list", "--formula", "--versions"])?;
        let casks = Self::run_brew(&["list", "--cask", "--versions"])?;

        Ok(format!("FORMULAE\n{}\nCASKS\n{}", formulae, casks))
    }

    // Health check
    pub fn doctor() -> Result<String> {
        let result = Self::execute_command(&["doctor"], |command| command)?;
        Ok(format!("{}{}", result.stdout, result.stderr))
    }

    pub fn bundle_dump(path: &str) -> Result<String> {
        Ok(Self::run_brew(&[
            "bundle", "dump", "--force", "--file", path,
        ])?)
    }

    pub fn bundle_check(path: &str) -> Result<String> {
        let output = Self::execute_command(
            &["bundle", "check", "--file", path, "--verbose"],
            |command| command,
        )?;
        // 'check' fails if there are missing dependencies
        // So we just return the output whether it fails or not.
        Ok(format!("{}\n{}", output.stdout, output.stderr))
    }

    pub fn bundle_cleanup_dry_run(path: &str) -> Result<String> {
        let output =
            Self::execute_command(&["bundle", "cleanup", "--file", path], |command| command)?;
        Ok(format!("{}\n{}", output.stdout, output.stderr))
    }

    pub fn bundle_install(path: &str) -> Result<String> {
        Ok(Self::run_brew(&["bundle", "install", "--file", path])?)
    }

    pub fn bundle_cleanup_force(path: &str) -> Result<String> {
        Ok(Self::run_brew(&[
            "bundle", "cleanup", "--force", "--file", path,
        ])?)
    }

    // Dependencies
    pub fn deps(name: &str) -> Result<String> {
        Ok(Self::run_brew(&["deps", "--tree", name])?)
    }

    pub fn uses(name: &str) -> Result<String> {
        Ok(Self::run_brew(&["uses", "--installed", name])?)
    }

    // Taps management
    pub fn list_taps() -> Result<String> {
        Ok(Self::run_brew(&["tap"])?)
    }

    pub fn tap(name: &str) -> Result<BrewOutput> {
        Ok(Self::run_brew_output(&["tap", name])?)
    }

    pub fn untap(name: &str) -> Result<BrewOutput> {
        Ok(Self::run_brew_output(&["untap", name])?)
    }
}

#[cfg(test)]
mod tests {
    use super::{BrewCommand, TempAskpassHelper};
    use crate::domain::entities::AppError;
    use std::io;

    #[test]
    fn escapes_applescript_prompt_text() {
        let script = TempAskpassHelper::script_contents(r#"Install "foo\bar" requires approval"#);

        assert!(script.contains(r#"set promptText to "Install \"foo\\bar\" requires approval""#));
    }

    #[test]
    fn classify_spawn_not_found_as_brew_missing() {
        let error = io::Error::new(io::ErrorKind::NotFound, "brew not found");
        let classified = BrewCommand::from_spawn_error("brew list".to_string(), error);

        assert_eq!(classified, AppError::BrewMissing);
    }

    #[test]
    fn classify_cancelled_auth_error() {
        let result = crate::domain::entities::CommandResult {
            command: "brew install wget".to_string(),
            stdout: String::new(),
            stderr: "Password prompt cancelled".to_string(),
            exit_code: Some(1),
        };

        assert_eq!(
            BrewCommand::classify_failure(result),
            AppError::AuthCancelled
        );
    }

    #[test]
    fn classify_incorrect_password_error() {
        let result = crate::domain::entities::CommandResult {
            command: "brew install wget".to_string(),
            stdout: String::new(),
            stderr: "Sorry, try again.".to_string(),
            exit_code: Some(1),
        };

        assert_eq!(BrewCommand::classify_failure(result), AppError::AuthFailed);
    }

    #[test]
    fn classify_command_failure_with_exit_code() {
        let result = crate::domain::entities::CommandResult {
            command: "brew upgrade wget".to_string(),
            stdout: String::new(),
            stderr: "boom".to_string(),
            exit_code: Some(2),
        };

        assert_eq!(
            BrewCommand::classify_failure(result),
            AppError::CommandFailed {
                command: "brew upgrade wget".to_string(),
                exit_code: Some(2),
                stderr: "boom".to_string(),
            }
        );
    }

    #[test]
    fn command_result_success_matches_zero_exit_code() {
        let result = crate::domain::entities::CommandResult {
            command: "brew list".to_string(),
            stdout: String::new(),
            stderr: String::new(),
            exit_code: Some(0),
        };

        assert!(result.succeeded());
    }

    #[test]
    fn service_root_retry_detects_bootstrap_hint() {
        let error = AppError::CommandFailed {
            command: "brew services start redis".to_string(),
            exit_code: Some(1),
            stderr: "Bootstrap failed: 5: Input/output error\nTry re-running the command as root for richer errors.".to_string(),
        };

        assert!(BrewCommand::should_retry_service_with_sudo(&error));
    }

    #[test]
    fn service_root_retry_skips_user_login_non_root_conflict() {
        let error = AppError::CommandFailed {
            command: "brew services start caddy".to_string(),
            exit_code: Some(1),
            stderr: "Warning: caddy must be run as non-root to start at user login!\nBootstrap failed: 5: Input/output error\nError: Failure while executing; `/bin/launchctl bootstrap system /Library/LaunchDaemons/homebrew.mxcl.caddy.plist` exited with 5.".to_string(),
        };

        assert!(!BrewCommand::should_retry_service_with_sudo(&error));
    }
}
