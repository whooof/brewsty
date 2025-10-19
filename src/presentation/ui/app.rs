use crate::application::UseCaseContainer;
use crate::domain::entities::{Package, PackageType};
use crate::presentation::components::{
    CleanupAction, CleanupModal, CleanupType, FilterState, InfoModal, LogManager, PackageList, Tab,
    TabManager,
};
use crate::presentation::services::{AsyncExecutor, AsyncTask, AsyncTaskManager};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct BrewstyApp {
    tab_manager: TabManager,
    filter_state: FilterState,
    cleanup_modal: CleanupModal,
    info_modal: InfoModal,
    log_manager: LogManager,
    log_rx: Receiver<String>,

    installed_packages: PackageList,
    outdated_packages: PackageList,
    search_results: PackageList,

    auto_load_version_info: bool,

    initialized: bool,

    loading_installed: bool,
    loading_outdated: bool,
    loading_search: bool,

    loading_install: bool,
    loading_uninstall: bool,
    loading_update: bool,
    loading_update_all: bool,
    loading_clean_cache: bool,
    loading_cleanup_old_versions: bool,

    current_install_package: Option<String>,
    current_uninstall_package: Option<String>,
    current_update_package: Option<String>,
    packages_in_operation: std::collections::HashSet<String>,

    task_manager: AsyncTaskManager,

    use_cases: Arc<UseCaseContainer>,
    executor: AsyncExecutor,

    loading: bool,
    status_message: String,
    output_panel_height: f32,
}

impl BrewstyApp {
    pub fn new(use_cases: Arc<UseCaseContainer>, log_rx: Receiver<String>) -> Self {
        let executor = AsyncExecutor::new();

        Self {
            tab_manager: TabManager::new(),
            filter_state: FilterState::new(),
            cleanup_modal: CleanupModal::new(),
            info_modal: InfoModal::new(),
            log_manager: LogManager::new(),
            log_rx,
            installed_packages: PackageList::new(),
            outdated_packages: PackageList::new(),
            search_results: PackageList::new(),
            auto_load_version_info: false,
            initialized: false,
            loading_installed: false,
            loading_outdated: false,
            loading_search: false,
            loading_install: false,
            loading_uninstall: false,
            loading_update: false,
            loading_update_all: false,
            loading_clean_cache: false,
            loading_cleanup_old_versions: false,
            current_install_package: None,
            current_uninstall_package: None,
            current_update_package: None,
            packages_in_operation: std::collections::HashSet::new(),
            task_manager: AsyncTaskManager::new(),
            use_cases,
            executor,
            loading: false,
            status_message: String::new(),
            output_panel_height: 150.0,
        }
    }

