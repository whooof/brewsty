use crate::domain::entities::Package;

pub struct PackageValidator;

impl PackageValidator {
    pub fn validate_package_name(name: &str) -> bool {
        !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    }

    pub fn validate_package(package: &Package) -> Result<(), String> {
        if !Self::validate_package_name(&package.name) {
            return Err(format!("Invalid package name: {}", package.name));
        }
        Ok(())
    }
}
