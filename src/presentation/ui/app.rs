use crate::application::UseCaseContainer;
use crate::domain::entities::{AppConfig, Package, PackageType};
use crate::infrastructure::config_repository::ConfigRepository;
use crate::presentation::components::{
    CleanupAction, CleanupModal, CleanupType, FilterState, InfoModal, LogManager,
    MergedPackageList, PackageList, PasswordModal, ServiceList, Tab, TabManager,
};
use crate::presentation::services::{AsyncExecutor, AsyncTask, AsyncTaskManager};
use crate::presentation::ui::tabs::installed::{InstalledAction, InstalledTab};
use crate::presentation::ui::tabs::log::{LogAction, LogTab};
use crate::presentation::ui::tabs::search::{SearchAction, SearchTab};
use crate::presentation::ui::tabs::services::{ServiceAction, ServicesTab};
use crate::presentation::ui::tabs::settings::{SettingsAction, SettingsTab};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

pub struct BrewstyApp {
    tab_manager: TabManager,
    filter_state: FilterState,

    config: AppConfig,
    config_repo: ConfigRepository,

    cleanup_modal: CleanupModal,
    info_modal: InfoModal,
    password_modal: PasswordModal,
    log_manager: LogManager,
    log_rx: Receiver<String>,

    merged_packages: MergedPackageList,
    search_results: PackageList,
    service_list: ServiceList,

    auto_load_version_info: bool,

    initialized: bool,

    loading_installed: bool,
    loading_outdated: bool,
    loading_search: bool,
    loading_services: bool,

    loading_install: bool,
    loading_uninstall: bool,
    loading_update: bool,
    loading_update_all: bool,
    loading_clean_cache: bool,
    loading_cleanup_old_versions: bool,
    loading_export: bool,
    loading_import: bool,

    current_install_package: Option<String>,
    current_uninstall_package: Option<String>,
    current_update_package: Option<String>,
    pending_updates: Vec<Package>,
    pending_operation: Option<PendingOperation>,
    packages_in_operation: std::collections::HashSet<String>,
    services_in_operation: std::collections::HashSet<String>,

    task_manager: AsyncTaskManager,

    use_cases: Arc<UseCaseContainer>,
    executor: AsyncExecutor,

    loading: bool,
    status_message: String,
    output_panel_height: f32,
}

#[derive(Clone, Debug)]
enum PendingOperation {
    Install(Package),
    Uninstall(Package),
}

impl BrewstyApp {
    pub fn new(
        use_cases: Arc<UseCaseContainer>,
        log_rx: Receiver<String>,
        executor: AsyncExecutor,
    ) -> Self {
        let config_repo = ConfigRepository::new();
        let config = config_repo.load().unwrap_or_else(|e| {
            tracing::error!("Failed to load config: {}", e);
            AppConfig::default()
        });

        Self {
            tab_manager: TabManager::new(),
            filter_state: FilterState::new(),

            config: config.clone(),
            config_repo,

            cleanup_modal: CleanupModal::new(),
            info_modal: InfoModal::new(),
            password_modal: PasswordModal::new(),
            log_manager: LogManager::new(),
            log_rx,
            merged_packages: MergedPackageList::new(),
            search_results: PackageList::new(),
            service_list: ServiceList::new(),
            auto_load_version_info: false,
            initialized: false,
            loading_installed: false,
            loading_outdated: false,
            loading_search: false,
            loading_services: false,
            loading_install: false,
            loading_uninstall: false,
            loading_update: false,
            loading_update_all: false,
            loading_clean_cache: false,
            loading_cleanup_old_versions: false,
            loading_export: false,
            loading_import: false,
            current_install_package: None,
            current_uninstall_package: None,
            current_update_package: None,
            pending_updates: Vec::new(),
            pending_operation: None,
            packages_in_operation: std::collections::HashSet::new(),
            services_in_operation: std::collections::HashSet::new(),
            task_manager: AsyncTaskManager::new(),
            use_cases,
            executor,
            loading: false,
            status_message: String::new(),
            output_panel_height: 250.0,
        }
    }

    fn save_config(&self) {
        if let Err(e) = self.config_repo.save(&self.config) {
            tracing::error!("Failed to save config: {}", e);
        }
    }

    fn apply_theme(&self, ctx: &egui::Context) {
        crate::presentation::style::configure_style(ctx, self.config.theme);
    }

    fn load_installed_packages(&mut self, include_outdated: bool) {
        if self.loading_installed || self.loading_outdated {
            return;
        }

        self.loading_installed = true;
        self.loading_installed = true;
        if include_outdated {
            self.loading_outdated = true;
        }
        self.status_message = if include_outdated {
            "Loading installed and outdated packages...".to_string()
        } else {
            "Loading installed packages...".to_string()
        };

        if include_outdated {
            self.log_manager
                .push("Loading installed and outdated packages (formulae and casks)".to_string());
            tracing::info!("Loading installed and outdated packages (formulae and casks)");
        } else {
            self.log_manager
                .push("Loading installed packages (formulae and casks)".to_string());
            tracing::info!("Loading installed packages (formulae and casks)");
        }

        let use_case_installed = Arc::clone(&self.use_cases.list_installed);
        let use_case_outdated = Arc::clone(&self.use_cases.list_outdated);

        let installed_packages = Arc::new(Mutex::new(Vec::new()));
        let outdated_packages = Arc::new(Mutex::new(Vec::new()));
        let installed_log = Arc::new(Mutex::new(Vec::new()));
        let outdated_log = Arc::new(Mutex::new(Vec::new()));

        self.task_manager.set_active_task(AsyncTask::LoadInstalled {
            packages: Arc::clone(&installed_packages),
            logs: Arc::clone(&installed_log),
        });

        if include_outdated {
            self.task_manager.set_active_task(AsyncTask::LoadOutdated {
                packages: Arc::clone(&outdated_packages),
                logs: Arc::clone(&outdated_log),
            });
        }

        self.executor.spawn(async move {
            tracing::trace!("TASK STARTED: load_installed_packages");
            let task_result = async {
                tracing::debug!("Starting to load installed packages");

                tracing::trace!("TASK: about to execute installed formulae");
                let installed_formulae_result =
                    use_case_installed.execute(PackageType::Formula).await;

                tracing::debug!(
                    "Installed formulae result: {:?}",
                    installed_formulae_result
                        .as_ref()
                        .map(|p| p.len())
                        .map_err(|e| e.to_string())
                );

                tracing::trace!("TASK: about to execute installed casks");
                let installed_casks_result = use_case_installed.execute(PackageType::Cask).await;

                tracing::debug!(
                    "Installed casks result: {:?}",
                    installed_casks_result
                        .as_ref()
                        .map(|p| p.len())
                        .map_err(|e| e.to_string())
                );

                let mut outdated_formulae_result: anyhow::Result<Vec<Package>> = Ok(Vec::new());
                let mut outdated_casks_result: anyhow::Result<Vec<Package>> = Ok(Vec::new());

                if include_outdated {
                    tracing::trace!("TASK: about to execute outdated formulae");
                    outdated_formulae_result =
                        use_case_outdated.execute(PackageType::Formula).await;

                    tracing::debug!(
                        "Outdated formulae result: {:?}",
                        outdated_formulae_result
                            .as_ref()
                            .map(|p| p.len())
                            .map_err(|e| e.to_string())
                    );

                    tracing::trace!("TASK: about to execute outdated casks");
                    outdated_casks_result = use_case_outdated.execute(PackageType::Cask).await;

                    tracing::debug!(
                        "Outdated casks result: {:?}",
                        outdated_casks_result
                            .as_ref()
                            .map(|p| p.len())
                            .map_err(|e| e.to_string())
                    );
                }

                let mut installed = Vec::new();
                let mut outdated = Vec::new();
                let mut installed_logs_vec = Vec::new();
                let mut outdated_logs_vec = Vec::new();

                match installed_formulae_result {
                    Ok(pkgs) => {
                        let msg = format!("Loaded {} installed formulae", pkgs.len());
                        installed_logs_vec.push(msg.clone());
                        tracing::info!("{}", msg);
                        installed.extend(pkgs);
                    }
                    Err(e) => {
                        let msg = format!("Error loading installed formulae: {}", e);
                        installed_logs_vec.push(msg.clone());
                        tracing::error!("{}", msg);
                    }
                }

                match installed_casks_result {
                    Ok(pkgs) => {
                        let msg = format!("Loaded {} installed casks", pkgs.len());
                        installed_logs_vec.push(msg.clone());
                        tracing::info!("{}", msg);
                        installed.extend(pkgs);
                    }
                    Err(e) => {
                        let msg = format!("Error loading installed casks: {}", e);
                        installed_logs_vec.push(msg.clone());
                        tracing::error!("{}", msg);
                    }
                }

                if include_outdated {
                    match outdated_formulae_result {
                        Ok(pkgs) => {
                            let msg = format!("Loaded {} outdated formulae", pkgs.len());
                            outdated_logs_vec.push(msg.clone());
                            tracing::info!("{}", msg);
                            outdated.extend(pkgs);
                        }
                        Err(e) => {
                            let msg = format!("Error loading outdated formulae: {}", e);
                            outdated_logs_vec.push(msg.clone());
                            tracing::error!("{}", msg);
                        }
                    }

                    match outdated_casks_result {
                        Ok(pkgs) => {
                            let msg = format!("Loaded {} outdated casks", pkgs.len());
                            outdated_logs_vec.push(msg.clone());
                            tracing::info!("{}", msg);
                            outdated.extend(pkgs);
                        }
                        Err(e) => {
                            let msg = format!("Error loading outdated casks: {}", e);
                            outdated_logs_vec.push(msg.clone());
                            tracing::error!("{}", msg);
                        }
                    }
                }

                tracing::debug!(
                    "About to write {} installed packages to mutex",
                    installed.len()
                );
                *installed_packages
                    .lock()
                    .map_err(|e| anyhow::anyhow!("Failed to lock installed packages: {}", e))? =
                    installed;

                tracing::debug!(
                    "About to write {} outdated packages to mutex",
                    outdated.len()
                );
                *outdated_packages
                    .lock()
                    .map_err(|e| anyhow::anyhow!("Failed to lock outdated packages: {}", e))? =
                    outdated;

                installed_logs_vec.push("Finished loading installed packages".to_string());
                if include_outdated {
                    outdated_logs_vec.push("Finished loading outdated packages".to_string());
                    tracing::info!("Finished loading installed and outdated packages");
                } else {
                    tracing::info!("Finished loading installed packages");
                }

                tracing::debug!(
                    "About to lock installed logs mutex with {} log entries",
                    installed_logs_vec.len()
                );
                *installed_log
                    .lock()
                    .map_err(|e| anyhow::anyhow!("Failed to lock installed logs: {}", e))? =
                    installed_logs_vec;

                tracing::debug!(
                    "About to lock outdated logs mutex with {} log entries",
                    outdated_logs_vec.len()
                );
                *outdated_log
                    .lock()
                    .map_err(|e| anyhow::anyhow!("Failed to lock outdated logs: {}", e))? =
                    outdated_logs_vec;

                tracing::debug!("Successfully updated mutexes");

                Ok::<(), anyhow::Error>(())
            }
            .await;

            if let Err(e) = task_result {
                tracing::error!("Error in load_installed_packages task: {}", e);
                if let Ok(mut logs) = installed_log.lock() {
                    logs.push(format!("Task error: {}", e));
                }
            }
            tracing::trace!("TASK ENDED: load_installed_packages");
        });
    }

