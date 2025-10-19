use crate::application::use_cases::{InstallPackage, PinPackage, UninstallPackage, UnpinPackage, UpdatePackage};
use crate::domain::entities::Package;
use crate::presentation::services::AsyncExecutor;
use std::sync::Arc;

pub struct OperationResult {
    pub success: bool,
    pub message: String,
    pub log_messages: Vec<String>,
}

pub struct PackageOperationHandler {
    install_use_case: Arc<InstallPackage>,
    uninstall_use_case: Arc<UninstallPackage>,
    update_use_case: Arc<UpdatePackage>,
    pin_use_case: Arc<PinPackage>,
    unpin_use_case: Arc<UnpinPackage>,
    executor: AsyncExecutor,
}

impl PackageOperationHandler {
    pub fn new(
        install_use_case: Arc<InstallPackage>,
        uninstall_use_case: Arc<UninstallPackage>,
        update_use_case: Arc<UpdatePackage>,
        pin_use_case: Arc<PinPackage>,
        unpin_use_case: Arc<UnpinPackage>,
        executor: AsyncExecutor,
    ) -> Self {
        Self {
            install_use_case,
            uninstall_use_case,
            update_use_case,
            pin_use_case,
            unpin_use_case,
            executor,
        }
    }

    pub fn install(&self, package: Package) -> OperationResult {
        let mut logs = Vec::new();
        let package_clone = package.clone();
        let package_name = package.name.clone();
        let package_type = package.package_type;
        let initial_msg = format!("Installing package: {} ({:?})", package_name, package_type);
        logs.push(initial_msg.clone());
        tracing::info!("{}", initial_msg);

        let use_case = Arc::clone(&self.install_use_case);
        let result = self.executor.execute(async move {
            use_case.execute(package_clone).await
        });

        match result {
            Ok(_) => {
                let msg = format!("Successfully installed {}", package_name);
                logs.push(msg.clone());
                tracing::info!("{}", msg);
                OperationResult {
                    success: true,
                    message: format!("{} installed successfully", package_name),
                    log_messages: logs,
                }
            }
            Err(e) => {
                let msg = format!("Error installing {}: {}", package_name, e);
                logs.push(msg.clone());
                tracing::error!("{}", msg);
                OperationResult {
                    success: false,
                    message: msg.to_string(),
                    log_messages: logs,
                }
            }
        }
    }

    pub fn uninstall(&self, package: Package) -> OperationResult {
        let mut logs = Vec::new();
        let package_clone = package.clone();
        let package_name = package.name.clone();
        let package_type = package.package_type;
        let initial_msg = format!("Uninstalling package: {} ({:?})", package_name, package_type);
        logs.push(initial_msg.clone());
        tracing::info!("{}", initial_msg);

        let use_case = Arc::clone(&self.uninstall_use_case);
        let result = self.executor.execute(async move {
            use_case.execute(package_clone).await
        });

        match result {
            Ok(_) => {
                let msg = format!("Successfully uninstalled {}", package_name);
                logs.push(msg.clone());
                tracing::info!("{}", msg);
                OperationResult {
                    success: true,
                    message: format!("{} uninstalled successfully", package_name),
                    log_messages: logs,
                }
            }
            Err(e) => {
                let msg = format!("Error uninstalling {}: {}", package_name, e);
                logs.push(msg.clone());
                tracing::error!("{}", msg);
                OperationResult {
                    success: false,
                    message: msg.to_string(),
                    log_messages: logs,
                }
            }
        }
    }

    pub fn update(&self, package: Package) -> OperationResult {
        let mut logs = Vec::new();
        let package_clone = package.clone();
        let package_name = package.name.clone();
        let package_type = package.package_type;
        let initial_msg = format!("Updating package: {} ({:?})", package_name, package_type);
        logs.push(initial_msg.clone());
        tracing::info!("{}", initial_msg);

        let use_case = Arc::clone(&self.update_use_case);
        let result = self.executor.execute(async move {
            use_case.execute(package_clone).await
        });

        match result {
            Ok(_) => {
                let msg = format!("Successfully updated {}", package_name);
                logs.push(msg.clone());
                tracing::info!("{}", msg);
                OperationResult {
                    success: true,
                    message: format!("{} updated successfully", package_name),
                    log_messages: logs,
                }
            }
            Err(e) => {
                let msg = format!("Error updating {}: {}", package_name, e);
                logs.push(msg.clone());
                tracing::error!("{}", msg);
                OperationResult {
                    success: false,
                    message: msg.to_string(),
                    log_messages: logs,
                }
            }
        }
    }

    pub fn pin(&self, package: Package) -> OperationResult {
        let mut logs = Vec::new();
        let package_clone = package.clone();
        let package_name = package.name.clone();
        let package_type = package.package_type;
        let initial_msg = format!("Pinning package: {} ({:?})", package_name, package_type);
        logs.push(initial_msg.clone());
        tracing::info!("{}", initial_msg);

        let use_case = Arc::clone(&self.pin_use_case);
        let result = self.executor.execute(async move {
            use_case.execute(package_clone).await
        });

        match result {
            Ok(_) => {
                let msg = format!("Successfully pinned {}", package_name);
                logs.push(msg.clone());
                tracing::info!("{}", msg);
                OperationResult {
                    success: true,
                    message: format!("{} pinned successfully", package_name),
                    log_messages: logs,
                }
            }
            Err(e) => {
                let msg = format!("Error pinning {}: {}", package_name, e);
                logs.push(msg.clone());
                tracing::error!("{}", msg);
                OperationResult {
                    success: false,
                    message: msg.to_string(),
                    log_messages: logs,
                }
            }
        }
    }

    pub fn unpin(&self, package: Package) -> OperationResult {
        let mut logs = Vec::new();
        let package_clone = package.clone();
        let package_name = package.name.clone();
        let package_type = package.package_type;
        let initial_msg = format!("Unpinning package: {} ({:?})", package_name, package_type);
        logs.push(initial_msg.clone());
        tracing::info!("{}", initial_msg);

        let use_case = Arc::clone(&self.unpin_use_case);
        let result = self.executor.execute(async move {
            use_case.execute(package_clone).await
        });

        match result {
            Ok(_) => {
                let msg = format!("Successfully unpinned {}", package_name);
                logs.push(msg.clone());
                tracing::info!("{}", msg);
                OperationResult {
                    success: true,
                    message: format!("{} unpinned successfully", package_name),
                    log_messages: logs,
                }
            }
            Err(e) => {
                let msg = format!("Error unpinning {}: {}", package_name, e);
                logs.push(msg.clone());
                tracing::error!("{}", msg);
                OperationResult {
                    success: false,
                    message: msg.to_string(),
                    log_messages: logs,
                }
            }
        }
    }
}