    fn load_installed_packages(&mut self) {
        if self.loading_installed {
            return;
        }

        self.loading_installed = true;
        self.status_message = "Loading installed packages...".to_string();
        self.log_manager
            .push("Loading installed packages (formulae and casks)".to_string());
        tracing::info!("Loading installed packages (formulae and casks)");

        let use_case_formulae = Arc::clone(&self.use_cases.list_installed);
        let use_case_casks = Arc::clone(&self.use_cases.list_installed);

        let installed_packages = Arc::new(Mutex::new(Vec::new()));
        let output_log = Arc::new(Mutex::new(Vec::new()));

        self.task_manager.set_active_task(AsyncTask::LoadInstalled {
            packages: Arc::clone(&installed_packages),
            logs: Arc::clone(&output_log),
        });

        thread::spawn(move || {
            tracing::trace!("THREAD STARTED: load_installed_packages");
            if let Err(e) = (|| {
                tracing::trace!("THREAD: about to create runtime");
                let rt = tokio::runtime::Runtime::new()?;
                tracing::trace!("THREAD: runtime created");

                tracing::debug!("Starting to load installed packages");

                tracing::trace!("THREAD: about to execute formulae");
                let formulae_result =
                    rt.block_on(async { use_case_formulae.execute(PackageType::Formula).await });

                tracing::debug!("Formulae result: {:?}", formulae_result.as_ref().map(|p| p.len()).map_err(|e| e.to_string()));

                tracing::trace!("THREAD: about to execute casks");
                let casks_result =
                    rt.block_on(async { use_case_casks.execute(PackageType::Cask).await });

                tracing::debug!("Casks result: {:?}", casks_result.as_ref().map(|p| p.len()).map_err(|e| e.to_string()));

                let mut packages = Vec::new();
                let mut logs = Vec::new();

                match formulae_result {
                    Ok(pkgs) => {
                        let msg = format!("Loaded {} formulae", pkgs.len());
                        logs.push(msg.clone());
                        tracing::info!("{}", msg);
                        packages.extend(pkgs);
                    }
                    Err(e) => {
                        let msg = format!("Error loading formulae: {}", e);
                        logs.push(msg.clone());
                        tracing::error!("{}", msg);
                    }
                }

                match casks_result {
                    Ok(pkgs) => {
                        let msg = format!("Loaded {} casks", pkgs.len());
                        logs.push(msg.clone());
                        tracing::info!("{}", msg);
                        packages.extend(pkgs);
                    }
                    Err(e) => {
                        let msg = format!("Error loading casks: {}", e);
                        logs.push(msg.clone());
                        tracing::error!("{}", msg);
                    }
                }

                tracing::debug!("About to write {} packages to mutex", packages.len());
                tracing::debug!("About to lock packages mutex");
                *installed_packages.lock().map_err(|e| anyhow::anyhow!("Failed to lock packages: {}", e))? = packages;
                tracing::debug!("Successfully locked packages, now adding finish log");
                
                logs.push("Finished loading installed packages".to_string());
                tracing::info!("Finished loading installed packages");
                
                tracing::debug!("About to lock logs mutex with {} log entries", logs.len());
                *output_log.lock().map_err(|e| anyhow::anyhow!("Failed to lock logs: {}", e))? = logs;
                tracing::debug!("Successfully updated mutexes");

                anyhow::Ok(())
            })() {
                tracing::error!("Error in load_installed_packages thread: {}", e);
                if let Ok(mut logs) = output_log.lock() {
                    logs.push(format!("Thread error: {}", e));
                }
            }
            tracing::trace!("THREAD ENDED: load_installed_packages");
        });
    }

