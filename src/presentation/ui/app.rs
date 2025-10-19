use crate::application::UseCaseContainer;
use crate::domain::entities::{Package, PackageType};
use crate::presentation::components::{
    CleanupAction, CleanupModal, CleanupType, FilterState, LogManager, PackageList, Tab, TabManager
};
use crate::presentation::services::{
    AsyncExecutor, AsyncTask, AsyncTaskManager, PackageOperationHandler
};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct BrewstyApp {
    tab_manager: TabManager,
    filter_state: FilterState,
    cleanup_modal: CleanupModal,
    log_manager: LogManager,
    
    installed_packages: PackageList,
    outdated_packages: PackageList,
    search_results: PackageList,
    
    auto_load_version_info: bool,
    
    initialized: bool,
    
    loading_installed: bool,
    loading_outdated: bool,
    loading_search: bool,
    
    task_manager: AsyncTaskManager,
    
    use_cases: Arc<UseCaseContainer>,
    executor: AsyncExecutor,
    package_ops: PackageOperationHandler,
    
    loading: bool,
    status_message: String,
}

impl BrewstyApp {
    pub fn new(use_cases: Arc<UseCaseContainer>) -> Self {
        let executor = AsyncExecutor::new();
        
        let package_ops = PackageOperationHandler::new(
            Arc::clone(&use_cases.install),
            Arc::clone(&use_cases.uninstall),
            Arc::clone(&use_cases.update),
            executor.clone(),
        );

        Self {
            tab_manager: TabManager::new(),
            filter_state: FilterState::new(),
            cleanup_modal: CleanupModal::new(),
            log_manager: LogManager::new(),
            installed_packages: PackageList::new(),
            outdated_packages: PackageList::new(),
            search_results: PackageList::new(),
            auto_load_version_info: false,
            initialized: false,
            loading_installed: false,
            loading_outdated: false,
            loading_search: false,
            task_manager: AsyncTaskManager::new(),
            use_cases,
            executor,
            package_ops,
            loading: false,
            status_message: String::new(),
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
        self.loading = true;
        self.status_message = format!("Installing {}...", package.name);
        
        let result = self.package_ops.install(package);
        
        self.status_message = result.message;
        self.log_manager.extend(result.log_messages);
        self.loading = false;
        
        if result.success {
            self.tab_manager.mark_unloaded(Tab::Installed);
            self.load_installed_packages();
        }
    }

    fn handle_uninstall(&mut self, package: Package) {
        self.loading = true;
        self.status_message = format!("Uninstalling {}...", package.name);
        
        let result = self.package_ops.uninstall(package);
        
        self.status_message = result.message;
        self.log_manager.extend(result.log_messages);
        self.loading = false;
        
        if result.success {
            self.tab_manager.mark_unloaded(Tab::Installed);
            self.load_installed_packages();
        }
    }

    fn handle_update(&mut self, package: Package) {
        self.loading = true;
        self.status_message = format!("Updating {}...", package.name);
        
        let result = self.package_ops.update(package);
        
        self.status_message = result.message;
        self.log_manager.extend(result.log_messages);
        self.loading = false;
        
        if result.success {
            self.tab_manager.mark_unloaded(Tab::Installed);
            self.tab_manager.mark_unloaded(Tab::Outdated);
            self.load_installed_packages();
            self.load_outdated_packages();
        }
    }

    fn handle_update_all(&mut self) {
        self.loading = true;
        self.status_message = "Updating all packages...".to_string();
        self.log_manager.push("Updating all packages".to_string());
        tracing::info!("Updating all packages");

        let use_case = Arc::clone(&self.use_cases.update_all);
        let result = self.executor.execute(async {
            use_case.execute().await
        });

        match result {
            Ok(_) => {
                let msg = "Successfully updated all packages".to_string();
                self.log_manager.push(msg.clone());
                tracing::info!("{}", msg);
                self.status_message = "All packages updated successfully".to_string();
                self.tab_manager.mark_unloaded(Tab::Installed);
                self.tab_manager.mark_unloaded(Tab::Outdated);
                self.load_installed_packages();
                self.load_outdated_packages();
            }
            Err(e) => {
                let msg = format!("Error updating all packages: {}", e);
                self.log_manager.push(msg.clone());
                tracing::error!("{}", msg);
                self.status_message = msg;
            }
        }

        self.loading = false;
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
        self.poll_async_tasks();
        ctx.request_repaint();
        
        if !self.initialized {
            self.initialized = true;
            self.load_installed_packages();
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🍺 Brewsty");
                ui.label("v0.1.0");
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
                if ui.selectable_label(self.tab_manager.is_current(Tab::Browse), "Browse").clicked() {
                    self.tab_manager.switch_to(Tab::Browse);
                }
                if ui.selectable_label(self.tab_manager.is_current(Tab::Maintenance), "Maintenance").clicked() {
                    self.tab_manager.switch_to(Tab::Maintenance);
                }
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Status:");
                if self.loading {
                    ui.spinner();
                }
                ui.label(&self.status_message);
            });
            
            ui.separator();
            
            ui.label("Output:");
            egui::ScrollArea::vertical()
                .max_height(100.0)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    for line in self.log_manager.recent() {
                        ui.label(line);
                    }
                });
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

                        self.installed_packages.show_filtered_with_search(
                            ui,
                            &mut install_action,
                            &mut uninstall_action,
                            &mut update_action,
                            self.filter_state.show_formulae(),
                            self.filter_state.show_casks(),
                            self.filter_state.installed_search_query(),
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

                        self.outdated_packages.show_filtered(
                            ui,
                            &mut install_action,
                            &mut uninstall_action,
                            &mut update_action,
                            self.filter_state.show_formulae(),
                            self.filter_state.show_casks(),
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
                    }
                }

                Tab::Browse => {
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

                        self.search_results.show_filtered_with_search_and_load_info(
                            ui,
                            &mut install_action,
                            &mut uninstall_action,
                            &mut update_action,
                            self.filter_state.show_formulae(),
                            self.filter_state.show_casks(),
                            "",
                            &mut load_info_action,
                            self.task_manager.get_loading_info(),
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
                    }
                }

                Tab::Maintenance => {
                    ui.heading("Maintenance Operations");
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
        });
    }
}
