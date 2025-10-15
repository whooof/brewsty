use crate::application::use_cases::*;
use crate::domain::entities::{CleanupPreview, Package, PackageType};
use crate::presentation::components::PackageList;
use std::sync::{Arc, Mutex};
use std::thread;

enum AsyncTask {
    LoadInstalled {
        packages: Arc<Mutex<Vec<Package>>>,
        logs: Arc<Mutex<Vec<String>>>,
    },
    LoadOutdated {
        packages: Arc<Mutex<Vec<Package>>>,
        logs: Arc<Mutex<Vec<String>>>,
    },
    Search {
        results: Arc<Mutex<Vec<Package>>>,
        logs: Arc<Mutex<Vec<String>>>,
    },
    LoadPackageInfo {
        package_name: String,
        package_type: PackageType,
        result: Arc<Mutex<Option<Package>>>,
        started_at: std::time::Instant,
    },
}

#[derive(PartialEq)]
enum Tab {
    Installed,
    Outdated,
    Browse,
    Maintenance,
}

#[derive(PartialEq)]
enum CleanupType {
    Cache,
    OldVersions,
}

pub struct BrustyApp {
    current_tab: Tab,
    
    installed_packages: PackageList,
    outdated_packages: PackageList,
    search_results: PackageList,
    
    search_query: String,
    installed_search_query: String,
    
    show_formulae: bool,
    show_casks: bool,
    auto_load_version_info: bool,
    packages_loading_info: std::collections::HashSet<String>,
    pending_package_info_loads: Vec<(String, PackageType)>,
    
    show_cleanup_modal: bool,
    cleanup_type: Option<CleanupType>,
    cleanup_preview: Option<CleanupPreview>,
    cleanup_files: Vec<String>,
    
    initialized: bool,
    installed_loaded: bool,
    outdated_loaded: bool,
    
    loading_installed: bool,
    loading_outdated: bool,
    loading_search: bool,
    
    active_task: Option<AsyncTask>,
    package_info_tasks: Vec<(String, AsyncTask)>,
    
    list_installed_use_case: Arc<ListInstalledPackages>,
    list_outdated_use_case: Arc<ListOutdatedPackages>,
    install_use_case: Arc<InstallPackage>,
    uninstall_use_case: Arc<UninstallPackage>,
    update_use_case: Arc<UpdatePackage>,
    update_all_use_case: Arc<UpdateAllPackages>,
    clean_cache_use_case: Arc<CleanCache>,
    cleanup_old_versions_use_case: Arc<CleanupOldVersions>,
    search_use_case: Arc<SearchPackages>,
    get_package_info_use_case: Arc<GetPackageInfo>,
    
    loading: bool,
    status_message: String,
    output_log: Vec<String>,
    
    runtime: tokio::runtime::Runtime,
}

impl BrustyApp {
    pub fn new(
        list_installed_use_case: Arc<ListInstalledPackages>,
        list_outdated_use_case: Arc<ListOutdatedPackages>,
        install_use_case: Arc<InstallPackage>,
        uninstall_use_case: Arc<UninstallPackage>,
        update_use_case: Arc<UpdatePackage>,
        update_all_use_case: Arc<UpdateAllPackages>,
        clean_cache_use_case: Arc<CleanCache>,
        cleanup_old_versions_use_case: Arc<CleanupOldVersions>,
        search_use_case: Arc<SearchPackages>,
        get_package_info_use_case: Arc<GetPackageInfo>,
    ) -> Self {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        Self {
            current_tab: Tab::Installed,
            installed_packages: PackageList::new(),
            outdated_packages: PackageList::new(),
            search_results: PackageList::new(),
            search_query: String::new(),
            installed_search_query: String::new(),
            show_formulae: true,
            show_casks: true,
            auto_load_version_info: false,
            packages_loading_info: std::collections::HashSet::new(),
            pending_package_info_loads: Vec::new(),
            show_cleanup_modal: false,
            cleanup_type: None,
            cleanup_preview: None,
            cleanup_files: Vec::new(),
            initialized: false,
            installed_loaded: false,
            outdated_loaded: false,
            loading_installed: false,
            loading_outdated: false,
            loading_search: false,
            active_task: None,
            package_info_tasks: Vec::new(),
            list_installed_use_case,
            list_outdated_use_case,
            install_use_case,
            uninstall_use_case,
            update_use_case,
            update_all_use_case,
            clean_cache_use_case,
            cleanup_old_versions_use_case,
            search_use_case,
            get_package_info_use_case,
            loading: false,
            status_message: String::new(),
            output_log: Vec::new(),
            runtime,
        }
    }