    fn handle_update_selected(&mut self, package_names: Vec<String>) {
        if self.loading_update_all {
            return;
        }

        let mut packages_to_update = Vec::new();

        for package_name in package_names {
            if let Some(package) = self.merged_packages.get_package(&package_name) {
                packages_to_update.push(package);
                self.packages_in_operation.insert(package_name);
            }
        }

        if packages_to_update.is_empty() {
            return;
        }

        let count = packages_to_update.len();
        self.status_message = format!("Queued {} packages for sequential update", count);
        self.log_manager
            .push(format!("Queued {} packages for sequential update", count));
        tracing::info!("Queued {} packages for sequential update", count);

        // Queue all packages for sequential update
        self.pending_updates = packages_to_update;
        self.loading_update_all = true;

        // Start updating the first package
        self.process_next_pending_update();
    }

    fn process_next_pending_update(&mut self) {
        if self.pending_updates.is_empty() {
            return;
        }

        let package = self.pending_updates.remove(0);
        let remaining = self.pending_updates.len();
        let total = self.packages_in_operation.len();
        let completed = total - remaining;

        self.status_message = format!(
            "Updating {}/{}: {}... ({} remaining)",
            completed, total, package.name, remaining
        );

        let msg = format!(
            "Updating {}/{}: {} ({} remaining)",
            completed, total, package.name, remaining
        );
        self.log_manager.push(msg);
        tracing::info!(
            "Processing package {}/{}: {}",
            completed,
            total,
            package.name
        );

        self.handle_update(package);
    }

    fn is_password_error(&self, error_msg: &str) -> bool {
        error_msg.contains("authentication failure")
            || error_msg.contains("sudo")
            || error_msg.contains("password")
            || error_msg.contains("Permission denied")
            || error_msg.contains("Incorrect password")
            || error_msg.contains("incorrect password attempt")
            || error_msg.contains("sorry, try again")
            || error_msg.contains("sudo: a password is required")
    }

    fn retry_with_password(&mut self, password: &str) {
        if let Some(operation) = self.pending_operation.take() {
            match operation {
                PendingOperation::Install(package) => {
                    self.handle_install_with_password(package, password.to_string());
                }
                PendingOperation::Uninstall(package) => {
                    self.handle_uninstall_with_password(package, password.to_string());
                }
            }
        }
    }

    fn handle_install(&mut self, package: Package) {
        if self.loading_install {
            return;
        }

        let package_name = package.name.clone();
        self.loading_install = true;
        self.loading = true;
        self.current_install_package = Some(package_name.clone());
        self.packages_in_operation.insert(package_name.clone());
        self.status_message = format!("Installing {}...", package.name);

        let package_type = package.package_type.clone();
        let initial_msg = format!("Installing package: {} ({:?})", package_name, package_type);
        self.log_manager.push(initial_msg.clone());
        tracing::info!("{}", initial_msg);

        let success = Arc::new(Mutex::new(None));
        let logs = Arc::new(Mutex::new(Vec::new()));
        let message = Arc::new(Mutex::new(String::new()));

        self.task_manager.set_active_task(AsyncTask::Install {
            success: Arc::clone(&success),
            logs: Arc::clone(&logs),
            message: Arc::clone(&message),
        });

        let use_case = Arc::clone(&self.use_cases.install);

        self.executor.spawn(async move {
            let result = use_case.execute(package).await;

            let mut log_vec = Vec::new();
            match result {
                Ok(_) => {
                    let msg = format!("Successfully installed {}", package_name);
                    log_vec.push(msg.clone());
                    tracing::info!("{}", msg);
                    if let Ok(mut success_guard) = success.lock() {
                        *success_guard = Some(true);
                    }
                    if let Ok(mut message_guard) = message.lock() {
                        *message_guard = format!("{} installed successfully", package_name);
                    }
                }
                Err(e) => {
                    let error_str = e.to_string();
                    let msg = format!("Error installing {}: {}", package_name, error_str);
                    log_vec.push(msg.clone());
                    tracing::error!("{}", msg);
                    if let Ok(mut success_guard) = success.lock() {
                        *success_guard = Some(false);
                    }
                    if let Ok(mut message_guard) = message.lock() {
                        *message_guard = error_str;
                    }
                }
            }

            if let Ok(mut logs_guard) = logs.lock() {
                *logs_guard = log_vec;
            }
        });
    }

