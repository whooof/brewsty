use crate::domain::entities::PackageType;
use anyhow::{Result, anyhow};
use std::io::Write;
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

    fn execute_brew(args: &[&str]) -> Result<String> {
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
        let output = Command::new("brew").args(args).output()?;

        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8(output.stderr)?;

        if !output.status.success() {
            return Err(anyhow!("Brew command failed: {}", stderr));
        }

        Ok(BrewOutput { stdout, stderr })
    }

    fn execute_brew_with_password(args: &[&str], password: &str) -> Result<BrewOutput> {
        let mut child = Command::new("brew")
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        {
            if let Some(mut stdin) = child.stdin.take() {
                // Write password followed by newline
                write!(stdin, "{}\n", password)?;
            }
        }

        let output = child.wait_with_output()?;
        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8(output.stderr)?;

        if !output.status.success() {
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
            .args(&["info", "--json=v2", type_arg, name])
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

    pub fn upgrade_package(name: &str) -> Result<BrewOutput> {
        let output = Command::new("brew").args(["upgrade", name]).output()?;

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
}
