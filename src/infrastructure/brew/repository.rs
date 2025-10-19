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

    fn parse_installed_packages(&self, json: &str, package_type: PackageType) -> Result<Vec<Package>> {
        let data: Value = serde_json::from_str(json)?;
        let mut packages = Vec::new();

        let items_key = match package_type {
            PackageType::Formula => "formulae",
            PackageType::Cask => "casks",
        };

        // Get the list of pinned packages
        let pinned_packages = self.get_pinned_packages().unwrap_or_default();

        if let Some(items) = data.get(items_key).and_then(|v| v.as_array()) {
            for item in items {
                if let Some(name) = item.get("name").and_then(|v| v.as_str()) {
                    let version = item
                        .get("installed")
                        .and_then(|v| v.as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|v| v.get("version"))
                        .and_then(|v| v.as_str())
                        .map(String::from);

                    let is_pinned = pinned_packages.contains(&name.to_string());

                    packages.push(
                        Package::new(name.to_string(), package_type.clone())
                            .set_installed(true)
                            .with_version(version.unwrap_or_default())
                            .set_pinned(is_pinned),
                    );
                }
            }
        }

        Ok(packages)
    }

    fn parse_outdated_json(&self, json: &str, package_type: PackageType) -> Result<Vec<Package>> {
        let data: Value = serde_json::from_str(json)?;
        let mut packages = Vec::new();

        let items_key = match package_type {
            PackageType::Formula => "formulae",
            PackageType::Cask => "casks",
        };

        // Get the list of pinned packages
        let pinned_packages = self.get_pinned_packages().unwrap_or_default();

        if let Some(items) = data.get(items_key).and_then(|v| v.as_array()) {
            for item in items {
                if let Some(name) = item.get("name").and_then(|v| v.as_str()) {
                    let version = item
                        .get("installed_versions")
                        .and_then(|v| v.as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|v| v.as_str())
                        .map(String::from);

                    let available_version = item
                        .get("current_version")
                        .and_then(|v| v.as_str())
                        .map(String::from);

                    let is_pinned = pinned_packages.contains(&name.to_string());

                    let mut package = Package::new(name.to_string(), package_type.clone())
                        .set_installed(true)
                        .set_outdated(true)
                        .with_version(version.unwrap_or_default())
                        .set_pinned(is_pinned);

                    if let Some(av) = available_version {
                        package = package.with_available_version(av);
                    }

                    packages.push(package);
                }
            }
        }

        Ok(packages)
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

            if let Some(path_str) = trimmed.strip_prefix("Would remove: ").or_else(|| Some(trimmed)) {
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
}

#[async_trait]
impl PackageRepository for BrewPackageRepository {
    async fn get_installed_packages(&self, package_type: PackageType) -> Result<Vec<Package>> {
        let package_type_clone = package_type.clone();
        let output = tokio::task::spawn_blocking(move || match package_type_clone {
            PackageType::Formula => BrewCommand::list_formulae(),
            PackageType::Cask => BrewCommand::list_casks(),
        })
        .await??;

        self.parse_installed_packages(&output, package_type)
    }

    async fn get_outdated_packages(&self, package_type: PackageType) -> Result<Vec<Package>> {
        let package_type_clone = package_type.clone();
        let output = tokio::task::spawn_blocking(move || match package_type_clone {
            PackageType::Formula => BrewCommand::outdated_formulae(),
            PackageType::Cask => BrewCommand::outdated_casks(),
        })
        .await??;

        self.parse_outdated_json(&output, package_type)
    }

    async fn install_package(&self, package: &Package) -> Result<()> {
        let name = package.name.clone();
        let package_type = package.package_type.clone();

        tokio::task::spawn_blocking(move || match package_type {
            PackageType::Formula => BrewCommand::install_formula(&name),
            PackageType::Cask => BrewCommand::install_cask(&name),
        })
        .await?
    }

    async fn uninstall_package(&self, package: &Package) -> Result<()> {
        let name = package.name.clone();
        let package_type = package.package_type.clone();

        tokio::task::spawn_blocking(move || match package_type {
            PackageType::Formula => BrewCommand::uninstall_formula(&name),
            PackageType::Cask => BrewCommand::uninstall_cask(&name),
        })
        .await?
    }

    async fn update_package(&self, package: &Package) -> Result<()> {
        let name = package.name.clone();

        tokio::task::spawn_blocking(move || BrewCommand::upgrade_package(&name)).await?
    }

    async fn update_all(&self) -> Result<()> {
        tokio::task::spawn_blocking(|| BrewCommand::upgrade_all()).await?
    }

    async fn get_cleanup_preview(&self) -> Result<CleanupPreview> {
        let output = tokio::task::spawn_blocking(|| BrewCommand::cleanup_dry_run()).await??;
        self.parse_cleanup_output(&output)
    }

    async fn get_cleanup_old_versions_preview(&self) -> Result<CleanupPreview> {
        let output = tokio::task::spawn_blocking(|| BrewCommand::cleanup_old_versions_dry_run()).await??;
        self.parse_cleanup_output(&output)
    }

    async fn clean_cache(&self) -> Result<()> {
        tokio::task::spawn_blocking(|| BrewCommand::cleanup()).await?
    }

    async fn cleanup_old_versions(&self) -> Result<()> {
        tokio::task::spawn_blocking(|| BrewCommand::cleanup_old_versions()).await?
    }

    async fn search_packages(&self, query: &str, package_type: PackageType) -> Result<Vec<Package>> {
        let query = query.to_string();
        let package_type_clone = package_type.clone();
        let output = tokio::task::spawn_blocking(move || match package_type_clone {
            PackageType::Formula => BrewCommand::search_formula(&query),
            PackageType::Cask => BrewCommand::search_cask(&query),
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
            tokio::task::spawn_blocking(move || match package_type_clone {
                PackageType::Formula => BrewCommand::get_formula_info(&name_clone),
                PackageType::Cask => BrewCommand::get_cask_info(&name_clone),
            })
        )
        .await
        .map_err(|_| anyhow::anyhow!("Timeout loading package info for {}", name))???;
        
        tracing::debug!("Raw brew output for {}: {} bytes", name, output.len());

        let data: Value = serde_json::from_str(&output)
            .map_err(|e| {
                tracing::error!("Failed to parse JSON for {}: {}", name, e);
                e
            })?;
        
        tracing::debug!("Parsed JSON for {}: {:?}", name, data);
        
        let items_key = match package_type {
            PackageType::Formula => "formulae",
            PackageType::Cask => "casks",
        };

        if let Some(items) = data.get(items_key).and_then(|v| v.as_array()) {
            tracing::debug!("Found {} items for {} in '{}'", items.len(), name, items_key);
            
            if let Some(item) = items.first() {
                let version = item
                    .get("version")
                    .or_else(|| item.get("versions").and_then(|v| v.get("stable")))
                    .and_then(|v| v.as_str())
                    .map(String::from);

                let description = item
                    .get("desc")
                    .and_then(|v| v.as_str())
                    .map(String::from);

                tracing::debug!("Extracted for {}: version={:?}, desc={:?}", name, version, description);

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
        tokio::task::spawn_blocking(move || BrewCommand::pin_package(&name)).await?
    }

    async fn unpin_package(&self, package: &Package) -> Result<()> {
        let name = package.name.clone();
        tokio::task::spawn_blocking(move || BrewCommand::unpin_package(&name)).await?
    }
}