    fn handle_install_with_password(&mut self, package: Package, password: String) {
        if self.loading_install {
            return;
        }

        let package_name = package.name.clone();
        self.loading_install = true;
        self.loading = true;
        self.current_install_package = Some(package_name.clone());
        self.status_message = format!("Installing {} (with password)...", package.name);

        let package_type = package.package_type.clone();
        let initial_msg = format!(
            "Retrying install with password: {} ({:?})",
            package_name, package_type
        );
        self.log_manager.push(initial_msg.clone());
        tracing::info!("{}", initial_msg);

        let success = Arc::new(Mutex::new(None));
        let logs = Arc::new(Mutex::new(Vec::new()));
        let message = Arc::new(Mutex::new(String::new()));

        self.task_manager.set_active_task(AsyncTask::Install {
            success: Arc::clone(&success),
            logs: Arc::clone(&logs),
            message: Arc::clone(&message),
        });

        let name = package_name.clone();
        let pkg_type = package_type.clone();

        self.executor.spawn(async move {
            use crate::infrastructure::brew::command::BrewCommand;

            let mut log_vec = Vec::new();

            let brew_result = tokio::task::spawn_blocking(move || {
                BrewCommand::install_package_with_password(&name, pkg_type, &password)
            })
            .await;

            let result = match brew_result {
                Ok(inner) => inner,
                Err(e) => Err(anyhow::anyhow!("Task join error: {}", e)),
            };

            match result {
                Ok(_) => {
                    let msg = format!("Successfully installed {}", package_name);
                    log_vec.push(msg.clone());
                    tracing::info!("{}", msg);
                    if let Ok(mut success_guard) = success.lock() {
                        *success_guard = Some(true);
                    }
                    if let Ok(mut message_guard) = message.lock() {
                        *message_guard = format!("{} installed successfully", package_name);
                    }
                }
                Err(e) => {
                    let error_str = e.to_string();
                    let msg = format!("Error installing {}: {}", package_name, error_str);
                    log_vec.push(msg.clone());
                    tracing::error!("{}", msg);
                    if let Ok(mut success_guard) = success.lock() {
                        *success_guard = Some(false);
                    }
                    if let Ok(mut message_guard) = message.lock() {
                        *message_guard = error_str;
                    }
                }
            }

            if let Ok(mut logs_guard) = logs.lock() {
                *logs_guard = log_vec;
            }
        });
    }

    fn handle_uninstall(&mut self, package: Package) {
        if self.loading_uninstall {
            return;
        }

        let package_name = package.name.clone();
        self.loading_uninstall = true;
        self.loading = true;
        self.current_uninstall_package = Some(package_name.clone());
        self.packages_in_operation.insert(package_name.clone());
        self.status_message = format!("Uninstalling {}...", package.name);

        let package_type = package.package_type.clone();
        let initial_msg = format!(
            "Uninstalling package: {} ({:?})",
            package_name, package_type
        );
        self.log_manager.push(initial_msg.clone());
        tracing::info!("{}", initial_msg);

        let success = Arc::new(Mutex::new(None));
        let logs = Arc::new(Mutex::new(Vec::new()));
        let message = Arc::new(Mutex::new(String::new()));

        self.task_manager.set_active_task(AsyncTask::Uninstall {
            success: Arc::clone(&success),
            logs: Arc::clone(&logs),
            message: Arc::clone(&message),
        });

        let use_case = Arc::clone(&self.use_cases.uninstall);

        self.executor.spawn(async move {
            let result = use_case.execute(package).await;

            let mut log_vec = Vec::new();
            match result {
                Ok(_) => {
                    let msg = format!("Successfully uninstalled {}", package_name);
                    log_vec.push(msg.clone());
                    tracing::info!("{}", msg);
                    if let Ok(mut success_guard) = success.lock() {
                        *success_guard = Some(true);
                    }
                    if let Ok(mut message_guard) = message.lock() {
                        *message_guard = format!("{} uninstalled successfully", package_name);
                    }
                }
                Err(e) => {
                    let error_str = e.to_string();
                    let msg = format!("Error uninstalling {}: {}", package_name, error_str);
                    log_vec.push(msg.clone());
                    tracing::error!("{}", msg);
                    if let Ok(mut success_guard) = success.lock() {
                        *success_guard = Some(false);
                    }
                    if let Ok(mut message_guard) = message.lock() {
                        *message_guard = error_str;
                    }
                }
            }

            if let Ok(mut logs_guard) = logs.lock() {
                *logs_guard = log_vec;
            }
        });
    }

    fn handle_uninstall_with_password(&mut self, package: Package, password: String) {
        if self.loading_uninstall {
            return;
        }

        let package_name = package.name.clone();
        self.loading_uninstall = true;
        self.loading = true;
        self.current_uninstall_package = Some(package_name.clone());
        self.status_message = format!("Uninstalling {} (with password)...", package.name);

        let package_type = package.package_type.clone();
        let initial_msg = format!(
            "Retrying uninstall with password: {} ({:?})",
            package_name, package_type
        );
        self.log_manager.push(initial_msg.clone());
        tracing::info!("{}", initial_msg);

        let success = Arc::new(Mutex::new(None));
        let logs = Arc::new(Mutex::new(Vec::new()));
        let message = Arc::new(Mutex::new(String::new()));

        self.task_manager.set_active_task(AsyncTask::Uninstall {
            success: Arc::clone(&success),
            logs: Arc::clone(&logs),
            message: Arc::clone(&message),
        });

        let name = package_name.clone();
        let pkg_type = package_type.clone();

        self.executor.spawn(async move {
            use crate::infrastructure::brew::command::BrewCommand;

            let mut log_vec = Vec::new();

            let brew_result = tokio::task::spawn_blocking(move || {
                BrewCommand::uninstall_package_with_password(&name, pkg_type, &password)
            })
            .await;

            let result = match brew_result {
                Ok(inner) => inner,
                Err(e) => Err(anyhow::anyhow!("Task join error: {}", e)),
            };

            match result {
                Ok(_) => {
                    let msg = format!("Successfully uninstalled {}", package_name);
                    log_vec.push(msg.clone());
                    tracing::info!("{}", msg);
                    if let Ok(mut success_guard) = success.lock() {
                        *success_guard = Some(true);
                    }
                    if let Ok(mut message_guard) = message.lock() {
                        *message_guard = format!("{} uninstalled successfully", package_name);
                    }
                }
                Err(e) => {
                    let error_str = e.to_string();
                    let msg = format!("Error uninstalling {}: {}", package_name, error_str);
                    log_vec.push(msg.clone());
                    tracing::error!("{}", msg);
                    if let Ok(mut success_guard) = success.lock() {
                        *success_guard = Some(false);
                    }
                    if let Ok(mut message_guard) = message.lock() {
                        *message_guard = error_str;
                    }
                }
            }

            if let Ok(mut logs_guard) = logs.lock() {
                *logs_guard = log_vec;
            }
        });
    }

    fn handle_update(&mut self, package: Package) {
        if self.loading_update {
            return;
        }

        let package_name = package.name.clone();
        self.loading_update = true;
        self.loading = true;
        self.current_update_package = Some(package_name.clone());
        self.packages_in_operation.insert(package_name.clone());
        self.status_message = format!("Updating {}...", package.name);

        let package_type = package.package_type.clone();
        let initial_msg = format!("Updating package: {} ({:?})", package_name, package_type);
        self.log_manager.push(initial_msg.clone());
        tracing::info!("{}", initial_msg);

        let success = Arc::new(Mutex::new(None));
        let logs = Arc::new(Mutex::new(Vec::new()));
        let message = Arc::new(Mutex::new(String::new()));

        self.task_manager.set_active_task(AsyncTask::Update {
            success: Arc::clone(&success),
            logs: Arc::clone(&logs),
            message: Arc::clone(&message),
        });

        let use_case = Arc::clone(&self.use_cases.update);

        self.executor.spawn(async move {
            let result = use_case.execute(&package).await;

            let mut log_vec = Vec::new();
            match result {
                Ok(_) => {
                    let msg = format!("Successfully updated {}", package_name);
                    log_vec.push(msg.clone());
                    tracing::info!("{}", msg);
                    if let Ok(mut success_guard) = success.lock() {
                        *success_guard = Some(true);
                    }
                    if let Ok(mut message_guard) = message.lock() {
                        *message_guard = format!("{} updated successfully", package_name);
                    }
                }
                Err(e) => {
                    let msg = format!("Error updating {}: {}", package_name, e);
                    log_vec.push(msg.clone());
                    tracing::error!("{}", msg);
                    if let Ok(mut success_guard) = success.lock() {
                        *success_guard = Some(false);
                    }
                    if let Ok(mut message_guard) = message.lock() {
                        *message_guard = msg;
                    }
                }
            }

            if let Ok(mut logs_guard) = logs.lock() {
                *logs_guard = log_vec;
            }
        });
    }

