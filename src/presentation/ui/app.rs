use crate::application::UseCaseContainer;
use crate::domain::entities::{Package, PackageType};
use crate::presentation::components::{
    CleanupAction, CleanupModal, CleanupType, FilterState, InfoModal, LogManager, PackageList, Tab, TabManager
};
use crate::presentation::services::{
    AsyncExecutor, AsyncTask, AsyncTaskManager, PackageOperationHandler
};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Receiver;
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
    
    task_manager: AsyncTaskManager,
    
    use_cases: Arc<UseCaseContainer>,
    executor: AsyncExecutor,
    package_ops: PackageOperationHandler,
    
    loading: bool,
    status_message: String,
    output_panel_height: f32,
}

impl BrewstyApp {
    pub fn new(use_cases: Arc<UseCaseContainer>, log_rx: Receiver<String>) -> Self {
        let executor = AsyncExecutor::new();
        
        let package_ops = PackageOperationHandler::new(
            Arc::clone(&use_cases.install),
            Arc::clone(&use_cases.uninstall),
            Arc::clone(&use_cases.update),
            Arc::clone(&use_cases.pin),
            Arc::clone(&use_cases.unpin),
            executor.clone(),
        );

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
            task_manager: AsyncTaskManager::new(),
            use_cases,
            executor,
            package_ops,
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
        self.log_manager.push("Loading installed packages (formulae and casks)".to_string());
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
            let rt = tokio::runtime::Runtime::new().unwrap();
            
            let formulae_result = rt.block_on(async {
                use_case_formulae.execute(PackageType::Formula).await
            });

            let casks_result = rt.block_on(async {
                use_case_casks.execute(PackageType::Cask).await
            });

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

            logs.push("Finished loading installed packages".to_string());
            tracing::info!("Finished loading installed packages");

            *installed_packages.lock().unwrap() = packages;
            *output_log.lock().unwrap() = logs;
        });
    }

    fn load_outdated_packages(&mut self) {
        if self.loading_outdated {
            return;
        }
        
        self.loading_outdated = true;
        self.status_message = "Loading outdated packages...".to_string();
        self.log_manager.push("Loading outdated packages (formulae and casks)".to_string());
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
            let rt = tokio::runtime::Runtime::new().unwrap();
            
            let formulae_result = rt.block_on(async {
                use_case_formulae.execute(PackageType::Formula).await
            });

            let casks_result = rt.block_on(async {
                use_case_casks.execute(PackageType::Cask).await
            });

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

            logs.push("Finished loading outdated packages".to_string());
            tracing::info!("Finished loading outdated packages");

            *outdated_packages.lock().unwrap() = packages;
            *output_log.lock().unwrap() = logs;
        });
    }

    fn handle_install(&mut self, package: Package) {
        if self.loading_install {
            return;
        }
        
        self.loading_install = true;
        self.loading = true;
        self.status_message = format!("Installing {}...", package.name);
        
        let package_name = package.name.clone();
        let package_type = package.package_type;
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
        
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            
            let result = rt.block_on(async move {
                use_case.execute(package).await
            });

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
        
        self.loading_uninstall = true;
        self.loading = true;
        self.status_message = format!("Uninstalling {}...", package.name);
        
        let package_name = package.name.clone();
        let package_type = package.package_type;
        let initial_msg = format!("Uninstalling package: {} ({:?})", package_name, package_type);
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
        
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            
            let result = rt.block_on(async move {
                use_case.execute(package).await
            });

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
        
        self.loading_update = true;
        self.loading = true;
        self.status_message = format!("Updating {}...", package.name);
        
        let package_name = package.name.clone();
        let package_type = package.package_type;
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
        
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            
            let result = rt.block_on(async move {
                use_case.execute(package).await
            });

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
        self.status_message = format!("Pinning {}...", package.name);
        
        let result = self.package_ops.pin(package);
        
        self.status_message = result.message;
        self.log_manager.extend(result.log_messages);
        self.loading = false;
        
        if result.success {
            // Mark tabs as unloaded so they reload when accessed
            self.tab_manager.mark_unloaded(Tab::Installed);
            self.tab_manager.mark_unloaded(Tab::Outdated);
            
            // Reload the current tab immediately
            match self.tab_manager.current() {
                Tab::Installed => self.load_installed_packages(),
                Tab::Outdated => self.load_outdated_packages(),
                _ => {}
            }
        }
    }

    fn handle_unpin(&mut self, package: Package) {
        self.loading = true;
        self.status_message = format!("Unpinning {}...", package.name);
        
        let result = self.package_ops.unpin(package);
        
        self.status_message = result.message;
        self.log_manager.extend(result.log_messages);
        self.loading = false;
        
        if result.success {
            // Mark tabs as unloaded so they reload when accessed
            self.tab_manager.mark_unloaded(Tab::Installed);
            self.tab_manager.mark_unloaded(Tab::Outdated);
            
            // Reload the current tab immediately
            match self.tab_manager.current() {
                Tab::Installed => self.load_installed_packages(),
                Tab::Outdated => self.load_outdated_packages(),
                _ => {}
            }
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
        
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            
            let result = rt.block_on(async move {
                use_case.execute().await
            });

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
                let msg = format!("Found {} items to clean ({})", 
                    preview.items.len(), 
                    format_size(preview.total_size));
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
        self.loading = true;
        self.status_message = "Cleaning cache...".to_string();
        self.log_manager.push("Cleaning Homebrew cache".to_string());
        tracing::info!("Cleaning Homebrew cache");

        let use_case = Arc::clone(&self.use_cases.clean_cache);
        let result = self.executor.execute(async {
            use_case.execute().await
        });

        match result {
            Ok(_) => {
                let msg = "Successfully cleaned cache".to_string();
                self.log_manager.push(msg.clone());
                tracing::info!("{}", msg);
                self.status_message = "Cache cleaned successfully".to_string();
            }
            Err(e) => {
                let msg = format!("Error cleaning cache: {}", e);
                self.log_manager.push(msg.clone());
                tracing::error!("{}", msg);
                self.status_message = msg;
            }
        }

        self.loading = false;
        self.cleanup_modal.close();
    }

    fn handle_cleanup_old_versions(&mut self) {
        self.loading = true;
        self.status_message = "Cleaning up old versions...".to_string();
        self.log_manager.push("Cleaning up old versions".to_string());
        tracing::info!("Cleaning up old versions");

        let use_case = Arc::clone(&self.use_cases.cleanup_old_versions);
        let result = self.executor.execute(async {
            use_case.execute().await
        });

        match result {
            Ok(_) => {
                let msg = "Successfully cleaned up old versions".to_string();
                self.log_manager.push(msg.clone());
                tracing::info!("{}", msg);
                self.status_message = "Old versions cleaned up successfully".to_string();
            }
            Err(e) => {
                let msg = format!("Error cleaning up old versions: {}", e);
                self.log_manager.push(msg.clone());
                tracing::error!("{}", msg);
                self.status_message = msg;
            }
        }

        self.loading = false;
        self.cleanup_modal.close();
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
                use_case_formulae.execute(&query, PackageType::Formula).await
            });

            let casks_result = rt.block_on(async {
                use_case_casks.execute(&query_clone, PackageType::Cask).await
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
            self.task_manager.queue_package_info_load(package_name, package_type);
        }
    }
    
    fn load_package_info_immediate(&mut self, package_name: String, package_type: PackageType) {
        if self.task_manager.is_loading_package_info(&package_name) {
            tracing::debug!("Already loading info for {}, skipping", package_name);
            return;
        }
        
        tracing::info!("Starting to load package info for {} ({:?})", package_name, package_type);
        
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
        
        self.task_manager.add_package_info_task(package_name.clone(), task);
        
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            
            tracing::debug!("Spawned thread for loading {}", name_clone);
            
            let info_result = rt.block_on(async {
                use_case.execute(&name_clone, package_type_clone).await
            });
            
            match info_result {
                Ok(package) => {
                    tracing::info!("Successfully loaded package info for {}: version={:?}", name_clone, package.version);
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
        let result = self.task_manager.poll();

        if let Some(packages) = result.installed_packages {
            self.installed_packages.update_packages(packages);
            self.tab_manager.mark_loaded(Tab::Installed);
            self.loading_installed = false;
            self.status_message = "Packages loaded".to_string();
        }

        if let Some(packages) = result.outdated_packages {
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
            self.status_message = message;
            
            if success {
                self.tab_manager.mark_unloaded(Tab::Installed);
                self.load_installed_packages();
            }
        }

        if let Some((success, message)) = result.uninstall_completed {
            self.loading_uninstall = false;
            self.loading = false;
            self.status_message = message;
            
            if success {
                self.tab_manager.mark_unloaded(Tab::Installed);
                self.load_installed_packages();
            }
        }

        if let Some((success, message)) = result.update_completed {
            self.loading_update = false;
            self.loading = false;
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

        self.log_manager.extend(result.logs);

        if self.task_manager.can_load_more_package_info() && self.task_manager.pending_loads_count() > 0 {
            let to_load = 15 - self.task_manager.pending_loads_count();
            let batch = self.task_manager.drain_pending_loads(to_load);
            
            if !batch.is_empty() {
                tracing::info!("Starting batch load of {} packages ({} remaining in queue)", 
                    batch.len(), self.task_manager.pending_loads_count());
                
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
                
                if ui.selectable_label(self.tab_manager.is_current(Tab::Installed), "Installed").clicked() {
                    self.tab_manager.switch_to(Tab::Installed);
                    if !self.tab_manager.is_loaded(Tab::Installed) {
                        self.load_installed_packages();
                    }
                }
                if ui.selectable_label(self.tab_manager.is_current(Tab::Outdated), "Outdated").clicked() {
                    self.tab_manager.switch_to(Tab::Outdated);
                    if !self.tab_manager.is_loaded(Tab::Outdated) {
                        self.load_outdated_packages();
                    }
                }
                if ui.selectable_label(self.tab_manager.is_current(Tab::SearchInstall), "Search & Install").clicked() {
                    self.tab_manager.switch_to(Tab::SearchInstall);
                }
                if ui.selectable_label(self.tab_manager.is_current(Tab::Settings), "Settings").clicked() {
                    self.tab_manager.switch_to(Tab::Settings);
                }
                if ui.selectable_label(self.tab_manager.is_current(Tab::Log), "Log").clicked() {
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
                    ui.add_space(ui.available_width() / 2.0 - 40.0);
                    if ui.button("Clear Output").clicked() {
                        self.log_manager = LogManager::new();
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
                                ui.label(egui::RichText::new(format!("[{}]", entry.format_timestamp()))
                                    .color(egui::Color32::GRAY).monospace());
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
                        let empty_loading_set = std::collections::HashSet::new();

                        self.installed_packages.show_filtered_with_search_and_pin(
                            ui,
                            &mut install_action,
                            &mut uninstall_action,
                            &mut update_action,
                            self.filter_state.show_formulae(),
                            self.filter_state.show_casks(),
                            self.filter_state.installed_search_query(),
                            &mut load_info_action,
                            &empty_loading_set,
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
                        let empty_loading_set = std::collections::HashSet::new();

                        self.outdated_packages.show_filtered_with_search_and_pin(
                            ui,
                            &mut install_action,
                            &mut uninstall_action,
                            &mut update_action,
                            self.filter_state.show_formulae(),
                            self.filter_state.show_casks(),
                            "",
                            &mut load_info_action,
                            &empty_loading_set,
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
                        let response = ui.text_edit_singleline(self.filter_state.search_query_mut());
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
                            self.task_manager.get_loading_info(),
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
                    
                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            
                            for entry in self.log_manager.all_logs() {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(format!("[{}]", entry.format_timestamp()))
                                        .color(egui::Color32::GRAY).monospace());
                                    ui.monospace(&entry.message);
                                });
                            }
                        });
                }
            }

            if let Some(action) = self.cleanup_modal.render(ctx) {
                match action {
                    CleanupAction::Confirm(cleanup_type) => {
                        match cleanup_type {
                            CleanupType::Cache => self.handle_clean_cache(),
                            CleanupType::OldVersions => self.handle_cleanup_old_versions(),
                        }
                    }
                    CleanupAction::Cancel => {
                        self.cleanup_modal.close();
                    }
                }
            }

            self.info_modal.render(ctx);
        });
    }
}