    fn load_outdated_packages(&mut self) {
        if self.loading_outdated {
            return;
        }

        self.loading_outdated = true;
        self.status_message = "Loading outdated packages...".to_string();
        self.log_manager
            .push("Loading outdated packages (formulae and casks)".to_string());
        tracing::info!("Loading outdated packages (formulae and casks)");

        let use_case_formulae = Arc::clone(&self.use_cases.list_outdated);
        let use_case_casks = Arc::clone(&self.use_cases.list_outdated);

        let outdated_packages = Arc::new(Mutex::new(Vec::new()));
        let output_log = Arc::new(Mutex::new(Vec::new()));

        self.task_manager.set_active_task(AsyncTask::LoadOutdated {
            packages: Arc::clone(&outdated_packages),
            logs: Arc::clone(&output_log),
        });

        thread::spawn(move || {
            if let Err(e) = (|| {
                let rt = tokio::runtime::Runtime::new()?;

                tracing::debug!("Starting to load outdated packages");

                let formulae_result =
                    rt.block_on(async { use_case_formulae.execute(PackageType::Formula).await });

                tracing::debug!("Outdated formulae result: {:?}", formulae_result.as_ref().map(|p| p.len()).map_err(|e| e.to_string()));

                let casks_result =
                    rt.block_on(async { use_case_casks.execute(PackageType::Cask).await });

                tracing::debug!("Outdated casks result: {:?}", casks_result.as_ref().map(|p| p.len()).map_err(|e| e.to_string()));

                let mut packages = Vec::new();
                let mut logs = Vec::new();

                match formulae_result {
                    Ok(pkgs) => {
                        let msg = format!("Loaded {} outdated formulae", pkgs.len());
                        logs.push(msg.clone());
                        tracing::info!("{}", msg);
                        packages.extend(pkgs);
                    }
                    Err(e) => {
                        let msg = format!("Error loading outdated formulae: {}", e);
                        logs.push(msg.clone());
                        tracing::error!("{}", msg);
                    }
                }

                match casks_result {
                    Ok(pkgs) => {
                        let msg = format!("Loaded {} outdated casks", pkgs.len());
                        logs.push(msg.clone());
                        tracing::info!("{}", msg);
                        packages.extend(pkgs);
                    }
                    Err(e) => {
                        let msg = format!("Error loading outdated casks: {}", e);
                        logs.push(msg.clone());
                        tracing::error!("{}", msg);
                    }
                }

                tracing::debug!("About to write {} outdated packages to mutex", packages.len());
                tracing::debug!("About to lock outdated packages mutex");
                *outdated_packages.lock().map_err(|e| anyhow::anyhow!("Failed to lock packages: {}", e))? = packages;
                tracing::debug!("Successfully locked packages, now adding finish log");
                
                logs.push("Finished loading outdated packages".to_string());
                tracing::info!("Finished loading outdated packages");

                tracing::debug!("About to lock outdated logs mutex with {} log entries", logs.len());
                *output_log.lock().map_err(|e| anyhow::anyhow!("Failed to lock logs: {}", e))? = logs;
                tracing::debug!("Successfully updated outdated mutexes");

                anyhow::Ok(())
            })() {
                tracing::error!("Error in load_outdated_packages thread: {}", e);
                if let Ok(mut logs) = output_log.lock() {
                    logs.push(format!("Thread error: {}", e));
                }
            }
        });
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
        let executor = self.executor.clone();

        thread::spawn(move || {
            let result = executor.execute(async move { use_case.execute(package).await });

            let mut log_vec = Vec::new();
            match result {
                Ok(_) => {
                    let msg = format!("Successfully installed {}", package_name);
                    log_vec.push(msg.clone());
                    tracing::info!("{}", msg);
                    *success.lock().unwrap() = Some(true);
                    *message.lock().unwrap() = format!("{} installed successfully", package_name);
                }
                Err(e) => {
                    let msg = format!("Error installing {}: {}", package_name, e);
                    log_vec.push(msg.clone());
                    tracing::error!("{}", msg);
                    *success.lock().unwrap() = Some(false);
                    *message.lock().unwrap() = msg;
                }
            }

            *logs.lock().unwrap() = log_vec;
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
        let executor = self.executor.clone();

        thread::spawn(move || {
            let result = executor.execute(async move { use_case.execute(package).await });

            let mut log_vec = Vec::new();
            match result {
                Ok(_) => {
                    let msg = format!("Successfully uninstalled {}", package_name);
                    log_vec.push(msg.clone());
                    tracing::info!("{}", msg);
                    *success.lock().unwrap() = Some(true);
                    *message.lock().unwrap() = format!("{} uninstalled successfully", package_name);
                }
                Err(e) => {
                    let msg = format!("Error uninstalling {}: {}", package_name, e);
                    log_vec.push(msg.clone());
                    tracing::error!("{}", msg);
                    *success.lock().unwrap() = Some(false);
                    *message.lock().unwrap() = msg;
                }
            }

            *logs.lock().unwrap() = log_vec;
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
        let executor = self.executor.clone();

        thread::spawn(move || {
            let result = executor.execute(async move { use_case.execute(package).await });

            let mut log_vec = Vec::new();
            match result {
                Ok(_) => {
                    let msg = format!("Successfully updated {}", package_name);
                    log_vec.push(msg.clone());
                    tracing::info!("{}", msg);
                    *success.lock().unwrap() = Some(true);
                    *message.lock().unwrap() = format!("{} updated successfully", package_name);
                }
                Err(e) => {
                    let msg = format!("Error updating {}: {}", package_name, e);
                    log_vec.push(msg.clone());
                    tracing::error!("{}", msg);
                    *success.lock().unwrap() = Some(false);
                    *message.lock().unwrap() = msg;
                }
            }

            *logs.lock().unwrap() = log_vec;
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

        self.executor.execute(async move {
            match use_case.execute(package_clone).await {
                Ok(_) => {
                    let msg = format!("Successfully pinned {}", package_name);
                    *logs.lock().unwrap() = vec![msg.clone()];
                    *success.lock().unwrap() = Some(true);
                    *message.lock().unwrap() = format!("{} pinned successfully", package_name);
                }
                Err(e) => {
                    let msg = format!("Error pinning {}: {}", package_name, e);
                    *logs.lock().unwrap() = vec![msg.clone()];
                    *success.lock().unwrap() = Some(false);
                    *message.lock().unwrap() = msg;
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

        self.executor.execute(async move {
            match use_case.execute(package_clone).await {
                Ok(_) => {
                    let msg = format!("Successfully unpinned {}", package_name);
                    *logs.lock().unwrap() = vec![msg.clone()];
                    *success.lock().unwrap() = Some(true);
                    *message.lock().unwrap() = format!("{} unpinned successfully", package_name);
                }
                Err(e) => {
                    let msg = format!("Error unpinning {}: {}", package_name, e);
                    *logs.lock().unwrap() = vec![msg.clone()];
                    *success.lock().unwrap() = Some(false);
                    *message.lock().unwrap() = msg;
                }
            }
        });
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
        let executor = self.executor.clone();

        thread::spawn(move || {
            let result = executor.execute(async move { use_case.execute().await });

            let mut log_vec = Vec::new();
            match result {
                Ok(_) => {
                    let msg = "Successfully updated all packages".to_string();
                    log_vec.push(msg.clone());
                    tracing::info!("{}", msg);
                    *success.lock().unwrap() = Some(true);
                    *message.lock().unwrap() = "All packages updated successfully".to_string();
                }
                Err(e) => {
                    let msg = format!("Error updating all packages: {}", e);
                    log_vec.push(msg.clone());
                    tracing::error!("{}", msg);
                    *success.lock().unwrap() = Some(false);
                    *message.lock().unwrap() = msg;
                }
            }

            *logs.lock().unwrap() = log_vec;
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
        let executor = self.executor.clone();

        thread::spawn(move || {
            let result = executor.execute(async move { use_case.execute().await });

            let mut log_vec = Vec::new();
            match result {
                Ok(_) => {
                    let msg = "Successfully cleaned cache".to_string();
                    log_vec.push(msg.clone());
                    tracing::info!("{}", msg);
                    *success.lock().unwrap() = Some(true);
                    *message.lock().unwrap() = "Cache cleaned successfully".to_string();
                }
                Err(e) => {
                    let msg = format!("Error cleaning cache: {}", e);
                    log_vec.push(msg.clone());
                    tracing::error!("{}", msg);
                    *success.lock().unwrap() = Some(false);
                    *message.lock().unwrap() = msg;
                }
            }

            *logs.lock().unwrap() = log_vec;
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
        let executor = self.executor.clone();

        thread::spawn(move || {
            let result = executor.execute(async move { use_case.execute().await });

            let mut log_vec = Vec::new();
            match result {
                Ok(_) => {
                    let msg = "Successfully cleaned up old versions".to_string();
                    log_vec.push(msg.clone());
                    tracing::info!("{}", msg);
                    *success.lock().unwrap() = Some(true);
                    *message.lock().unwrap() = "Old versions cleaned up successfully".to_string();
                }
                Err(e) => {
                    let msg = format!("Error cleaning up old versions: {}", e);
                    log_vec.push(msg.clone());
                    tracing::error!("{}", msg);
                    *success.lock().unwrap() = Some(false);
                    *message.lock().unwrap() = msg;
                }
            }

            *logs.lock().unwrap() = log_vec;
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

        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();

            let formulae_result = rt.block_on(async {
                use_case_formulae
                    .execute(&query, PackageType::Formula)
                    .await
            });

            let casks_result = rt.block_on(async {
                use_case_casks
                    .execute(&query_clone, PackageType::Cask)
                    .await
            });

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

            *search_results.lock().unwrap() = results;
            *output_log.lock().unwrap() = logs;
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

        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();

            tracing::debug!("Spawned thread for loading {}", name_clone);

            let info_result =
                rt.block_on(async { use_case.execute(&name_clone, package_type_clone).await });

            match info_result {
                Ok(package) => {
                    tracing::info!(
                        "Successfully loaded package info for {}: version={:?}",
                        name_clone,
                        package.version
                    );
                    *result.lock().unwrap() = Some(package);
                }
                Err(e) => {
                    tracing::error!("Error loading package info for {}: {}", name_clone, e);
                    let failed_package = Package::new(name_clone.clone(), package_type_clone2)
                        .set_version_load_failed(true);
                    *result.lock().unwrap() = Some(failed_package);
                }
            }
        });
    }

    fn poll_async_tasks(&mut self) {
        tracing::trace!("poll_async_tasks called, checking for active task");
        let result = self.task_manager.poll();

        if let Some(packages) = result.installed_packages {
            tracing::info!("Got {} installed packages from poll", packages.len());
            self.installed_packages.update_packages(packages);
            self.tab_manager.mark_loaded(Tab::Installed);
            self.loading_installed = false;
            self.status_message = "Packages loaded".to_string();
        }

        if let Some(packages) = result.outdated_packages {
            tracing::info!("Got {} outdated packages from poll", packages.len());
            self.outdated_packages.update_packages(packages);
            self.tab_manager.mark_loaded(Tab::Outdated);
            self.loading_outdated = false;
            self.status_message = "Outdated packages loaded".to_string();
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
            self.search_results.update_package(package);
        }

        if let Some((success, message)) = result.install_completed {
            self.loading_install = false;
            self.loading = false;
            let installed_pkg_name = self.current_install_package.take();
            if let Some(pkg) = &installed_pkg_name {
                self.packages_in_operation.remove(pkg);
            }
            self.status_message = message;

            if success {
                self.tab_manager.mark_unloaded(Tab::Installed);
                self.load_installed_packages();

                if let Some(pkg_name) = installed_pkg_name {
                    if let Some(mut pkg) = self.search_results.get_package(&pkg_name) {
                        pkg.installed = true;
                        self.search_results.update_package(pkg);
                    }
                }
            }
        }

        if let Some((success, message)) = result.uninstall_completed {
            self.loading_uninstall = false;
            self.loading = false;
            if let Some(pkg) = self.current_uninstall_package.take() {
                self.packages_in_operation.remove(&pkg);
            }
            self.status_message = message;

            if success {
                self.tab_manager.mark_unloaded(Tab::Installed);
                self.load_installed_packages();
            }
        }

        if let Some((success, message)) = result.update_completed {
            self.loading_update = false;
            self.loading = false;
            if let Some(pkg) = self.current_update_package.take() {
                self.packages_in_operation.remove(&pkg);
            }
            self.status_message = message;

            if success {
                self.tab_manager.mark_unloaded(Tab::Installed);
                self.tab_manager.mark_unloaded(Tab::Outdated);
                self.load_installed_packages();
                self.load_outdated_packages();
            }
        }

        if let Some((success, message)) = result.update_all_completed {
            self.loading_update_all = false;
            self.loading = false;
            self.status_message = message;

            if success {
                self.tab_manager.mark_unloaded(Tab::Installed);
                self.tab_manager.mark_unloaded(Tab::Outdated);
                self.load_installed_packages();
                self.load_outdated_packages();
            }
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
            self.load_installed_packages();
        }

        if let Some((package_name, _success, message)) = result.unpin_completed {
            self.packages_in_operation.remove(&package_name);
            self.status_message = message;
            self.load_installed_packages();
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

    fn show_loader(&self, ui: &mut egui::Ui, message: &str) {
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);
            ui.spinner();
            ui.label(message);
        });
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
            self.load_installed_packages();
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🍺 Brewsty");
                ui.label(format!("v{}", env!("CARGO_PKG_VERSION")));
                ui.separator();

                if ui
                    .selectable_label(self.tab_manager.is_current(Tab::Installed), "Installed")
                    .clicked()
                {
                    self.tab_manager.switch_to(Tab::Installed);
                    if !self.tab_manager.is_loaded(Tab::Installed) {
                        self.load_installed_packages();
                    }
                }
                if ui
                    .selectable_label(self.tab_manager.is_current(Tab::Outdated), "Outdated")
                    .clicked()
                {
                    self.tab_manager.switch_to(Tab::Outdated);
                    if !self.tab_manager.is_loaded(Tab::Outdated) {
                        self.load_outdated_packages();
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
                        let output = self.log_manager.all_logs()
                            .map(|entry| format!("[{}] {}", entry.format_timestamp(), entry.message))
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

                        for entry in self.log_manager.all_logs() {
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
                    ui.horizontal(|ui| {
                        ui.label("Search:");
                        ui.text_edit_singleline(self.filter_state.installed_search_query_mut());
                        ui.separator();
                        let mut show_formulae = self.filter_state.show_formulae();
                        let mut show_casks = self.filter_state.show_casks();
                        ui.checkbox(&mut show_formulae, "Show Formulae");
                        ui.checkbox(&mut show_casks, "Show Casks");
                        self.filter_state.set_show_formulae(show_formulae);
                        self.filter_state.set_show_casks(show_casks);
                        ui.separator();
                        if ui.button("Refresh").clicked() {
                            self.load_installed_packages();
                        }
                    });

                    ui.separator();

                    if self.loading_installed {
                        self.show_loader(ui, "Loading installed packages...");
                    } else {
                        let mut install_action = None;
                        let mut uninstall_action = None;
                        let mut update_action = None;
                        let mut pin_action = None;
                        let mut unpin_action = None;
                        let mut load_info_action = None;

                        self.installed_packages.show_filtered_with_search_and_pin(
                            ui,
                            &mut install_action,
                            &mut uninstall_action,
                            &mut update_action,
                            self.filter_state.show_formulae(),
                            self.filter_state.show_casks(),
                            self.filter_state.installed_search_query(),
                            &mut load_info_action,
                            &self.packages_in_operation,
                            &mut pin_action,
                            &mut unpin_action,
                        );

                        if let Some(package) = install_action {
                            self.handle_install(package);
                        }
                        if let Some(package) = uninstall_action {
                            self.handle_uninstall(package);
                        }
                        if let Some(package) = update_action {
                            self.handle_update(package);
                        }
                        if let Some(package) = pin_action {
                            self.handle_pin(package);
                        }
                        if let Some(package) = unpin_action {
                            self.handle_unpin(package);
                        }
                        if let Some(package) = self.installed_packages.get_show_info_action() {
                            self.info_modal.show(package);
                        }
                    }
                }

                Tab::Outdated => {
                    ui.horizontal(|ui| {
                        let mut show_formulae = self.filter_state.show_formulae();
                        let mut show_casks = self.filter_state.show_casks();
                        ui.checkbox(&mut show_formulae, "Show Formulae");
                        ui.checkbox(&mut show_casks, "Show Casks");
                        self.filter_state.set_show_formulae(show_formulae);
                        self.filter_state.set_show_casks(show_casks);
                        ui.separator();
                        if ui.button("Refresh").clicked() {
                            self.load_outdated_packages();
                        }
                        if ui.button("Update All").clicked() {
                            self.handle_update_all();
                        }
                    });

                    ui.separator();

                    if self.loading_outdated {
                        self.show_loader(ui, "Loading outdated packages...");
                    } else {
                        let mut install_action = None;
                        let mut uninstall_action = None;
                        let mut update_action = None;
                        let mut pin_action = None;
                        let mut unpin_action = None;
                        let mut load_info_action = None;

                        self.outdated_packages.show_filtered_with_search_and_pin(
                            ui,
                            &mut install_action,
                            &mut uninstall_action,
                            &mut update_action,
                            self.filter_state.show_formulae(),
                            self.filter_state.show_casks(),
                            "",
                            &mut load_info_action,
                            &self.packages_in_operation,
                            &mut pin_action,
                            &mut unpin_action,
                        );

                        if let Some(package) = install_action {
                            self.handle_install(package);
                        }
                        if let Some(package) = uninstall_action {
                            self.handle_uninstall(package);
                        }
                        if let Some(package) = update_action {
                            self.handle_update(package);
                        }
                        if let Some(package) = pin_action {
                            self.handle_pin(package);
                        }
                        if let Some(package) = unpin_action {
                            self.handle_unpin(package);
                        }
                        if let Some(package) = self.outdated_packages.get_show_info_action() {
                            self.info_modal.show(package);
                        }
                    }
                }

                Tab::SearchInstall => {
                    ui.horizontal(|ui| {
                        ui.label("Search:");
                        let response =
                            ui.text_edit_singleline(self.filter_state.search_query_mut());
                        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            self.handle_search();
                        }
                        if ui.button("Search").clicked() {
                            self.handle_search();
                        }
                    });

                    ui.horizontal(|ui| {
                        let mut show_formulae = self.filter_state.show_formulae();
                        let mut show_casks = self.filter_state.show_casks();
                        ui.checkbox(&mut show_formulae, "Show Formulae");
                        ui.checkbox(&mut show_casks, "Show Casks");
                        self.filter_state.set_show_formulae(show_formulae);
                        self.filter_state.set_show_casks(show_casks);
                        ui.separator();
                        ui.checkbox(&mut self.auto_load_version_info, "Auto-load version info");
                    });

                    ui.separator();

                    if self.loading_search {
                        self.show_loader(ui, "Searching...");
                    } else {
                        let mut install_action = None;
                        let mut uninstall_action = None;
                        let mut update_action = None;
                        let mut load_info_action = None;
                        let mut pin_action = None;
                        let mut unpin_action = None;

                        self.search_results.show_filtered_with_search_and_pin(
                            ui,
                            &mut install_action,
                            &mut uninstall_action,
                            &mut update_action,
                            self.filter_state.show_formulae(),
                            self.filter_state.show_casks(),
                            "",
                            &mut load_info_action,
                            &self.packages_in_operation,
                            &mut pin_action,
                            &mut unpin_action,
                        );

                        if let Some(package) = install_action {
                            self.handle_install(package);
                        }
                        if let Some(package) = uninstall_action {
                            self.handle_uninstall(package);
                        }
                        if let Some(package) = update_action {
                            self.handle_update(package);
                        }
                        if let Some(package) = load_info_action {
                            self.load_package_info(package.name.clone(), package.package_type);
                        }
                        if let Some(package) = pin_action {
                            self.handle_pin(package);
                        }
                        if let Some(package) = unpin_action {
                            self.handle_unpin(package);
                        }
                        if let Some(package) = self.search_results.get_show_info_action() {
                            self.info_modal.show(package);
                        }
                    }
                }

                Tab::Settings => {
                    ui.heading("Settings & Maintenance");
                    ui.separator();

                    ui.vertical_centered(|ui| {
                        if ui.button("Clean Cache").clicked() {
                            self.show_cleanup_preview(CleanupType::Cache);
                        }
                        ui.label("Remove old downloads from cache");

                        ui.add_space(10.0);

                        if ui.button("Cleanup Old Versions").clicked() {
                            self.show_cleanup_preview(CleanupType::OldVersions);
                        }
                        ui.label("Remove old versions of installed packages");

                        ui.add_space(10.0);

                        if ui.button("Update All Packages").clicked() {
                            self.handle_update_all();
                        }
                        ui.label("Update all installed packages");
                    });
                }

                Tab::Log => {
                    ui.heading("Command Log");
                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("📋 Copy All").clicked() {
                            let output = self.log_manager.all_logs()
                                .map(|entry| format!("[{}] {}", entry.format_timestamp(), entry.message))
                                .collect::<Vec<_>>()
                                .join("\n");
                            ctx.copy_text(output);
                        }
                        if ui.button("🗑 Clear").clicked() {
                            self.log_manager = LogManager::new();
                        }
                    });

                    ui.separator();

                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.set_style({
                                let mut style = (*ui.ctx().style()).clone();
                                style.override_font_id = Some(egui::FontId::monospace(12.0));
                                style
                            });

                            for entry in self.log_manager.all_logs() {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "[{}]",
                                            entry.format_timestamp()
                                        ))
                                        .color(egui::Color32::GRAY)
                                        .monospace(),
                                    );
                                    ui.monospace(&entry.message);
                                });
                            }
                        });
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
        });
    }
}