    fn handle_pin(&mut self, package: Package) {
        self.loading = true;
        self.packages_in_operation.insert(package.name.clone());
        self.status_message = format!("Pinning {}...", package.name);

        let package_name = package.name.clone();
        let package_type = package.package_type.clone();
        let initial_msg = format!("Pinning package: {} ({:?})", package_name, package_type);
        self.log_manager.push(initial_msg.clone());
        tracing::info!("{}", initial_msg);

        let success = Arc::new(Mutex::new(None));
        let logs = Arc::new(Mutex::new(Vec::new()));
        let message = Arc::new(Mutex::new(String::new()));

        self.task_manager.set_active_task(AsyncTask::Pin {
            package_name: package.name.clone(),
            success: Arc::clone(&success),
            logs: Arc::clone(&logs),
            message: Arc::clone(&message),
        });

        let use_case = Arc::clone(&self.use_cases.pin);
        let package_clone = package.clone();

        self.executor.spawn(async move {
            match use_case.execute(package_clone).await {
                Ok(_) => {
                    let msg = format!("Successfully pinned {}", package_name);
                    if let Ok(mut logs_guard) = logs.lock() {
                        *logs_guard = vec![msg.clone()];
                    }
                    if let Ok(mut success_guard) = success.lock() {
                        *success_guard = Some(true);
                    }
                    if let Ok(mut message_guard) = message.lock() {
                        *message_guard = format!("{} pinned successfully", package_name);
                    }
                }
                Err(e) => {
                    let msg = format!("Error pinning {}: {}", package_name, e);
                    if let Ok(mut logs_guard) = logs.lock() {
                        *logs_guard = vec![msg.clone()];
                    }
                    if let Ok(mut success_guard) = success.lock() {
                        *success_guard = Some(false);
                    }
                    if let Ok(mut message_guard) = message.lock() {
                        *message_guard = msg;
                    }
                }
            }
        });
    }

    fn handle_unpin(&mut self, package: Package) {
        self.loading = true;
        self.packages_in_operation.insert(package.name.clone());
        self.status_message = format!("Unpinning {}...", package.name);

        let package_name = package.name.clone();
        let package_type = package.package_type.clone();
        let initial_msg = format!("Unpinning package: {} ({:?})", package_name, package_type);
        self.log_manager.push(initial_msg.clone());
        tracing::info!("{}", initial_msg);

        let success = Arc::new(Mutex::new(None));
        let logs = Arc::new(Mutex::new(Vec::new()));
        let message = Arc::new(Mutex::new(String::new()));

        self.task_manager.set_active_task(AsyncTask::Unpin {
            package_name: package.name.clone(),
            success: Arc::clone(&success),
            logs: Arc::clone(&logs),
            message: Arc::clone(&message),
        });

        let use_case = Arc::clone(&self.use_cases.unpin);
        let package_clone = package.clone();

        self.executor.spawn(async move {
            match use_case.execute(package_clone).await {
                Ok(_) => {
                    let msg = format!("Successfully unpinned {}", package_name);
                    if let Ok(mut logs_guard) = logs.lock() {
                        *logs_guard = vec![msg.clone()];
                    }
                    if let Ok(mut success_guard) = success.lock() {
                        *success_guard = Some(true);
                    }
                    if let Ok(mut message_guard) = message.lock() {
                        *message_guard = format!("{} unpinned successfully", package_name);
                    }
                }
                Err(e) => {
                    let msg = format!("Error unpinning {}: {}", package_name, e);
                    if let Ok(mut logs_guard) = logs.lock() {
                        *logs_guard = vec![msg.clone()];
                    }
                    if let Ok(mut success_guard) = success.lock() {
                        *success_guard = Some(false);
                    }
                    if let Ok(mut message_guard) = message.lock() {
                        *message_guard = msg;
                    }
                }
            }
        });
    }

    fn load_services(&mut self) {
        if self.loading_services {
            return;
        }

        self.loading_services = true;
        self.status_message = "Loading services...".to_string();
        self.log_manager.push("Loading brew services".to_string());
        tracing::info!("Loading brew services");

        let use_case = Arc::clone(&self.use_cases.list_services);

        let services = Arc::new(Mutex::new(Vec::new()));
        let logs = Arc::new(Mutex::new(Vec::new()));

        self.task_manager.set_active_task(AsyncTask::LoadServices {
            services: Arc::clone(&services),
            logs: Arc::clone(&logs),
        });

        self.executor.spawn(async move {
            match use_case.execute().await {
                Ok(service_list) => {
                    let msg = format!("Loaded {} services", service_list.len());
                    tracing::info!("{}", msg);
                    if let Ok(mut services_guard) = services.lock() {
                        *services_guard = service_list;
                    }
                    if let Ok(mut logs_guard) = logs.lock() {
                        *logs_guard = vec![msg];
                    }
                }
                Err(e) => {
                    let msg = format!("Error loading services: {}", e);
                    tracing::error!("{}", msg);
                    if let Ok(mut logs_guard) = logs.lock() {
                        *logs_guard = vec![msg];
                    }
                }
            }
        });
    }

    fn handle_start_service(&mut self, service_name: String) {
        self.services_in_operation.insert(service_name.clone());
        self.status_message = format!("Starting service {}...", service_name);

        let initial_msg = format!("Starting service: {}", service_name);
        self.log_manager.push(initial_msg.clone());
        tracing::info!("{}", initial_msg);

        let success = Arc::new(Mutex::new(None));
        let logs = Arc::new(Mutex::new(Vec::new()));
        let message = Arc::new(Mutex::new(String::new()));

        self.task_manager.set_active_task(AsyncTask::StartService {
            service_name: service_name.clone(),
            success: Arc::clone(&success),
            logs: Arc::clone(&logs),
            message: Arc::clone(&message),
        });

        let use_case = Arc::clone(&self.use_cases.start_service);
        let service_name_clone = service_name.clone();

        self.executor.spawn(async move {
            match use_case.execute(&service_name_clone).await {
                Ok(_) => {
                    let msg = format!("Successfully started service {}", service_name);
                    if let Ok(mut logs_guard) = logs.lock() {
                        *logs_guard = vec![msg.clone()];
                    }
                    if let Ok(mut success_guard) = success.lock() {
                        *success_guard = Some(true);
                    }
                    if let Ok(mut message_guard) = message.lock() {
                        *message_guard = msg;
                    }
                }
                Err(e) => {
                    let msg = format!("Error starting service {}: {}", service_name, e);
                    if let Ok(mut logs_guard) = logs.lock() {
                        *logs_guard = vec![msg.clone()];
                    }
                    if let Ok(mut success_guard) = success.lock() {
                        *success_guard = Some(false);
                    }
                    if let Ok(mut message_guard) = message.lock() {
                        *message_guard = msg;
                    }
                }
            }
        });
    }

    fn handle_stop_service(&mut self, service_name: String) {
        self.services_in_operation.insert(service_name.clone());
        self.status_message = format!("Stopping service {}...", service_name);

        let initial_msg = format!("Stopping service: {}", service_name);
        self.log_manager.push(initial_msg.clone());
        tracing::info!("{}", initial_msg);

        let success = Arc::new(Mutex::new(None));
        let logs = Arc::new(Mutex::new(Vec::new()));
        let message = Arc::new(Mutex::new(String::new()));

        self.task_manager.set_active_task(AsyncTask::StopService {
            service_name: service_name.clone(),
            success: Arc::clone(&success),
            logs: Arc::clone(&logs),
            message: Arc::clone(&message),
        });

        let use_case = Arc::clone(&self.use_cases.stop_service);
        let service_name_clone = service_name.clone();

        self.executor.spawn(async move {
            match use_case.execute(&service_name_clone).await {
                Ok(_) => {
                    let msg = format!("Successfully stopped service {}", service_name);
                    if let Ok(mut logs_guard) = logs.lock() {
                        *logs_guard = vec![msg.clone()];
                    }
                    if let Ok(mut success_guard) = success.lock() {
                        *success_guard = Some(true);
                    }
                    if let Ok(mut message_guard) = message.lock() {
                        *message_guard = msg;
                    }
                }
                Err(e) => {
                    let msg = format!("Error stopping service {}: {}", service_name, e);
                    if let Ok(mut logs_guard) = logs.lock() {
                        *logs_guard = vec![msg.clone()];
                    }
                    if let Ok(mut success_guard) = success.lock() {
                        *success_guard = Some(false);
                    }
                    if let Ok(mut message_guard) = message.lock() {
                        *message_guard = msg;
                    }
                }
            }
        });
    }

