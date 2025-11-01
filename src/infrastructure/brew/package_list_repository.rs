use crate::domain::{
    entities::{PackageList, PackageListItem, PackageType},
    repositories::PackageListRepository,
};
use crate::infrastructure::brew::command::BrewCommand;
use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;

pub struct BrewPackageListRepository;

impl BrewPackageListRepository {
    pub fn new() -> Self {
        Self
    }

    fn parse_package_list(&self, output: &str) -> Result<PackageList> {
        let mut package_list = PackageList::new();
        let export_date = Utc::now().to_rfc3339();
        package_list = package_list.with_export_date(export_date);

        let mut current_section = None;

        for line in output.lines() {
            let trimmed = line.trim();

            if trimmed == "FORMULAE" {
                current_section = Some(PackageType::Formula);
                continue;
            } else if trimmed == "CASKS" {
                current_section = Some(PackageType::Cask);
                continue;
            }

            if trimmed.is_empty() {
                continue;
            }

            if let Some(ref package_type) = current_section {
                // Parse package name and version
                // Format from "brew list --versions": "package-name version1 version2 ..."
                // We'll take the first version if multiple exist
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                
                if parts.is_empty() {
                    continue;
                }
                
                let name = parts[0].to_string();
                let version = if parts.len() > 1 {
                    Some(parts[1].to_string())
                } else {
                    None
                };
                
                let mut item = PackageListItem::new(name, package_type.clone());
                if let Some(ver) = version {
                    item = item.with_version(ver);
                }

                match package_type {
                    PackageType::Formula => package_list.add_formula(item),
                    PackageType::Cask => package_list.add_cask(item),
                }
            }
        }

        Ok(package_list)
    }
}

#[async_trait]
impl PackageListRepository for BrewPackageListRepository {
    async fn export_package_list(&self) -> Result<PackageList> {
        let output = tokio::task::spawn_blocking(|| BrewCommand::export_installed()).await??;
        self.parse_package_list(&output)
    }

    async fn import_packages(&self, package_list: &PackageList) -> Result<Vec<String>> {
        let mut installed = Vec::new();
        let mut failed = Vec::new();

        // Install formulae
        for item in &package_list.formulae {
            let name = item.name.clone();
            let package_type = item.package_type.clone();

            match tokio::task::spawn_blocking(move || {
                BrewCommand::install_package(&name, package_type)
            })
            .await?
            {
                Ok(_) => {
                    installed.push(item.name.clone());
                    tracing::info!("Successfully installed formula: {}", item.name);
                }
                Err(e) => {
                    failed.push(item.name.clone());
                    tracing::error!("Failed to install formula {}: {}", item.name, e);
                }
            }
        }

        // Install casks
        for item in &package_list.casks {
            let name = item.name.clone();
            let package_type = item.package_type.clone();

            match tokio::task::spawn_blocking(move || {
                BrewCommand::install_package(&name, package_type)
            })
            .await?
            {
                Ok(_) => {
                    installed.push(item.name.clone());
                    tracing::info!("Successfully installed cask: {}", item.name);
                }
                Err(e) => {
                    failed.push(item.name.clone());
                    tracing::error!("Failed to install cask {}: {}", item.name, e);
                }
            }
        }

        if !failed.is_empty() {
            tracing::warn!(
                "Imported {} packages, {} failed: {:?}",
                installed.len(),
                failed.len(),
                failed
            );
        }

        Ok(installed)
    }
}