    fn load_installed_packages(&mut self) {
        if self.loading_installed {
            return;
        }
        
        self.loading_installed = true;
        self.status_message = "Loading installed packages...".to_string();
        self.output_log.push("Loading installed packages (formulae and casks)".to_string());
        tracing::info!("Loading installed packages (formulae and casks)");

        let use_case_formulae = Arc::clone(&self.list_installed_use_case);
        let use_case_casks = Arc::clone(&self.list_installed_use_case);
        
        let installed_packages = Arc::new(Mutex::new(Vec::new()));
        let output_log = Arc::new(Mutex::new(Vec::new()));
        
        self.active_task = Some(AsyncTask::LoadInstalled {
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
        self.output_log.push("Loading outdated packages (formulae and casks)".to_string());
        tracing::info!("Loading outdated packages (formulae and casks)");

        let use_case_formulae = Arc::clone(&self.list_outdated_use_case);
        let use_case_casks = Arc::clone(&self.list_outdated_use_case);
        
        let outdated_packages = Arc::new(Mutex::new(Vec::new()));
        let output_log = Arc::new(Mutex::new(Vec::new()));

        self.active_task = Some(AsyncTask::LoadOutdated {
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
        let msg = format!("Installing package: {} ({:?})", package.name, package.package_type);
        self.output_log.push(msg.clone());
        tracing::info!("{}", msg);

        let use_case = Arc::clone(&self.install_use_case);
        let result = self.runtime.block_on(async {
            use_case.execute(package.clone()).await
        });

        match result {
            Ok(_) => {
                let msg = format!("Successfully installed {}", package.name);
                self.output_log.push(msg.clone());
                tracing::info!("{}", msg);
                self.status_message = format!("{} installed successfully", package.name);
                self.installed_loaded = false;
                self.load_installed_packages();
            }
            Err(e) => {
                let msg = format!("Error installing {}: {}", package.name, e);
                self.output_log.push(msg.clone());
                tracing::error!("{}", msg);
                self.status_message = msg;
            }
        }

        self.loading = false;
    }

    fn handle_uninstall(&mut self, package: Package) {
        self.loading = true;
        self.status_message = format!("Uninstalling {}...", package.name);
        let msg = format!("Uninstalling package: {} ({:?})", package.name, package.package_type);
        self.output_log.push(msg.clone());
        tracing::info!("{}", msg);

        let use_case = Arc::clone(&self.uninstall_use_case);
        let result = self.runtime.block_on(async {
            use_case.execute(package.clone()).await
        });

        match result {
            Ok(_) => {
                let msg = format!("Successfully uninstalled {}", package.name);
                self.output_log.push(msg.clone());
                tracing::info!("{}", msg);
                self.status_message = format!("{} uninstalled successfully", package.name);
                self.installed_loaded = false;
                self.load_installed_packages();
            }
            Err(e) => {
                let msg = format!("Error uninstalling {}: {}", package.name, e);
                self.output_log.push(msg.clone());
                tracing::error!("{}", msg);
                self.status_message = msg;
            }
        }

        self.loading = false;
    }

    fn handle_update(&mut self, package: Package) {
        self.loading = true;
        self.status_message = format!("Updating {}...", package.name);
        let msg = format!("Updating package: {} ({:?})", package.name, package.package_type);
        self.output_log.push(msg.clone());
        tracing::info!("{}", msg);

        let use_case = Arc::clone(&self.update_use_case);
        let result = self.runtime.block_on(async {
            use_case.execute(package.clone()).await
        });

        match result {
            Ok(_) => {
                let msg = format!("Successfully updated {}", package.name);
                self.output_log.push(msg.clone());
                tracing::info!("{}", msg);
                self.status_message = format!("{} updated successfully", package.name);
                self.installed_loaded = false;
                self.outdated_loaded = false;
                self.load_installed_packages();
                self.load_outdated_packages();
            }
            Err(e) => {
                let msg = format!("Error updating {}: {}", package.name, e);
                self.output_log.push(msg.clone());
                tracing::error!("{}", msg);
                self.status_message = msg;
            }
        }

        self.loading = false;
    }

    fn handle_update_all(&mut self) {
        self.loading = true;
        self.status_message = "Updating all packages...".to_string();
        self.output_log.push("Updating all packages".to_string());
        tracing::info!("Updating all packages");

        let use_case = Arc::clone(&self.update_all_use_case);
        let result = self.runtime.block_on(async {
            use_case.execute().await
        });

        match result {
            Ok(_) => {
                let msg = "Successfully updated all packages".to_string();
                self.output_log.push(msg.clone());
                tracing::info!("{}", msg);
                self.status_message = "All packages updated successfully".to_string();
                self.installed_loaded = false;
                self.outdated_loaded = false;
                self.load_installed_packages();
                self.load_outdated_packages();
            }
            Err(e) => {
                let msg = format!("Error updating all packages: {}", e);
                self.output_log.push(msg.clone());
                tracing::error!("{}", msg);
                self.status_message = msg;
            }
        }

        self.loading = false;
    }

    fn show_cleanup_preview(&mut self, cleanup_type: CleanupType) {
        self.loading = true;
        self.status_message = "Loading cleanup preview...".to_string();
        self.output_log.push("Loading cleanup preview".to_string());

        let preview_result = match cleanup_type {
            CleanupType::Cache => {
                let use_case = Arc::clone(&self.clean_cache_use_case);
                self.runtime.block_on(async { use_case.preview().await })
            }
            CleanupType::OldVersions => {
                let use_case = Arc::clone(&self.cleanup_old_versions_use_case);
                self.runtime.block_on(async { use_case.preview().await })
            }
        };

        match preview_result {
            Ok(preview) => {
                let msg = format!("Found {} items to clean ({})", 
                    preview.items.len(), 
                    format_size(preview.total_size));
                self.output_log.push(msg);
                self.cleanup_preview = Some(preview);
                self.cleanup_type = Some(cleanup_type);
                self.show_cleanup_modal = true;
            }
            Err(e) => {
                let msg = format!("Error getting cleanup preview: {}", e);
                self.output_log.push(msg.clone());
                self.status_message = msg;
            }
        }

        self.loading = false;
    }

    fn handle_clean_cache(&mut self) {
        self.loading = true;
        self.status_message = "Cleaning cache...".to_string();
        self.output_log.push("Cleaning Homebrew cache".to_string());
        tracing::info!("Cleaning Homebrew cache");

        let use_case = Arc::clone(&self.clean_cache_use_case);
        let result = self.runtime.block_on(async {
            use_case.execute().await
        });

        match result {
            Ok(_) => {
                let msg = "Successfully cleaned cache".to_string();
                self.output_log.push(msg.clone());
                tracing::info!("{}", msg);
                self.status_message = "Cache cleaned successfully".to_string();
            }
            Err(e) => {
                let msg = format!("Error cleaning cache: {}", e);
                self.output_log.push(msg.clone());
                tracing::error!("{}", msg);
                self.status_message = msg;
            }
        }

        self.loading = false;
        self.show_cleanup_modal = false;
        self.cleanup_preview = None;
        self.cleanup_type = None;
    }

    fn handle_cleanup_old_versions(&mut self) {
        self.loading = true;
        self.status_message = "Cleaning up old versions...".to_string();
        self.output_log.push("Cleaning up old versions".to_string());
        tracing::info!("Cleaning up old versions");

        let use_case = Arc::clone(&self.cleanup_old_versions_use_case);
        let result = self.runtime.block_on(async {
            use_case.execute().await
        });

        match result {
            Ok(_) => {
                let msg = "Successfully cleaned up old versions".to_string();
                self.output_log.push(msg.clone());
                tracing::info!("{}", msg);
                self.status_message = "Old versions cleaned up successfully".to_string();
            }
            Err(e) => {
                let msg = format!("Error cleaning up old versions: {}", e);
                self.output_log.push(msg.clone());
                tracing::error!("{}", msg);
                self.status_message = msg;
            }
        }

        self.loading = false;
        self.show_cleanup_modal = false;
        self.cleanup_preview = None;
        self.cleanup_type = None;
    }

    fn handle_search(&mut self) {
        if self.search_query.is_empty() {
            return;
        }
        
        if self.loading_search {
            return;
        }

        self.loading_search = true;
        self.status_message = format!("Searching for '{}'...", self.search_query);
        let msg = format!("Searching for: {}", self.search_query);
        self.output_log.push(msg.clone());
        tracing::info!("{}", msg);

        let use_case_formulae = Arc::clone(&self.search_use_case);
        let use_case_casks = Arc::clone(&self.search_use_case);
        let query = self.search_query.clone();
        
        let search_results = Arc::new(Mutex::new(Vec::new()));
        let output_log = Arc::new(Mutex::new(Vec::new()));
        let query_clone = query.clone();

        self.active_task = Some(AsyncTask::Search {
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
        if self.packages_loading_info.contains(&package_name) {
            tracing::debug!("Already loading info for {}, skipping", package_name);
            return;
        }
        
        if self.pending_package_info_loads.iter().any(|(name, _)| name == &package_name) {
            tracing::debug!("Already queued for loading: {}", package_name);
            return;
        }
        
        if self.packages_loading_info.len() < 15 {
            self.load_package_info_immediate(package_name, package_type);
        } else {
            tracing::debug!("Queueing {} for batch loading (current queue size: {})", package_name, self.pending_package_info_loads.len());
            self.pending_package_info_loads.push((package_name, package_type));
        }
    }
    
    fn load_package_info_immediate(&mut self, package_name: String, package_type: PackageType) {
        if self.packages_loading_info.contains(&package_name) {
            tracing::debug!("Already loading info for {}, skipping", package_name);
            return;
        }
        
        tracing::info!("Starting to load package info for {} ({:?})", package_name, package_type);
        self.packages_loading_info.insert(package_name.clone());
        
        let use_case = Arc::clone(&self.get_package_info_use_case);
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
        
        self.package_info_tasks.push((package_name.clone(), task));
        
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
        let mut tasks_to_keep = Vec::new();
        
        for (pkg_name, task) in self.package_info_tasks.drain(..) {
            match task {
                AsyncTask::LoadPackageInfo { package_name, package_type, result, started_at } => {
                    let elapsed = started_at.elapsed();
                    
                    if elapsed > std::time::Duration::from_secs(10) {
                        tracing::warn!("Package info loading timed out for {} after {:?}", package_name, elapsed);
                        let failed_package = Package::new(package_name.clone(), package_type)
                            .set_version_load_failed(true);
                        self.search_results.update_package(failed_package);
                        self.packages_loading_info.remove(&package_name);
                        continue;
                    }
                    
                    let should_keep = match result.try_lock() {
                        Ok(pkg_opt) => {
                            if let Some(package) = pkg_opt.clone() {
                                tracing::info!("Updating search results with package info for {}", package_name);
                                self.search_results.update_package(package);
                                self.packages_loading_info.remove(&package_name);
                                false
                            } else {
                                true
                            }
                        }
                        Err(_) => true
                    };
                    
                    if should_keep {
                        tasks_to_keep.push((pkg_name, AsyncTask::LoadPackageInfo { package_name, package_type, result, started_at }));
                    }
                }
                _ => {}
            }
        }
        
        self.package_info_tasks = tasks_to_keep;
        
        let current_loading = self.packages_loading_info.len();
        if current_loading < 15 && !self.pending_package_info_loads.is_empty() {
            let to_load = 15 - current_loading;
            let batch: Vec<_> = self.pending_package_info_loads.drain(..to_load.min(self.pending_package_info_loads.len())).collect();
            
            tracing::info!("Starting batch load of {} packages ({} remaining in queue)", batch.len(), self.pending_package_info_loads.len());
            
            for (name, pkg_type) in batch {
                self.load_package_info_immediate(name, pkg_type);
            }
        }
        
        if let Some(task) = self.active_task.take() {
            match task {
                AsyncTask::LoadInstalled { packages, logs } => {
                    let should_put_back = match logs.try_lock() {
                        Ok(log) => {
                            if !log.is_empty() {
                                if let Ok(pkgs) = packages.try_lock() {
                                    self.installed_packages.update_packages(pkgs.clone());
                                    self.installed_loaded = true;
                                    self.loading_installed = false;
                                    self.status_message = "Packages loaded".to_string();
                                    self.output_log.extend(log.clone());
                                    false
                                } else {
                                    true
                                }
                            } else {
                                true
                            }
                        }
                        Err(_) => true
                    };
                    
                    if should_put_back {
                        self.active_task = Some(AsyncTask::LoadInstalled { packages, logs });
                    }
                }
                AsyncTask::LoadOutdated { packages, logs } => {
                    let should_put_back = match logs.try_lock() {
                        Ok(log) => {
                            if !self.loading_outdated {
                                false
                            } else if !log.is_empty() {
                                if let Ok(pkgs) = packages.try_lock() {
                                    self.outdated_packages.update_packages(pkgs.clone());
                                    self.outdated_loaded = true;
                                    self.loading_outdated = false;
                                    self.status_message = "Outdated packages loaded".to_string();
                                    self.output_log.extend(log.clone());
                                    false
                                } else {
                                    true
                                }
                            } else {
                                true
                            }
                        }
                        Err(_) => true
                    };
                    
                    if should_put_back {
                        self.active_task = Some(AsyncTask::LoadOutdated { packages, logs });
                    }
                }
                AsyncTask::Search { results, logs } => {
                    let should_put_back = match results.try_lock() {
                        Ok(res) => {
                            if let Ok(log) = logs.try_lock() {
                                if !log.is_empty() {
                                    tracing::info!("Search completed, found {} packages", res.len());
                                    self.search_results.update_packages(res.clone());
                                    self.loading_search = false;
                                    self.status_message = format!("Search completed");
                                    self.output_log.extend(log.clone());
                                    
                                    if self.auto_load_version_info {
                                        tracing::info!("Auto-loading version info for {} packages", res.len());
                                        for package in res.iter() {
                                            if package.version.is_none() && !package.version_load_failed {
                                                tracing::debug!("Auto-loading info for {}", package.name);
                                                self.load_package_info(package.name.clone(), package.package_type.clone());
                                            }
                                        }
                                    }
                                    
                                    false
                                } else {
                                    true
                                }
                            } else {
                                true
                            }
                        }
                        Err(_) => true
                    };
                    
                    if should_put_back {
                        self.active_task = Some(AsyncTask::Search { results, logs });
                    }
                }
                AsyncTask::LoadPackageInfo { .. } => {
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

impl eframe::App for BrustyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_async_tasks();
        ctx.request_repaint();
        
        if !self.initialized {
            self.initialized = true;
            self.load_installed_packages();
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🍺 Brusty");
                ui.label("v0.1.0");
                ui.separator();
                
                if ui.selectable_label(self.current_tab == Tab::Installed, "Installed").clicked() {
                    self.current_tab = Tab::Installed;
                    if !self.installed_loaded {
                        self.load_installed_packages();
                    }
                }
                if ui.selectable_label(self.current_tab == Tab::Outdated, "Outdated").clicked() {
                    self.current_tab = Tab::Outdated;
                    if !self.outdated_loaded {
                        self.load_outdated_packages();
                    }
                }
                if ui.selectable_label(self.current_tab == Tab::Browse, "Browse").clicked() {
                    self.current_tab = Tab::Browse;
                }
                if ui.selectable_label(self.current_tab == Tab::Maintenance, "Maintenance").clicked() {
                    self.current_tab = Tab::Maintenance;
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
                    for line in self.output_log.iter().rev().take(20).rev() {
                        ui.label(line);
                    }
                });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_tab {
                Tab::Installed => {
                    ui.horizontal(|ui| {
                        ui.label("Search:");
                        ui.text_edit_singleline(&mut self.installed_search_query);
                        ui.separator();
                        ui.checkbox(&mut self.show_formulae, "Show Formulae");
                        ui.checkbox(&mut self.show_casks, "Show Casks");
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
                            self.show_formulae,
                            self.show_casks,
                            &self.installed_search_query,
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
                        ui.checkbox(&mut self.show_formulae, "Show Formulae");
                        ui.checkbox(&mut self.show_casks, "Show Casks");
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
                            self.show_formulae,
                            self.show_casks,
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
                        let response = ui.text_edit_singleline(&mut self.search_query);
                        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            self.handle_search();
                        }
                        if ui.button("Search").clicked() {
                            self.handle_search();
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.show_formulae, "Show Formulae");
                        ui.checkbox(&mut self.show_casks, "Show Casks");
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
                            self.show_formulae,
                            self.show_casks,
                            "",
                            &mut load_info_action,
                            &self.packages_loading_info,
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

            if self.show_cleanup_modal {
                egui::Window::new("Cleanup Preview")
                    .collapsible(false)
                    .resizable(true)
                    .show(ctx, |ui| {
                        if let Some(preview) = &self.cleanup_preview {
                            ui.heading(format!("Total size to free: {}", format_size(preview.total_size)));
                            ui.separator();

                            ui.label(format!("Files and folders to be removed ({} items):", preview.items.len()));
                            
                            egui::ScrollArea::vertical()
                                .max_height(300.0)
                                .show(ui, |ui| {
                                    for item in &preview.items {
                                        ui.horizontal(|ui| {
                                            ui.label(&item.path);
                                            ui.label(format!("({})", format_size(item.size)));
                                        });
                                    }
                                });

                            ui.separator();

                            ui.horizontal(|ui| {
                                if ui.button("Confirm").clicked() {
                                    if let Some(cleanup_type) = &self.cleanup_type {
                                        match cleanup_type {
                                            CleanupType::Cache => self.handle_clean_cache(),
                                            CleanupType::OldVersions => self.handle_cleanup_old_versions(),
                                        }
                                    }
                                }

                                if ui.button("Cancel").clicked() {
                                    self.show_cleanup_modal = false;
                                    self.cleanup_preview = None;
                                    self.cleanup_type = None;
                                }
                            });
                        }
                    });
            }
        });
    }
}