    fn handle_restart_service(&mut self, service_name: String) {
        self.services_in_operation.insert(service_name.clone());
        self.status_message = format!("Restarting service {}...", service_name);

        let initial_msg = format!("Restarting service: {}", service_name);
        self.log_manager.push(initial_msg.clone());
        tracing::info!("{}", initial_msg);

        let success = Arc::new(Mutex::new(None));
        let logs = Arc::new(Mutex::new(Vec::new()));
        let message = Arc::new(Mutex::new(String::new()));

        self.task_manager
            .set_active_task(AsyncTask::RestartService {
                service_name: service_name.clone(),
                success: Arc::clone(&success),
                logs: Arc::clone(&logs),
                message: Arc::clone(&message),
            });

        let use_case = Arc::clone(&self.use_cases.restart_service);
        let service_name_clone = service_name.clone();

        self.executor.spawn(async move {
            match use_case.execute(&service_name_clone).await {
                Ok(_) => {
                    let msg = format!("Successfully restarted service {}", service_name);
                    if let Ok(mut logs_guard) = logs.lock() {
                        *logs_guard = vec![msg.clone()];
                    }
                    if let Ok(mut success_guard) = success.lock() {
                        *success_guard = Some(true);
                    }
                    if let Ok(mut message_guard) = message.lock() {
                        *message_guard = msg;
                    }
                }
                Err(e) => {
                    let msg = format!("Error restarting service {}: {}", service_name, e);
                    if let Ok(mut logs_guard) = logs.lock() {
                        *logs_guard = vec![msg.clone()];
                    }
                    if let Ok(mut success_guard) = success.lock() {
                        *success_guard = Some(false);
                    }
                    if let Ok(mut message_guard) = message.lock() {
                        *message_guard = msg;
                    }
                }
            }
        });
    }

    fn handle_export_packages(&mut self) {
        if self.loading_export {
            return;
        }

        // Open file save dialog
        let file_dialog = rfd::FileDialog::new()
            .add_filter("JSON files", &["json"])
            .set_file_name("brewsty_packages.json");

        if let Some(path) = file_dialog.save_file() {
            self.loading_export = true;
            self.loading = true;
            self.status_message = "Exporting packages...".to_string();
            self.log_manager
                .push(format!("Exporting packages to: {}", path.display()));
            tracing::info!("Exporting packages to: {}", path.display());

            let success = Arc::new(Mutex::new(None));
            let logs = Arc::new(Mutex::new(Vec::new()));
            let message = Arc::new(Mutex::new(String::new()));

            self.task_manager
                .set_active_task(AsyncTask::ExportPackages {
                    success: Arc::clone(&success),
                    logs: Arc::clone(&logs),
                    message: Arc::clone(&message),
                });

            let use_case = Arc::clone(&self.use_cases.export_packages);
            let path_display = path.display().to_string();

            self.executor.spawn(async move {
                let result: anyhow::Result<crate::domain::entities::PackageList> =
                    use_case.execute(&path).await;

                let mut log_vec = Vec::new();
                match result {
                    Ok(package_list) => {
                        let msg = format!(
                            "Successfully exported {} packages to {}",
                            package_list.total_count(),
                            path_display
                        );
                        log_vec.push(msg.clone());
                        tracing::info!("{}", msg);
                        if let Ok(mut success_guard) = success.lock() {
                            *success_guard = Some(true);
                        }
                        if let Ok(mut message_guard) = message.lock() {
                            *message_guard = "Packages exported successfully".to_string();
                        }
                    }
                    Err(e) => {
                        let msg = format!("Error exporting packages: {}", e);
                        log_vec.push(msg.clone());
                        tracing::error!("{}", msg);
                        if let Ok(mut success_guard) = success.lock() {
                            *success_guard = Some(false);
                        }
                        if let Ok(mut message_guard) = message.lock() {
                            *message_guard = msg;
                        }
                    }
                }

                if let Ok(mut logs_guard) = logs.lock() {
                    *logs_guard = log_vec;
                }
            });
        }
    }

    fn handle_import_packages(&mut self) {
        if self.loading_import {
            return;
        }

        // Open file open dialog
        let file_dialog = rfd::FileDialog::new()
            .add_filter("JSON files", &["json"])
            .set_file_name("brewsty_packages.json");

        if let Some(path) = file_dialog.pick_file() {
            self.loading_import = true;
            self.loading = true;
            self.status_message = "Importing packages...".to_string();
            self.log_manager
                .push(format!("Importing packages from: {}", path.display()));
            tracing::info!("Importing packages from: {}", path.display());

            let success = Arc::new(Mutex::new(None));
            let logs = Arc::new(Mutex::new(Vec::new()));
            let message = Arc::new(Mutex::new(String::new()));

            self.task_manager
                .set_active_task(AsyncTask::ImportPackages {
                    success: Arc::clone(&success),
                    logs: Arc::clone(&logs),
                    message: Arc::clone(&message),
                });

            let use_case = Arc::clone(&self.use_cases.import_packages);
            let path_display = path.display().to_string();

            self.executor.spawn(async move {
                let result = use_case.execute(&path).await;

                let mut log_vec = Vec::new();
                match result {
                    Ok(_) => {
                        let msg = format!("Successfully imported packages from {}", path_display);
                        log_vec.push(msg.clone());
                        tracing::info!("{}", msg);
                        if let Ok(mut success_guard) = success.lock() {
                            *success_guard = Some(true);
                        }
                        if let Ok(mut message_guard) = message.lock() {
                            *message_guard =
                                "Packages imported successfully. Reloading package list..."
                                    .to_string();
                        }
                    }
                    Err(e) => {
                        let msg = format!("Error importing packages: {}", e);
                        log_vec.push(msg.clone());
                        tracing::error!("{}", msg);
                        if let Ok(mut success_guard) = success.lock() {
                            *success_guard = Some(false);
                        }
                        if let Ok(mut message_guard) = message.lock() {
                            *message_guard = msg;
                        }
                    }
                }

                if let Ok(mut logs_guard) = logs.lock() {
                    *logs_guard = log_vec;
                }
            });
        }
    }

    fn handle_update_all(&mut self) {
        if self.loading_update_all {
            return;
        }

        self.loading_update_all = true;
        self.loading = true;
        self.status_message = "Updating all packages...".to_string();
        self.log_manager.push("Updating all packages".to_string());
        tracing::info!("Updating all packages");

        let success = Arc::new(Mutex::new(None));
        let logs = Arc::new(Mutex::new(Vec::new()));
        let message = Arc::new(Mutex::new(String::new()));

        self.task_manager.set_active_task(AsyncTask::UpdateAll {
            success: Arc::clone(&success),
            logs: Arc::clone(&logs),
            message: Arc::clone(&message),
        });

        let use_case = Arc::clone(&self.use_cases.update_all);

        self.executor.spawn(async move {
            let result = use_case.execute().await;

            let mut log_vec = Vec::new();
            match result {
                Ok(_) => {
                    let msg = "Successfully updated all packages".to_string();
                    log_vec.push(msg.clone());
                    tracing::info!("{}", msg);
                    if let Ok(mut success_guard) = success.lock() {
                        *success_guard = Some(true);
                    }
                    if let Ok(mut message_guard) = message.lock() {
                        *message_guard = "All packages updated successfully".to_string();
                    }
                }
                Err(e) => {
                    let msg = format!("Error updating all packages: {}", e);
                    log_vec.push(msg.clone());
                    tracing::error!("{}", msg);
                    if let Ok(mut success_guard) = success.lock() {
                        *success_guard = Some(false);
                    }
                    if let Ok(mut message_guard) = message.lock() {
                        *message_guard = msg;
                    }
                }
            }

            if let Ok(mut logs_guard) = logs.lock() {
                *logs_guard = log_vec;
            }
        });
    }

