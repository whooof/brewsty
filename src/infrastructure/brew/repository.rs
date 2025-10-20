use crate::domain::{
    entities::{CleanupItem, CleanupPreview, Package, PackageType},
    repositories::PackageRepository,
};
use crate::infrastructure::brew::command::BrewCommand;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::path::Path;

pub struct BrewPackageRepository;

impl BrewPackageRepository {
    pub fn new() -> Self {
        Self
    }

    fn get_pinned_packages(&self) -> Result<Vec<String>> {
        let output = BrewCommand::list_pinned()?;
        Ok(output
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| line.trim().to_string())
            .collect())
    }

    fn extract_package_item(
        item: &Value,
        package_type: PackageType,
        version_key: &str,
        is_pinned: bool,
    ) -> Option<Package> {
        let name = item.get("name").and_then(|v| v.as_str())?;

        let version_str = match version_key {
            "installed" => item
                .get("installed")
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.first())
                .and_then(|v| v.get("version"))
                .and_then(|v| v.as_str()),
            "installed_versions" => item
                .get("installed_versions")
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str()),
            _ => None,
        };

        let mut package = Package::new(name.to_string(), package_type)
            .set_installed(true)
            .with_version(version_str.unwrap_or_default().to_string())
            .set_pinned(is_pinned);

        if let Some(current_version) = item.get("current_version").and_then(|v| v.as_str()) {
            package = package
                .set_outdated(true)
                .with_available_version(current_version.to_string());
        }

        Some(package)
    }

    fn parse_packages_from_json(
        &self,
        json: &str,
        package_type: PackageType,
        version_key: &str,
    ) -> Result<Vec<Package>> {
        let data: Value = serde_json::from_str(json)?;
        let mut packages = Vec::new();

        let items_key = match package_type {
            PackageType::Formula => "formulae",
            PackageType::Cask => "casks",
        };

        let pinned_packages = self.get_pinned_packages().unwrap_or_default();

        if let Some(items) = data.get(items_key).and_then(|v| v.as_array()) {
            for item in items {
                if let Some(name) = item.get("name").and_then(|v| v.as_str()) {
                    let is_pinned = pinned_packages.contains(&name.to_string());

                    if let Some(package) = Self::extract_package_item(
                        item,
                        package_type.clone(),
                        version_key,
                        is_pinned,
                    ) {
                        packages.push(package);
                    }
                }
            }
        }

        Ok(packages)
    }

    fn parse_installed_packages_plain_text(
        &self,
        output: &str,
        package_type: PackageType,
        pinned_packages: &[String],
    ) -> Result<Vec<Package>> {
        tracing::debug!(
            "parse_installed_packages_plain_text called for {:?}",
            package_type
        );
        tracing::debug!("Output length: {} bytes", output.len());

        let mut packages = Vec::new();
        let line_count = output.lines().count();
        tracing::debug!("Processing {} lines", line_count);

        for line in output.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                let name = parts[0].to_string();
                let version = parts[1].to_string();
                let is_pinned = pinned_packages.contains(&name);

                let package = Package::new(name, package_type.clone())
                    .set_installed(true)
                    .with_version(version)
                    .set_pinned(is_pinned);

                packages.push(package);
            }
        }

        tracing::debug!("Parsed {} packages for {:?}", packages.len(), package_type);
        Ok(packages)
    }

    fn parse_installed_packages(
        &self,
        output: &str,
        package_type: PackageType,
    ) -> Result<Vec<Package>> {
        let pinned_packages = self.get_pinned_packages().unwrap_or_default();
        self.parse_installed_packages_plain_text(output, package_type, &pinned_packages)
    }

    fn parse_outdated_json(&self, json: &str, package_type: PackageType) -> Result<Vec<Package>> {
        self.parse_packages_from_json(json, package_type, "installed_versions")
    }

    fn parse_cleanup_output(&self, output: &str) -> Result<CleanupPreview> {
        let mut items = Vec::new();
        let mut total_size = 0u64;

        for line in output.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("==>") {
                continue;
            }

            if trimmed.starts_with("Would remove:") || trimmed.starts_with("Removing:") {
                continue;
            }

            if let Some(path_str) = trimmed.strip_prefix("Would remove: ") {
                let path = Path::new(path_str);
                let size = if path.exists() {
                    if path.is_file() {
                        std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
                    } else if path.is_dir() {
                        self.calculate_dir_size(path).unwrap_or(0)
                    } else {
                        0
                    }
                } else {
                    0
                };

                total_size += size;
                items.push(CleanupItem {
                    path: path_str.to_string(),
                    size,
                });
            }
        }

        Ok(CleanupPreview { items, total_size })
    }

    fn calculate_dir_size(&self, path: &Path) -> Result<u64> {
        let mut total = 0u64;
        if path.is_dir() {
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                let metadata = entry.metadata()?;
                if metadata.is_file() {
                    total += metadata.len();
                } else if metadata.is_dir() {
                    total += self.calculate_dir_size(&entry.path())?;
                }
            }
        }
        Ok(total)
    }

    async fn log_brew_output(output: &crate::infrastructure::brew::command::BrewOutput) {
        if !output.stdout.is_empty() {
            tracing::info!("brew output: {}", output.stdout);
        }
        if !output.stderr.is_empty() {
            tracing::info!("brew stderr: {}", output.stderr);
        }
    }
}

