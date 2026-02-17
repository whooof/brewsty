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
                
                let mut item = PackageListItem::new(name, *package_type);
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
        let output = tokio::task::spawn_blocking(BrewCommand::export_installed).await??;
        self.parse_package_list(&output)
    }

    async fn import_packages(&self, package_list: &PackageList) -> Result<Vec<String>> {
        let mut installed = Vec::new();
        let mut failed = Vec::new();

        // Batch install formulae
        if !package_list.formulae.is_empty() {
            let names: Vec<String> = package_list.formulae.iter().map(|i| i.name.clone()).collect();
            tracing::info!("Batch installing {} formulae", names.len());

            let names_clone = names.clone();
            match tokio::task::spawn_blocking(move || {
                let args: Vec<&str> = std::iter::once("install")
                    .chain(std::iter::once("--formula"))
                    .chain(names_clone.iter().map(|s| s.as_str()))
                    .collect();
                BrewCommand::execute_brew(&args)
            }).await? {
                Ok(_) => {
                    tracing::info!("Batch installed {} formulae", names.len());
                    installed.extend(names);
                }
                Err(e) => {
                    tracing::warn!("Batch formula install failed, falling back to individual: {}", e);
                    for item in &package_list.formulae {
                        let name = item.name.clone();
                        let package_type = item.package_type;
                        match tokio::task::spawn_blocking(move || {
                            BrewCommand::install_package(&name, package_type)
                        }).await? {
                            Ok(_) => {
                                installed.push(item.name.clone());
                                tracing::info!("Installed formula: {}", item.name);
                            }
                            Err(e) => {
                                failed.push(item.name.clone());
                                tracing::error!("Failed to install formula {}: {}", item.name, e);
                            }
                        }
                    }
                }
            }
        }

        // Batch install casks
        if !package_list.casks.is_empty() {
            let names: Vec<String> = package_list.casks.iter().map(|i| i.name.clone()).collect();
            tracing::info!("Batch installing {} casks", names.len());

            let names_clone = names.clone();
            match tokio::task::spawn_blocking(move || {
                let args: Vec<&str> = std::iter::once("install")
                    .chain(std::iter::once("--cask"))
                    .chain(names_clone.iter().map(|s| s.as_str()))
                    .collect();
                BrewCommand::execute_brew(&args)
            }).await? {
                Ok(_) => {
                    tracing::info!("Batch installed {} casks", names.len());
                    installed.extend(names);
                }
                Err(e) => {
                    tracing::warn!("Batch cask install failed, falling back to individual: {}", e);
                    for item in &package_list.casks {
                        let name = item.name.clone();
                        let package_type = item.package_type;
                        match tokio::task::spawn_blocking(move || {
                            BrewCommand::install_package(&name, package_type)
                        }).await? {
                            Ok(_) => {
                                installed.push(item.name.clone());
                                tracing::info!("Installed cask: {}", item.name);
                            }
                            Err(e) => {
                                failed.push(item.name.clone());
                                tracing::error!("Failed to install cask {}: {}", item.name, e);
                            }
                        }
                    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn repo() -> BrewPackageListRepository {
        BrewPackageListRepository::new()
    }

    #[test]
    fn parse_package_list_formulae_and_casks() {
        let output = "FORMULAE\nwget 1.21.4\ncurl 8.4.0\n\nCASKS\nfirefox 120.0\n";
        let result = repo().parse_package_list(output).unwrap();
        assert_eq!(result.formulae.len(), 2);
        assert_eq!(result.casks.len(), 1);
        assert_eq!(result.formulae[0].name, "wget");
        assert_eq!(result.formulae[0].version.as_deref(), Some("1.21.4"));
        assert_eq!(result.formulae[0].package_type, PackageType::Formula);
        assert_eq!(result.casks[0].name, "firefox");
        assert_eq!(result.casks[0].package_type, PackageType::Cask);
    }

    #[test]
    fn parse_package_list_formulae_only() {
        let output = "FORMULAE\ngit 2.43.0\n";
        let result = repo().parse_package_list(output).unwrap();
        assert_eq!(result.formulae.len(), 1);
        assert!(result.casks.is_empty());
    }

    #[test]
    fn parse_package_list_no_version() {
        let output = "FORMULAE\nwget\n";
        let result = repo().parse_package_list(output).unwrap();
        assert_eq!(result.formulae.len(), 1);
        assert_eq!(result.formulae[0].name, "wget");
        assert!(result.formulae[0].version.is_none());
    }

    #[test]
    fn parse_package_list_empty() {
        let output = "";
        let result = repo().parse_package_list(output).unwrap();
        assert!(result.formulae.is_empty());
        assert!(result.casks.is_empty());
    }

    #[test]
    fn parse_package_list_export_date_set() {
        let output = "FORMULAE\nwget 1.0\n";
        let result = repo().parse_package_list(output).unwrap();
        assert!(result.export_date.is_some());
    }

    #[test]
    fn total_count() {
        let output = "FORMULAE\nwget 1.0\ncurl 2.0\n\nCASKS\nfirefox 120.0\n";
        let result = repo().parse_package_list(output).unwrap();
        assert_eq!(result.total_count(), 3);
    }
}