    fn show_cleanup_preview(&mut self, cleanup_type: CleanupType) {
        self.loading = true;
        self.status_message = "Loading cleanup preview...".to_string();
        self.log_manager.push("Loading cleanup preview".to_string());

        let preview_result = match cleanup_type {
            CleanupType::Cache => {
                let use_case = Arc::clone(&self.use_cases.clean_cache);
                self.executor.execute(async { use_case.preview().await })
            }
            CleanupType::OldVersions => {
                let use_case = Arc::clone(&self.use_cases.cleanup_old_versions);
                self.executor.execute(async { use_case.preview().await })
            }
        };

        match preview_result {
            Ok(preview) => {
                let msg = format!(
                    "Found {} items to clean ({})",
                    preview.items.len(),
                    format_size(preview.total_size)
                );
                self.log_manager.push(msg);
                self.cleanup_modal.show_preview(cleanup_type, preview);
            }
            Err(e) => {
                let msg = format!("Error getting cleanup preview: {}", e);
                self.log_manager.push(msg.clone());
                self.status_message = msg;
            }
        }

        self.loading = false;
    }

    fn handle_clean_cache(&mut self) {
        if self.loading_clean_cache {
            return;
        }

        self.loading_clean_cache = true;
        self.loading = true;
        self.status_message = "Cleaning cache...".to_string();
        self.log_manager.push("Cleaning Homebrew cache".to_string());
        tracing::info!("Cleaning Homebrew cache");

        let success = Arc::new(Mutex::new(None));
        let logs = Arc::new(Mutex::new(Vec::new()));
        let message = Arc::new(Mutex::new(String::new()));

        self.task_manager.set_active_task(AsyncTask::CleanCache {
            success: Arc::clone(&success),
            logs: Arc::clone(&logs),
            message: Arc::clone(&message),
        });

        let use_case = Arc::clone(&self.use_cases.clean_cache);

        self.executor.spawn(async move {
            let result = use_case.execute().await;

            let mut log_vec = Vec::new();
            match result {
                Ok(_) => {
                    let msg = "Successfully cleaned cache".to_string();
                    log_vec.push(msg.clone());
                    tracing::info!("{}", msg);
                    if let Ok(mut success_guard) = success.lock() {
                        *success_guard = Some(true);
                    }
                    if let Ok(mut message_guard) = message.lock() {
                        *message_guard = "Cache cleaned successfully".to_string();
                    }
                }
                Err(e) => {
                    let msg = format!("Error cleaning cache: {}", e);
                    log_vec.push(msg.clone());
                    tracing::error!("{}", msg);
                    if let Ok(mut success_guard) = success.lock() {
                        *success_guard = Some(false);
                    }
                    if let Ok(mut message_guard) = message.lock() {
                        *message_guard = msg;
                    }
                }
            }

            if let Ok(mut logs_guard) = logs.lock() {
                *logs_guard = log_vec;
            }
        });
    }

    fn handle_cleanup_old_versions(&mut self) {
        if self.loading_cleanup_old_versions {
            return;
        }

        self.loading_cleanup_old_versions = true;
        self.loading = true;
        self.status_message = "Cleaning up old versions...".to_string();
        self.log_manager
            .push("Cleaning up old versions".to_string());
        tracing::info!("Cleaning up old versions");

        let success = Arc::new(Mutex::new(None));
        let logs = Arc::new(Mutex::new(Vec::new()));
        let message = Arc::new(Mutex::new(String::new()));

        self.task_manager
            .set_active_task(AsyncTask::CleanupOldVersions {
                success: Arc::clone(&success),
                logs: Arc::clone(&logs),
                message: Arc::clone(&message),
            });

        let use_case = Arc::clone(&self.use_cases.cleanup_old_versions);

        self.executor.spawn(async move {
            let result = use_case.execute().await;

            let mut log_vec = Vec::new();
            match result {
                Ok(_) => {
                    let msg = "Successfully cleaned up old versions".to_string();
                    log_vec.push(msg.clone());
                    tracing::info!("{}", msg);
                    if let Ok(mut success_guard) = success.lock() {
                        *success_guard = Some(true);
                    }
                    if let Ok(mut message_guard) = message.lock() {
                        *message_guard = "Old versions cleaned up successfully".to_string();
                    }
                }
                Err(e) => {
                    let msg = format!("Error cleaning up old versions: {}", e);
                    log_vec.push(msg.clone());
                    tracing::error!("{}", msg);
                    if let Ok(mut success_guard) = success.lock() {
                        *success_guard = Some(false);
                    }
                    if let Ok(mut message_guard) = message.lock() {
                        *message_guard = msg;
                    }
                }
            }

            if let Ok(mut logs_guard) = logs.lock() {
                *logs_guard = log_vec;
            }
        });
    }

    fn handle_search(&mut self) {
        if self.filter_state.search_query().is_empty() {
            return;
        }

        if self.loading_search {
            return;
        }

        self.loading_search = true;
        self.status_message = format!("Searching for '{}'...", self.filter_state.search_query());
        let msg = format!("Searching for: {}", self.filter_state.search_query());
        self.log_manager.push(msg.clone());
        tracing::info!("{}", msg);

        let use_case_formulae = Arc::clone(&self.use_cases.search);
        let use_case_casks = Arc::clone(&self.use_cases.search);
        let query = self.filter_state.search_query().to_string();

        let search_results = Arc::new(Mutex::new(Vec::new()));
        let output_log = Arc::new(Mutex::new(Vec::new()));
        let query_clone = query.clone();

        self.task_manager.set_active_task(AsyncTask::Search {
            results: Arc::clone(&search_results),
            logs: Arc::clone(&output_log),
        });

        self.executor.spawn(async move {
            let (formulae_result, casks_result) = tokio::join!(
                use_case_formulae.execute(&query, PackageType::Formula),
                use_case_casks.execute(&query_clone, PackageType::Cask)
            );

            let mut results = Vec::new();
            let mut logs = Vec::new();

            match formulae_result {
                Ok(packages) => {
                    let msg = format!("Found {} formulae matching '{}'", packages.len(), query);
                    logs.push(msg.clone());
                    tracing::info!("{}", msg);
                    results.extend(packages);
                }
                Err(e) => {
                    let msg = format!("Error searching formulae: {}", e);
                    logs.push(msg.clone());
                    tracing::error!("{}", msg);
                }
            }

            match casks_result {
                Ok(packages) => {
                    let msg = format!("Found {} casks matching '{}'", packages.len(), query_clone);
                    logs.push(msg.clone());
                    tracing::info!("{}", msg);
                    results.extend(packages);
                }
                Err(e) => {
                    let msg = format!("Error searching casks: {}", e);
                    logs.push(msg.clone());
                    tracing::error!("{}", msg);
                }
            }

            if let Ok(mut results_guard) = search_results.lock() {
                *results_guard = results;
            }
            if let Ok(mut logs_guard) = output_log.lock() {
                *logs_guard = logs;
            }
        });
    }

    fn load_package_info(&mut self, package_name: String, package_type: PackageType) {
        if self.task_manager.can_load_more_package_info() {
            self.load_package_info_immediate(package_name, package_type);
        } else {
            self.task_manager
                .queue_package_info_load(package_name, package_type);
        }
    }

