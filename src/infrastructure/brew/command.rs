use crate::domain::entities::PackageType;
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
    fn password_prompt(action: &str) -> String {
        format!("Brewsty needs your administrator password to {action}.")
    }

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

    fn execute_brew_with_output(args: &[&str], prompt: &str) -> Result<BrewOutput> {
        let askpass = TempAskpassHelper::create(prompt)
            .map_err(|error| anyhow!("Failed to create secure askpass helper: {}", error))?;

        tracing::debug!("Executing brew command with secure macOS askpass helper");

        let output = Command::new("brew")
            .args(args)
            .env("SUDO_ASKPASS", askpass.path())
            .env("SUDO_ASKPASS_REQUIRE", "force")
            .env("SUDO_PROMPT", prompt)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8(output.stderr)?;

        if !output.status.success() {
            let combined = format!("{} {}", stdout, stderr).to_lowercase();

            if combined.contains("password prompt cancelled")
                || combined.contains("no password was provided")
                || combined.contains("user canceled")
                || combined.contains("user cancelled")
            {
                return Err(anyhow!("Password prompt was cancelled"));
            }
            if combined.contains("incorrect password")
                || combined.contains("incorrect password attempt")
                || combined.contains("sorry, try again")
            {
                return Err(anyhow!("Incorrect password"));
            }
            return Err(anyhow!("Brew command failed: {}", stderr));
        }

        Ok(BrewOutput { stdout, stderr })
    }

    pub fn get_installed_sizes() -> Result<std::collections::HashMap<String, u64>> {
        tracing::debug!("Running: brew --prefix to get Cellar and Caskroom paths");
        let prefix = Self::execute_brew(&["--prefix"])?.trim().to_string();

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
            if parts.len() == 2 {
                if let Ok(size_kb) = parts[0].parse::<u64>() {
                    let path = parts[1];
                    if let Some(name) = path.split('/').last() {
                        sizes.insert(name.to_string(), size_kb * 1024);
                    }
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
        let prompt = Self::password_prompt(&format!("install {name}"));
        Self::execute_brew_with_output(&["install", type_arg, name], &prompt)
    }

    pub fn uninstall_package(name: &str, package_type: PackageType) -> Result<BrewOutput> {
        let type_arg = Self::get_package_type_arg(package_type);
        let prompt = Self::password_prompt(&format!("uninstall {name}"));
        Self::execute_brew_with_output(&["uninstall", type_arg, name], &prompt)
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

    pub fn autoremove_dry_run() -> Result<String> {
        Self::execute_brew(&["autoremove", "-n"])
    }

    pub fn autoremove() -> Result<BrewOutput> {
        let output = Command::new("brew").args(["autoremove"]).output()?;

        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8(output.stderr)?;

        if !output.status.success() {
            return Err(anyhow!("Failed to autoremove: {}", stderr));
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

    pub fn list_services_json() -> Result<String> {
        Self::execute_brew(&["services", "list", "--json"])
    }

    pub fn service_info_json(name: &str) -> Result<String> {
        Self::execute_brew(&["services", "info", name, "--json"])
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
        Self::execute_brew_with_output(&["services", "start", name], &prompt)
    }

    pub fn stop_service(name: &str) -> Result<BrewOutput> {
        let prompt = Self::password_prompt(&format!("stop service {name}"));
        Self::execute_brew_with_output(&["services", "stop", name], &prompt)
    }

    pub fn restart_service(name: &str) -> Result<BrewOutput> {
        let prompt = Self::password_prompt(&format!("restart service {name}"));
        Self::execute_brew_with_output(&["services", "restart", name], &prompt)
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

    pub fn bundle_dump(path: &str) -> Result<String> {
        let output = Command::new("brew")
            .args(["bundle", "dump", "--force", "--file", path])
            .output()?;
        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8(output.stderr)?;
        if !output.status.success() {
            anyhow::bail!("Brew bundle dump failed: {}", stderr);
        }
        Ok(stdout)
    }

    pub fn bundle_check(path: &str) -> Result<String> {
        let output = Command::new("brew")
            .args(["bundle", "check", "--file", path, "--verbose"])
            .output()?;
        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8(output.stderr)?;
        // 'check' fails if there are missing dependencies
        // So we just return the output whether it fails or not.
        Ok(format!("{}\n{}", stdout, stderr))
    }

    pub fn bundle_cleanup_dry_run(path: &str) -> Result<String> {
        let output = Command::new("brew")
            .args(["bundle", "cleanup", "--file", path])
            .output()?;
        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8(output.stderr)?;
        Ok(format!("{}\n{}", stdout, stderr))
    }

    pub fn bundle_install(path: &str) -> Result<String> {
        let output = Command::new("brew")
            .args(["bundle", "install", "--file", path])
            .output()?;
        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8(output.stderr)?;
        if !output.status.success() {
            anyhow::bail!("Brew bundle install failed: {}", stderr);
        }
        Ok(stdout)
    }

    pub fn bundle_cleanup_force(path: &str) -> Result<String> {
        let output = Command::new("brew")
            .args(["bundle", "cleanup", "--force", "--file", path])
            .output()?;
        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8(output.stderr)?;
        if !output.status.success() {
            anyhow::bail!("Brew bundle cleanup failed: {}", stderr);
        }
        Ok(stdout)
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

#[cfg(test)]
mod tests {
    use super::TempAskpassHelper;

    #[test]
    fn escapes_applescript_prompt_text() {
        let script = TempAskpassHelper::script_contents(r#"Install "foo\bar" requires approval"#);

        assert!(script.contains(r#"set promptText to "Install \"foo\\bar\" requires approval""#));
    }
}
