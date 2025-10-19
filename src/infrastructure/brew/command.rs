use anyhow::{anyhow, Result};
use std::process::Command;

pub struct BrewOutput {
    pub stdout: String,
    pub stderr: String,
}

pub struct BrewCommand;

impl BrewCommand {
    pub fn list_formulae() -> Result<String> {
        let output = Command::new("brew")
            .args(["info", "--json=v2", "--installed", "--formula"])
            .output()?;

        if !output.status.success() {
            return Err(anyhow!("Failed to list formulae: {}", String::from_utf8_lossy(&output.stderr)));
        }

        Ok(String::from_utf8(output.stdout)?)
    }

    pub fn list_casks() -> Result<String> {
        let output = Command::new("brew")
            .args(["info", "--json=v2", "--installed", "--cask"])
            .output()?;

        if !output.status.success() {
            return Err(anyhow!("Failed to list casks: {}", String::from_utf8_lossy(&output.stderr)));
        }

        Ok(String::from_utf8(output.stdout)?)
    }

    pub fn get_formula_info(name: &str) -> Result<String> {
        tracing::debug!("Running: brew info --json=v2 --formula {}", name);
        
        let output = Command::new("brew")
            .args(["info", "--json=v2", "--formula", name])
            .output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            tracing::error!("brew info --formula {} failed: {}", name, error_msg);
            return Err(anyhow!("Failed to get formula info: {}", error_msg));
        }