    fn load_package_info_immediate(&mut self, package_name: String, package_type: PackageType) {
        if self.task_manager.is_loading_package_info(&package_name) {
            tracing::debug!("Already loading info for {}, skipping", package_name);
            return;
        }

        tracing::info!(
            "Starting to load package info for {} ({:?})",
            package_name,
            package_type
        );

        let use_case = Arc::clone(&self.use_cases.get_package_info);
        let result = Arc::new(Mutex::new(None));
        let name_clone = package_name.clone();
        let package_type_clone = package_type.clone();
        let package_type_clone2 = package_type.clone();

        let task = AsyncTask::LoadPackageInfo {
            package_name: package_name.clone(),
            package_type: package_type.clone(),
            result: Arc::clone(&result),
            started_at: std::time::Instant::now(),
        };

        self.task_manager
            .add_package_info_task(package_name.clone(), task);

        self.executor.spawn(async move {
            tracing::debug!("Started task for loading {}", name_clone);

            let info_result = use_case.execute(&name_clone, package_type_clone).await;

            match info_result {
                Ok(package) => {
                    tracing::info!(
                        "Successfully loaded package info for {}: version={:?}",
                        name_clone,
                        package.version
                    );
                    if let Ok(mut result_guard) = result.lock() {
                        *result_guard = Some(package);
                    }
                }
                Err(e) => {
                    tracing::error!("Error loading package info for {}: {}", name_clone, e);
                    let failed_package = Package::new(name_clone.clone(), package_type_clone2)
                        .set_version_load_failed(true);
                    if let Ok(mut result_guard) = result.lock() {
                        *result_guard = Some(failed_package);
                    }
                }
            }
        });
    }

    fn poll_async_tasks(&mut self) {
        tracing::trace!("poll_async_tasks called, checking for active task");
        let result = self.task_manager.poll();

        if let Some(packages) = result.installed_packages {
            tracing::info!("Got {} installed packages from poll", packages.len());
            self.merged_packages.update_packages(packages);
            self.loading_installed = false;
        }

        if let Some(packages) = result.outdated_packages {
            tracing::info!("Got {} outdated packages from poll", packages.len());
            self.merged_packages.update_outdated_packages(packages);
            self.loading_outdated = false;
        }

        if self.loading_installed == false && self.loading_outdated == false {
            self.tab_manager.mark_loaded(Tab::Installed);
            self.status_message = "Packages loaded".to_string();
        }

        if let Some(packages) = result.search_results {
            self.search_results.update_packages(packages.clone());
            self.loading_search = false;
            self.status_message = "Search completed".to_string();

            if self.auto_load_version_info {
                tracing::info!("Auto-loading version info for {} packages", packages.len());
                for package in packages.iter() {
                    if package.version.is_none() && !package.version_load_failed {
                        tracing::debug!("Auto-loading info for {}", package.name);
                        self.load_package_info(package.name.clone(), package.package_type.clone());
                    }
                }
            }
        }

        if let Some((_name, package)) = result.package_info {
            self.search_results.update_package(package.clone());
            self.merged_packages.update_package(package);
        }

        if let Some((success, message)) = result.install_completed {
            self.loading_install = false;
            self.loading = false;
            let installed_pkg_name = self.current_install_package.clone();
            if let Some(pkg) = &installed_pkg_name {
                self.packages_in_operation.remove(pkg);
            }
            self.status_message = message.clone();

            if success {
                if let Some(pkg_name) = installed_pkg_name {
                    if let Some(mut pkg) = self.search_results.get_package(&pkg_name) {
                        pkg.installed = true;
                        self.search_results.update_package(pkg);
                    }
                    // Locally update installed packages instead of reloading
                    self.merged_packages.mark_package_updated(&pkg_name);
                    self.merged_packages
                        .remove_from_outdated_selection_by_name(&pkg_name);
                }
                self.current_install_package = None;
            } else {
                // Check if this is a password error
                if self.is_password_error(&message) {
                    if let Some(pkg_name) = &installed_pkg_name {
                        // Try to get the package from search results to retry with password
                        if let Some(pkg) = self.search_results.get_package(pkg_name) {
                            self.pending_operation = Some(PendingOperation::Install(pkg));
                            self.password_modal.show(format!("Install {}", pkg_name));
                        }
                    }
                } else {
                    self.current_install_package = None;
                }
            }
        }

        if let Some((success, message)) = result.uninstall_completed {
            self.loading_uninstall = false;
            self.loading = false;
            let uninstall_pkg_name = self.current_uninstall_package.clone();
            if let Some(pkg) = &uninstall_pkg_name {
                self.packages_in_operation.remove(pkg);
            }
            self.status_message = message.clone();

            if success {
                if let Some(pkg) = self.current_uninstall_package.as_ref() {
                    // Locally remove uninstalled package instead of reloading
                    self.merged_packages.remove_installed_package(pkg);
                }
                self.current_uninstall_package = None;
            } else {
                // Check if this is a password error
                if self.is_password_error(&message) {
                    if let Some(pkg_name) = &uninstall_pkg_name {
                        // Try to get the package from merged packages to retry with password
                        if let Some(pkg) = self.merged_packages.get_package(pkg_name) {
                            self.pending_operation = Some(PendingOperation::Uninstall(pkg));
                            self.password_modal.show(format!("Uninstall {}", pkg_name));
                        }
                    }
                } else {
                    self.current_uninstall_package = None;
                }
            }
        }

        if let Some((success, message)) = result.update_completed {
            self.loading_update = false;
            self.loading = false;
            let pkg = self.current_update_package.take();
            if let Some(ref pkg_name) = pkg {
                self.packages_in_operation.remove(pkg_name);
            }
            self.status_message = message;

            if success {
                if let Some(pkg_name) = pkg {
                    // Locally move package from outdated to installed instead of reloading
                    self.merged_packages.mark_package_updated(&pkg_name);
                    self.merged_packages
                        .remove_from_outdated_selection_by_name(&pkg_name);
                }
            }

            // If we're in the middle of updating selected packages, process the next one
            if self.loading_update_all && !self.pending_updates.is_empty() {
                self.process_next_pending_update();
                self.loading_update = true;
            } else if self.loading_update_all && self.pending_updates.is_empty() {
                // All updates are done
                self.loading_update_all = false;
                self.status_message = "Finished updating all packages".to_string();
                self.log_manager
                    .push("Finished updating all packages".to_string());
                tracing::info!("Finished updating all packages");
                self.merged_packages.clear_outdated_selection();
            }
        }

        if let Some((success, message)) = result.update_all_completed {
            self.loading_update_all = false;
            self.loading = false;
            self.status_message = message;

            if success {
                // Locally update all packages that were in operation instead of reloading
                for pkg_name in self.packages_in_operation.iter() {
                    self.merged_packages.mark_package_updated(pkg_name);
                    self.merged_packages
                        .remove_from_outdated_selection_by_name(pkg_name);
                }
                self.packages_in_operation.clear();
            }

            self.merged_packages.clear_outdated_selection();
        }

        if let Some((_success, message)) = result.clean_cache_completed {
            self.loading_clean_cache = false;
            self.loading = false;
            self.status_message = message;
            self.cleanup_modal.close();
        }

        if let Some((_success, message)) = result.cleanup_old_versions_completed {
            self.loading_cleanup_old_versions = false;
            self.loading = false;
            self.status_message = message;
            self.cleanup_modal.close();
        }

        if let Some((package_name, _success, message)) = result.pin_completed {
            self.packages_in_operation.remove(&package_name);
            self.status_message = message;
            self.load_installed_packages(true);
        }

        if let Some((package_name, _success, message)) = result.unpin_completed {
            self.packages_in_operation.remove(&package_name);
            self.status_message = message;
            self.load_installed_packages(true);
        }

        if let Some(services) = result.services {
            tracing::info!("Got {} services from poll", services.len());
            self.service_list.update_services(services);
            self.loading_services = false;
            self.tab_manager.mark_loaded(Tab::Services);
            self.status_message = "Services loaded".to_string();
        }

        if let Some((service_name, success, message)) = result.start_service_completed {
            self.services_in_operation.remove(&service_name);
            self.status_message = message;
            if success {
                self.load_services();
            }
        }

        if let Some((service_name, success, message)) = result.stop_service_completed {
            self.services_in_operation.remove(&service_name);
            self.status_message = message;
            if success {
                self.load_services();
            }
        }

        if let Some((service_name, success, message)) = result.restart_service_completed {
            self.services_in_operation.remove(&service_name);
            self.status_message = message;
            if success {
                self.load_services();
            }
        }

        if let Some((_success, message)) = result.export_packages_completed {
            self.loading_export = false;
            self.loading = false;
            self.status_message = message;
        }

        if let Some((success, message)) = result.import_packages_completed {
            self.loading_import = false;
            self.loading = false;
            self.status_message = message;
            if success {
                // Reload installed packages after successful import
                self.load_installed_packages(true);
            }
        }

        self.log_manager.extend(result.logs);

        if self.task_manager.can_load_more_package_info()
            && self.task_manager.pending_loads_count() > 0
        {
            let to_load = 15 - self.task_manager.pending_loads_count();
            let batch = self.task_manager.drain_pending_loads(to_load);

            if !batch.is_empty() {
                tracing::info!(
                    "Starting batch load of {} packages ({} remaining in queue)",
                    batch.len(),
                    self.task_manager.pending_loads_count()
                );

                for (name, pkg_type) in batch {
                    self.load_package_info_immediate(name, pkg_type);
                }
            }
        }
    }