#[async_trait]
impl PackageRepository for BrewPackageRepository {
    async fn get_installed_packages(&self, package_type: PackageType) -> Result<Vec<Package>> {
        tracing::info!("get_installed_packages called for {:?}", package_type);
        let package_type_clone = package_type.clone();
        let output =
            tokio::task::spawn_blocking(move || BrewCommand::list_packages(package_type_clone))
                .await??;
        tracing::info!("Got output for {:?}: {} bytes", package_type, output.len());
        let result = self.parse_installed_packages(&output, package_type);
        tracing::info!(
            "parse_installed_packages returned: {:?}",
            result.as_ref().map(|p| p.len()).map_err(|e| e.to_string())
        );
        result
    }

    async fn get_outdated_packages(&self, package_type: PackageType) -> Result<Vec<Package>> {
        let package_type_clone = package_type.clone();
        let output =
            tokio::task::spawn_blocking(move || BrewCommand::outdated_packages(package_type_clone))
                .await??;
        self.parse_outdated_json(&output, package_type)
    }

    async fn install_package(&self, package: &Package) -> Result<()> {
        let name = package.name.clone();
        let package_type = package.package_type.clone();

        let output =
            tokio::task::spawn_blocking(move || BrewCommand::install_package(&name, package_type))
                .await??;

        Self::log_brew_output(&output).await;

        Ok(())
    }

    async fn uninstall_package(&self, package: &Package) -> Result<()> {
        let name = package.name.clone();
        let package_type = package.package_type.clone();

        let output = tokio::task::spawn_blocking(move || {
            BrewCommand::uninstall_package(&name, package_type)
        })
        .await??;

        Self::log_brew_output(&output).await;

        Ok(())
    }

    async fn update_package(&self, package: &Package) -> Result<()> {
        let name = package.name.clone();

        let output =
            tokio::task::spawn_blocking(move || BrewCommand::upgrade_package(&name)).await??;

        Self::log_brew_output(&output).await;

        Ok(())
    }

    async fn update_all(&self) -> Result<()> {
        let output = tokio::task::spawn_blocking(|| BrewCommand::upgrade_all()).await??;

        Self::log_brew_output(&output).await;

        Ok(())
    }

    async fn get_cleanup_preview(&self) -> Result<CleanupPreview> {
        let output = tokio::task::spawn_blocking(|| BrewCommand::cleanup_dry_run()).await??;
        self.parse_cleanup_output(&output)
    }