        let result = String::from_utf8(output.stdout)?;
        tracing::debug!("brew info --formula {} returned {} bytes", name, result.len());
        Ok(result)
    }

    pub fn get_cask_info(name: &str) -> Result<String> {
        tracing::debug!("Running: brew info --json=v2 --cask {}", name);
        
        let output = Command::new("brew")
            .args(["info", "--json=v2", "--cask", name])
            .output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            tracing::error!("brew info --cask {} failed: {}", name, error_msg);
            return Err(anyhow!("Failed to get cask info: {}", error_msg));
        }

        let result = String::from_utf8(output.stdout)?;
        tracing::debug!("brew info --cask {} returned {} bytes", name, result.len());
        Ok(result)
    }

    pub fn outdated_formulae() -> Result<String> {
        let output = Command::new("brew")
            .args(["outdated", "--formula", "--json=v2"])
            .output()?;

        if !output.status.success() {
            return Err(anyhow!("Failed to get outdated formulae: {}", String::from_utf8_lossy(&output.stderr)));
        }

        Ok(String::from_utf8(output.stdout)?)
    }

    pub fn outdated_casks() -> Result<String> {
        let output = Command::new("brew")
            .args(["outdated", "--cask", "--json=v2"])
            .output()?;

        if !output.status.success() {
            return Err(anyhow!("Failed to get outdated casks: {}", String::from_utf8_lossy(&output.stderr)));
        }

        Ok(String::from_utf8(output.stdout)?)
    }

    pub fn install_formula(name: &str) -> Result<BrewOutput> {
        let output = Command::new("brew")
            .args(["install", "--formula", name])
            .output()?;

        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8(output.stderr)?;

        if !output.status.success() {
            return Err(anyhow!("Failed to install formula: {}", stderr));
        }

        Ok(BrewOutput { stdout, stderr })
    }

    pub fn install_cask(name: &str) -> Result<BrewOutput> {
        let output = Command::new("brew")
            .args(["install", "--cask", name])
            .output()?;

        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8(output.stderr)?;

        if !output.status.success() {
            return Err(anyhow!("Failed to install cask: {}", stderr));
        }

        Ok(BrewOutput { stdout, stderr })
    }

    pub fn uninstall_formula(name: &str) -> Result<BrewOutput> {
        let output = Command::new("brew")
            .args(["uninstall", "--formula", name])
            .output()?;

        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8(output.stderr)?;

        if !output.status.success() {
            return Err(anyhow!("Failed to uninstall formula: {}", stderr));
        }

        Ok(BrewOutput { stdout, stderr })
    }

    pub fn uninstall_cask(name: &str) -> Result<BrewOutput> {
        let output = Command::new("brew")
            .args(["uninstall", "--cask", name])
            .output()?;

        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8(output.stderr)?;

        if !output.status.success() {
            return Err(anyhow!("Failed to uninstall cask: {}", stderr));
        }

        Ok(BrewOutput { stdout, stderr })
    }

    pub fn upgrade_package(name: &str) -> Result<BrewOutput> {
        let output = Command::new("brew")
            .args(["upgrade", name])
            .output()?;

        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8(output.stderr)?;

        if !output.status.success() {
            return Err(anyhow!("Failed to upgrade package: {}", stderr));
        }

        Ok(BrewOutput { stdout, stderr })
    }

    pub fn upgrade_all() -> Result<BrewOutput> {
        let output = Command::new("brew")
            .args(["upgrade"])
            .output()?;

        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8(output.stderr)?;

        if !output.status.success() {
            return Err(anyhow!("Failed to upgrade all: {}", stderr));
        }

        Ok(BrewOutput { stdout, stderr })
    }

    pub fn cleanup_dry_run() -> Result<String> {
        let output = Command::new("brew")
            .args(["cleanup", "-s", "--dry-run"])
            .output()?;

        if !output.status.success() {
            return Err(anyhow!("Failed to get cleanup info: {}", String::from_utf8_lossy(&output.stderr)));
        }

        Ok(String::from_utf8(output.stdout)?)
    }

    pub fn cleanup() -> Result<BrewOutput> {
        let output = Command::new("brew")
            .args(["cleanup", "-s"])
            .output()?;

        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8(output.stderr)?;

        if !output.status.success() {
            return Err(anyhow!("Failed to cleanup: {}", stderr));
        }

        Ok(BrewOutput { stdout, stderr })
    }

    pub fn cleanup_old_versions_dry_run() -> Result<String> {
        let output = Command::new("brew")
            .args(["cleanup", "--prune=all", "--dry-run"])
            .output()?;

        if !output.status.success() {
            return Err(anyhow!("Failed to get cleanup info: {}", String::from_utf8_lossy(&output.stderr)));
        }

        Ok(String::from_utf8(output.stdout)?)
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

    pub fn search_formula(query: &str) -> Result<String> {
        let output = Command::new("brew")
            .args(["search", "--formula", query])
            .output()?;

        if !output.status.success() {
            return Err(anyhow!("Failed to search formulae: {}", String::from_utf8_lossy(&output.stderr)));
        }

        Ok(String::from_utf8(output.stdout)?)
    }

    pub fn search_cask(query: &str) -> Result<String> {
        let output = Command::new("brew")
            .args(["search", "--cask", query])
            .output()?;

        if !output.status.success() {
            return Err(anyhow!("Failed to search casks: {}", String::from_utf8_lossy(&output.stderr)));
        }

        Ok(String::from_utf8(output.stdout)?)
    }

    pub fn list_pinned() -> Result<String> {
        let output = Command::new("brew")
            .args(["list", "--pinned"])
            .output()?;

        if !output.status.success() {
            return Err(anyhow!("Failed to list pinned packages: {}", String::from_utf8_lossy(&output.stderr)));
        }

        Ok(String::from_utf8(output.stdout)?)
    }

    pub fn pin_package(name: &str) -> Result<BrewOutput> {
        let output = Command::new("brew")
            .args(["pin", name])
            .output()?;

        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8(output.stderr)?;

        if !output.status.success() {
            return Err(anyhow!("Failed to pin package: {}", stderr));
        }

        Ok(BrewOutput { stdout, stderr })
    }

     pub fn unpin_package(name: &str) -> Result<BrewOutput> {
         let output = Command::new("brew")
             .args(["unpin", name])
             .output()?;
 
         let stdout = String::from_utf8(output.stdout)?;
         let stderr = String::from_utf8(output.stderr)?;
 
         if !output.status.success() {
             return Err(anyhow!("Failed to unpin package: {}", stderr));
         }
 
         Ok(BrewOutput { stdout, stderr })
     }
}