    fn poll_logs(&mut self) {
        while let Ok(log_entry) = self.log_rx.try_recv() {
            self.log_manager.push(log_entry);
        }
    }
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

impl eframe::App for BrewstyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_logs();
        self.poll_async_tasks();
        ctx.request_repaint();

        if !self.initialized {
            self.initialized = true;
            // Only load installed packages if auto-update is enabled
            self.load_installed_packages(self.config.auto_update_check);

            // Apply initial theme
            self.apply_theme(ctx);
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.heading("🍺 Brewsty");
                ui.label(format!("v{}", env!("CARGO_PKG_VERSION")));
                ui.separator();

                if ui
                    .selectable_label(
                        self.tab_manager.is_current(Tab::Installed),
                        "Installed & Outdated",
                    )
                    .clicked()
                {
                    self.tab_manager.switch_to(Tab::Installed);
                    if !self.tab_manager.is_loaded(Tab::Installed) {
                        self.load_installed_packages(true);
                    }
                }
                if ui
                    .selectable_label(
                        self.tab_manager.is_current(Tab::SearchInstall),
                        "Search & Install",
                    )
                    .clicked()
                {
                    self.tab_manager.switch_to(Tab::SearchInstall);
                }
                if ui
                    .selectable_label(self.tab_manager.is_current(Tab::Services), "Services")
                    .clicked()
                {
                    self.tab_manager.switch_to(Tab::Services);
                    if !self.tab_manager.is_loaded(Tab::Services) {
                        self.load_services();
                    }
                }
                if ui
                    .selectable_label(self.tab_manager.is_current(Tab::Settings), "Settings")
                    .clicked()
                {
                    self.tab_manager.switch_to(Tab::Settings);
                }
                if ui
                    .selectable_label(self.tab_manager.is_current(Tab::Log), "Log")
                    .clicked()
                {
                    self.tab_manager.switch_to(Tab::Log);
                }
            });
            ui.add_space(8.0);
        });

        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(true)
            .default_height(self.output_panel_height)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Clear Output").clicked() {
                        self.log_manager = LogManager::new();
                    }
                    ui.separator();
                    if ui.button("📋 Copy Output").clicked() {
                        let output = self
                            .log_manager
                            .all_logs()
                            .map(|entry| {
                                format!("[{}] {}", entry.format_timestamp(), entry.message)
                            })
                            .collect::<Vec<_>>()
                            .join("\n");
                        ctx.copy_text(output);
                    }
                });

                ui.separator();

                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());

                        for entry in self.log_manager.filtered_logs() {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(format!("[{}]", entry.format_timestamp()))
                                        .color(egui::Color32::GRAY)
                                        .monospace(),
                                );
                                ui.monospace(&entry.message);
                            });
                        }
                    });

                self.output_panel_height = ui.min_rect().height();
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.tab_manager.current() {
                Tab::Installed => {
                    let actions = InstalledTab::show(
                        ui,
                        &mut self.merged_packages,
                        &mut self.filter_state,
                        &self.packages_in_operation,
                        self.loading_installed,
                        self.loading_outdated,
                        &mut self.info_modal,
                    );

                    for action in actions {
                        match action {
                            InstalledAction::Refresh => self.load_installed_packages(true),
                            InstalledAction::Install(pkg) => self.handle_install(pkg),
                            InstalledAction::Uninstall(pkg) => self.handle_uninstall(pkg),
                            InstalledAction::Update(pkg) => self.handle_update(pkg),
                            InstalledAction::UpdateSelected(pkgs) => {
                                self.handle_update_selected(pkgs)
                            }
                            InstalledAction::Pin(pkg) => self.handle_pin(pkg),
                            InstalledAction::Unpin(pkg) => self.handle_unpin(pkg),
                            InstalledAction::LoadInfo(name, pkg_type) => {
                                self.load_package_info(name, pkg_type)
                            }
                        }
                    }
                }

                Tab::SearchInstall => {
                    let actions = SearchTab::show(
                        ui,
                        &mut self.search_results,
                        &mut self.filter_state,
                        &self.packages_in_operation,
                        self.loading_search,
                        &mut self.auto_load_version_info,
                        &mut self.info_modal,
                    );

                    for action in actions {
                        match action {
                            SearchAction::Search => self.handle_search(),
                            SearchAction::Install(pkg) => self.handle_install(pkg),
                            SearchAction::Uninstall(pkg) => self.handle_uninstall(pkg),
                            SearchAction::Update(pkg) => self.handle_update(pkg),
                            SearchAction::LoadInfo(name, pkg_type) => {
                                self.load_package_info(name, pkg_type)
                            }
                            SearchAction::Pin(pkg) => self.handle_pin(pkg),
                            SearchAction::Unpin(pkg) => self.handle_unpin(pkg),
                        }
                    }
                }

                Tab::Services => {
                    let actions = ServicesTab::show(
                        ui,
                        &mut self.service_list,
                        &self.services_in_operation,
                        self.loading_services,
                    );

                    for action in actions {
                        match action {
                            ServiceAction::Refresh => self.load_services(),
                            ServiceAction::Start(name) => self.handle_start_service(name),
                            ServiceAction::Stop(name) => self.handle_stop_service(name),
                            ServiceAction::Restart(name) => self.handle_restart_service(name),
                        }
                    }
                }

                Tab::Settings => {
                    tracing::trace!("Rendering Settings Tab");
                    let actions = SettingsTab::show(
                        ui,
                        &mut self.config,
                        &mut self.log_manager,
                        self.loading_export,
                        self.loading_import,
                    );

                    for action in actions {
                        match action {
                            SettingsAction::SaveConfig => self.save_config(),
                            SettingsAction::ApplyTheme => self.apply_theme(ctx),
                            SettingsAction::ShowCleanupPreview(cleanup_type) => {
                                self.show_cleanup_preview(cleanup_type)
                            }
                            SettingsAction::UpdateAll => self.handle_update_all(),
                            SettingsAction::ExportPackages => self.handle_export_packages(),
                            SettingsAction::ImportPackages => self.handle_import_packages(),
                        }
                    }
                }

                Tab::Log => {
                    let actions = LogTab::show(ui, &self.log_manager);
                    for action in actions {
                        match action {
                            LogAction::CopyAll => {
                                let output = self
                                    .log_manager
                                    .all_logs()
                                    .map(|entry| {
                                        format!("[{}] {}", entry.format_timestamp(), entry.message)
                                    })
                                    .collect::<Vec<_>>()
                                    .join("\n");
                                ctx.copy_text(output);
                            }
                            LogAction::Clear => self.log_manager = LogManager::new(),
                        }
                    }
                }
            }

            if let Some(action) = self.cleanup_modal.render(ctx) {
                match action {
                    CleanupAction::Confirm(cleanup_type) => match cleanup_type {
                        CleanupType::Cache => self.handle_clean_cache(),
                        CleanupType::OldVersions => self.handle_cleanup_old_versions(),
                    },
                    CleanupAction::Cancel => {
                        self.cleanup_modal.close();
                    }
                }
            }

            self.info_modal.render(ctx);

            self.password_modal.render(ctx);
            if let Some((confirmed, password)) = self.password_modal.take_result() {
                if confirmed && !password.is_empty() {
                    self.retry_with_password(&password);
                } else {
                    self.pending_operation = None;
                    self.log_manager
                        .push("Password entry cancelled.".to_string());
                    tracing::info!("Password entry cancelled");
                }
            }
        });
    }
}