    async fn get_cleanup_old_versions_preview(&self) -> Result<CleanupPreview> {
        let output =
            tokio::task::spawn_blocking(|| BrewCommand::cleanup_old_versions_dry_run()).await??;
        self.parse_cleanup_output(&output)
    }

    async fn clean_cache(&self) -> Result<()> {
        let output = tokio::task::spawn_blocking(|| BrewCommand::cleanup()).await??;

        Self::log_brew_output(&output).await;

        Ok(())
    }

    async fn cleanup_old_versions(&self) -> Result<()> {
        let output = tokio::task::spawn_blocking(|| BrewCommand::cleanup_old_versions()).await??;

        Self::log_brew_output(&output).await;

        Ok(())
    }

    async fn search_packages(
        &self,
        query: &str,
        package_type: PackageType,
    ) -> Result<Vec<Package>> {
        let query = query.to_string();
        let package_type_clone = package_type.clone();
        let output = tokio::task::spawn_blocking(move || {
            BrewCommand::search_packages(&query, package_type_clone)
        })
        .await??;

        let packages: Vec<Package> = output
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| Package::new(line.trim().to_string(), package_type.clone()))
            .collect();

        Ok(packages)
    }

    async fn get_package_info(&self, name: &str, package_type: PackageType) -> Result<Package> {
        tracing::debug!("get_package_info called for {} ({:?})", name, package_type);

        let name = name.to_string();
        let name_clone = name.clone();
        let package_type_clone = package_type.clone();

        let output = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            tokio::task::spawn_blocking(move || {
                BrewCommand::get_package_info(&name_clone, package_type_clone)
            }),
        )
        .await
        .map_err(|_| anyhow::anyhow!("Timeout loading package info for {}", name))???;

        tracing::debug!("Raw brew output for {}: {} bytes", name, output.len());

        let data: Value = serde_json::from_str(&output).map_err(|e| {
            tracing::error!("Failed to parse JSON for {}: {}", name, e);
            e
        })?;

        tracing::debug!("Parsed JSON for {}: {:?}", name, data);

        let items_key = match package_type {
            PackageType::Formula => "formulae",
            PackageType::Cask => "casks",
        };

        if let Some(items) = data.get(items_key).and_then(|v| v.as_array()) {
            tracing::debug!(
                "Found {} items for {} in '{}'",
                items.len(),
                name,
                items_key
            );

            if let Some(item) = items.first() {
                let version = item
                    .get("version")
                    .or_else(|| item.get("versions").and_then(|v| v.get("stable")))
                    .and_then(|v| v.as_str())
                    .map(String::from);

                let description = item.get("desc").and_then(|v| v.as_str()).map(String::from);

                tracing::debug!(
                    "Extracted for {}: version={:?}, desc={:?}",
                    name,
                    version,
                    description
                );

                let mut package = Package::new(name.clone(), package_type);
                if let Some(v) = version {
                    package = package.with_version(v);
                }
                if let Some(d) = description {
                    package = package.with_description(d);
                }

                tracing::debug!("Successfully created package info for {}", name);
                return Ok(package);
            } else {
                tracing::error!("No items found in '{}' array for {}", items_key, name);
            }
        } else {
            tracing::error!("No '{}' key found in JSON for {}", items_key, name);
        }

        Err(anyhow::anyhow!("Package info not found for {}", name))
    }

    async fn pin_package(&self, package: &Package) -> Result<()> {
        let name = package.name.clone();
        let output = tokio::task::spawn_blocking(move || BrewCommand::pin_package(&name)).await??;

        Self::log_brew_output(&output).await;

        Ok(())
    }

    async fn unpin_package(&self, package: &Package) -> Result<()> {
        let name = package.name.clone();
        let output =
            tokio::task::spawn_blocking(move || BrewCommand::unpin_package(&name)).await??;

        Self::log_brew_output(&output).await;

        Ok(())
    }
}
