use crate::domain::entities::PackageType;
use anyhow::{Result, anyhow};
use std::process::{Command, Stdio};

pub struct BrewOutput {
    pub stdout: String,
    pub stderr: String,
}

pub struct BrewCommand;

impl BrewCommand {
    fn get_package_type_arg(package_type: PackageType) -> &'static str {
        match package_type {
            PackageType::Formula => "--formula",
            PackageType::Cask => "--cask",
        }
    }

    pub fn execute_brew(args: &[&str]) -> Result<String> {
        let output = Command::new("brew").args(args).output()?;

        if !output.status.success() {
            return Err(anyhow!(
                "Brew command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(String::from_utf8(output.stdout)?)
    }

    fn execute_brew_with_output(args: &[&str]) -> Result<BrewOutput> {
        // Run brew directly. When brew needs elevation, it will call sudo internally.
        // By setting SUDO_ASKPASS to a nonexistent script and setting SUDO_ASKPASS_REQUIRE=force,
        // we tell sudo to NOT prompt the terminal, but instead try to run that script.
        // When the script doesn't exist, sudo fails with an error we can detect.

        tracing::debug!("Executing brew command with SUDO_ASKPASS to prevent terminal prompts");

        let output = Command::new("brew")
            .args(args)
            .env("SUDO_ASKPASS", "/nonexistent/askpass") // Force sudo to not use terminal
            .env("SUDO_ASKPASS_REQUIRE", "force")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8(output.stderr)?;

        if !output.status.success() {
            // Check if this failed due to needing a password
            let combined = format!("{} {}", stdout, stderr).to_lowercase();

            if combined.contains("password")
                || combined.contains("sudo")
                || combined.contains("permission denied")
                || combined.contains("authentication")
                || combined.contains("privilege")
            {
                // This is a password/privilege error
                tracing::debug!("Password/privilege required - will show modal");
                return Err(anyhow!("a password is required"));
            }
            return Err(anyhow!("Brew command failed: {}", stderr));
        }

        Ok(BrewOutput { stdout, stderr })
    }

    fn execute_brew_with_password(args: &[&str], password: &str) -> Result<BrewOutput> {
        // Pass password via BREWSTY_SUDO_PASS env var and use an inline askpass
        // that reads it. This avoids writing the password to disk.

        tracing::debug!("Executing brew command with password via inline SUDO_ASKPASS");

        // Use /usr/bin/printenv to echo the env var — no shell escaping needed
        let output = Command::new("brew")
            .args(args)
            .env("SUDO_ASKPASS", "/usr/bin/printenv")
            .env("SUDO_ASKPASS_REQUIRE", "force")
            .env("SUDO_ASKPASS_VARS", "BREWSTY_SUDO_PASS")
            .env("BREWSTY_SUDO_PASS", password)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8(output.stderr)?;

        if !output.status.success() {
            // Check if it's a password-related error
            if stderr.contains("password is incorrect")
                || stderr.contains("sudo: 1 incorrect password attempt")
                || stderr.contains("sorry, try again")
                || stderr.contains("incorrect password")
            {
                return Err(anyhow!("Incorrect password"));
            }
            return Err(anyhow!("Brew command failed: {}", stderr));
        }

        Ok(BrewOutput { stdout, stderr })
    }

    pub fn list_packages(package_type: PackageType) -> Result<String> {
        let type_arg = match package_type {
            PackageType::Formula => "--formula",
            PackageType::Cask => "--cask",
        };
        tracing::debug!("Running: brew list {} --versions", type_arg);
        let result = Self::execute_brew(&["list", type_arg, "--versions"])?;
        tracing::debug!("brew list {} returned {} bytes", type_arg, result.len());
        Ok(result)
    }

    pub fn get_package_info(name: &str, package_type: PackageType) -> Result<String> {
        let type_arg = Self::get_package_type_arg(package_type);
        tracing::debug!("Running: brew info --json=v2 {} {}", type_arg, name);

        let output = Command::new("brew")
            .args(["info", "--json=v2", type_arg, name])
            .output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            tracing::error!("brew info {} {} failed: {}", type_arg, name, error_msg);
            return Err(anyhow!("Failed to get package info: {}", error_msg));
        }

        let result = String::from_utf8(output.stdout)?;
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
        Self::execute_brew(&["outdated", type_arg, "--json=v2"])
    }

    pub fn install_package(name: &str, package_type: PackageType) -> Result<BrewOutput> {
        let type_arg = Self::get_package_type_arg(package_type);
        Self::execute_brew_with_output(&["install", type_arg, name])
    }

    pub fn install_package_with_password(
        name: &str,
        package_type: PackageType,
        password: &str,
    ) -> Result<BrewOutput> {
        let type_arg = Self::get_package_type_arg(package_type);
        Self::execute_brew_with_password(&["install", type_arg, name], password)
    }

    pub fn uninstall_package(name: &str, package_type: PackageType) -> Result<BrewOutput> {
        let type_arg = Self::get_package_type_arg(package_type);
        Self::execute_brew_with_output(&["uninstall", type_arg, name])
    }

    pub fn uninstall_package_with_password(
        name: &str,
        package_type: PackageType,
        password: &str,
    ) -> Result<BrewOutput> {
        let type_arg = Self::get_package_type_arg(package_type);
        Self::execute_brew_with_password(&["uninstall", type_arg, name], password)
    }

    pub fn upgrade_package(name: &str, package_type: PackageType) -> Result<BrewOutput> {
        let type_arg = Self::get_package_type_arg(package_type);
        let output = Command::new("brew")
            .args(["upgrade", type_arg, name])
            .output()?;

        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8(output.stderr)?;

        if !output.status.success() {
            return Err(anyhow!("Failed to upgrade package: {}", stderr));
        }

        Ok(BrewOutput { stdout, stderr })
    }

    pub fn upgrade_all() -> Result<BrewOutput> {
        let output = Command::new("brew").args(["upgrade"]).output()?;

        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8(output.stderr)?;

        if !output.status.success() {
            return Err(anyhow!("Failed to upgrade all: {}", stderr));
        }

        Ok(BrewOutput { stdout, stderr })
    }

    pub fn cleanup_dry_run() -> Result<String> {
        Self::execute_brew(&["cleanup", "-s", "--dry-run"])
    }

    pub fn cleanup() -> Result<BrewOutput> {
        let output = Command::new("brew").args(["cleanup", "-s"]).output()?;

        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8(output.stderr)?;

        if !output.status.success() {
            return Err(anyhow!("Failed to cleanup: {}", stderr));
        }

        Ok(BrewOutput { stdout, stderr })
    }

    pub fn cleanup_old_versions_dry_run() -> Result<String> {
        Self::execute_brew(&["cleanup", "--prune=all", "--dry-run"])
    }

    pub fn cleanup_old_versions() -> Result<BrewOutput> {
        let output = Command::new("brew")
            .args(["cleanup", "--prune=all"])
            .output()?;

        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8(output.stderr)?;

        if !output.status.success() {
            return Err(anyhow!("Failed to cleanup old versions: {}", stderr));
        }

        Ok(BrewOutput { stdout, stderr })
    }

    pub fn search_packages(query: &str, package_type: PackageType) -> Result<String> {
        let type_arg = Self::get_package_type_arg(package_type);
        Self::execute_brew(&["search", type_arg, query])
    }

    pub fn list_pinned() -> Result<String> {
        Self::execute_brew(&["list", "--pinned"])
    }

    pub fn pin_package(name: &str) -> Result<BrewOutput> {
        let output = Command::new("brew").args(["pin", name]).output()?;

        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8(output.stderr)?;

        if !output.status.success() {
            return Err(anyhow!("Failed to pin package: {}", stderr));
        }

        Ok(BrewOutput { stdout, stderr })
    }

    pub fn unpin_package(name: &str) -> Result<BrewOutput> {
        let output = Command::new("brew").args(["unpin", name]).output()?;

        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8(output.stderr)?;

        if !output.status.success() {
            return Err(anyhow!("Failed to unpin package: {}", stderr));
        }

        Ok(BrewOutput { stdout, stderr })
    }

    // Services management
    pub fn list_services() -> Result<String> {
        Self::execute_brew(&["services", "list"])
    }

    pub fn start_service(name: &str) -> Result<BrewOutput> {
        let output = Command::new("brew")
            .args(["services", "start", name])
            .output()?;

        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8(output.stderr)?;

        if !output.status.success() {
            return Err(anyhow!("Failed to start service: {}", stderr));
        }

        Ok(BrewOutput { stdout, stderr })
    }

    pub fn stop_service(name: &str) -> Result<BrewOutput> {
        let output = Command::new("brew")
            .args(["services", "stop", name])
            .output()?;

        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8(output.stderr)?;

        if !output.status.success() {
            return Err(anyhow!("Failed to stop service: {}", stderr));
        }

        Ok(BrewOutput { stdout, stderr })
    }

    pub fn restart_service(name: &str) -> Result<BrewOutput> {
        let output = Command::new("brew")
            .args(["services", "restart", name])
            .output()?;

        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8(output.stderr)?;

        if !output.status.success() {
            return Err(anyhow!("Failed to restart service: {}", stderr));
        }

        Ok(BrewOutput { stdout, stderr })
    }

    // Export package list with versions
    pub fn export_installed() -> Result<String> {
        // Get list of formulae and casks with versions
        let formulae = Self::execute_brew(&["list", "--formula", "--versions"])?;
        let casks = Self::execute_brew(&["list", "--cask", "--versions"])?;

        Ok(format!("FORMULAE\n{}\nCASKS\n{}", formulae, casks))
    }

    // Health check
    pub fn doctor() -> Result<String> {
        // brew doctor exits non-zero when warnings exist, so capture output regardless
        let output = Command::new("brew").args(["doctor"]).output()?;
        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8(output.stderr)?;
        Ok(format!("{}{}", stdout, stderr))
    }

    // Dependencies
    pub fn deps(name: &str) -> Result<String> {
        Self::execute_brew(&["deps", "--tree", name])
    }

    pub fn uses(name: &str) -> Result<String> {
        Self::execute_brew(&["uses", "--installed", name])
    }

    // Taps management
    pub fn list_taps() -> Result<String> {
        Self::execute_brew(&["tap"])
    }

    pub fn tap(name: &str) -> Result<BrewOutput> {
        let output = Command::new("brew").args(["tap", name]).output()?;
        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8(output.stderr)?;
        if !output.status.success() {
            return Err(anyhow!("Failed to tap {}: {}", name, stderr));
        }
        Ok(BrewOutput { stdout, stderr })
    }

    pub fn untap(name: &str) -> Result<BrewOutput> {
        let output = Command::new("brew").args(["untap", name]).output()?;
        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8(output.stderr)?;
        if !output.status.success() {
            return Err(anyhow!("Failed to untap {}: {}", name, stderr));
        }
        Ok(BrewOutput { stdout, stderr })
    }
}
