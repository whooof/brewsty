use crate::domain::{entities::PackageList, repositories::PackageListRepository};
use anyhow::{Context, Result, bail};
use std::{path::Path, sync::Arc};

/// Validates that a package name contains only safe characters
fn is_valid_package_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 255
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '@' | '/' | '+'))
}

pub struct PackageListRepositoryUseCase {
    repository: Arc<dyn PackageListRepository>,
}

impl PackageListRepositoryUseCase {
    pub fn new(repository: Arc<dyn PackageListRepository>) -> Self {
        Self { repository }
    }

    pub fn repository(&self) -> Arc<dyn PackageListRepository> {
        Arc::clone(&self.repository)
    }
}

pub struct ExportPackages {
    use_case: PackageListRepositoryUseCase,
}

impl ExportPackages {
    pub fn new(repository: Arc<dyn PackageListRepository>) -> Self {
        Self {
            use_case: PackageListRepositoryUseCase::new(repository),
        }
    }

    pub async fn execute(&self, path: &Path) -> Result<PackageList> {
        // Get the package list from brew
        let package_list = self.use_case.repository().export_package_list().await?;

        // Serialize to JSON
        let json = serde_json::to_string_pretty(&package_list)
            .context("Failed to serialize package list to JSON")?;

        // Write to file
        tokio::fs::write(path, json)
            .await
            .context("Failed to write package list to file")?;

        Ok(package_list)
    }
}

pub struct ImportPackages {
    use_case: PackageListRepositoryUseCase,
}

impl ImportPackages {
    pub fn new(repository: Arc<dyn PackageListRepository>) -> Self {
        Self {
            use_case: PackageListRepositoryUseCase::new(repository),
        }
    }

    pub async fn execute(&self, path: &Path) -> Result<()> {
        // Read the JSON file
        let json = tokio::fs::read_to_string(path)
            .await
            .context("Failed to read package list file")?;

        // Deserialize from JSON
        let package_list: PackageList =
            serde_json::from_str(&json).context("Failed to parse package list JSON")?;

        // Validate all package names before installing anything
        for item in package_list
            .formulae
            .iter()
            .chain(package_list.casks.iter())
        {
            if !is_valid_package_name(&item.name) {
                bail!("Invalid package name: {:?}", item.name);
            }
        }

        // Import the packages
        let _installed = self
            .use_case
            .repository()
            .import_packages(&package_list)
            .await?;

        Ok(())
    }
}
