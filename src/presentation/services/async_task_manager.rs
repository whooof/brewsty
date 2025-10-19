use crate::domain::entities::{Package, PackageType};
use std::sync::{Arc, Mutex};
use std::collections::HashSet;

pub enum AsyncTask {
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
    Install {
        success: Arc<Mutex<Option<bool>>>,
        logs: Arc<Mutex<Vec<String>>>,
        message: Arc<Mutex<String>>,
    },
    Uninstall {
        success: Arc<Mutex<Option<bool>>>,
        logs: Arc<Mutex<Vec<String>>>,
        message: Arc<Mutex<String>>,
    },
    Update {
        success: Arc<Mutex<Option<bool>>>,
        logs: Arc<Mutex<Vec<String>>>,
        message: Arc<Mutex<String>>,
    },
    UpdateAll {
        success: Arc<Mutex<Option<bool>>>,
        logs: Arc<Mutex<Vec<String>>>,
        message: Arc<Mutex<String>>,
    },
    CleanCache {
        success: Arc<Mutex<Option<bool>>>,
        logs: Arc<Mutex<Vec<String>>>,
        message: Arc<Mutex<String>>,
    },
    CleanupOldVersions {
        success: Arc<Mutex<Option<bool>>>,
        logs: Arc<Mutex<Vec<String>>>,
        message: Arc<Mutex<String>>,
    },
    Pin {
        package_name: String,
        success: Arc<Mutex<Option<bool>>>,
        logs: Arc<Mutex<Vec<String>>>,
        message: Arc<Mutex<String>>,
    },
    Unpin {
        package_name: String,
        success: Arc<Mutex<Option<bool>>>,
        logs: Arc<Mutex<Vec<String>>>,
        message: Arc<Mutex<String>>,
    },
}

pub struct TaskResult {
    pub installed_packages: Option<Vec<Package>>,
    pub outdated_packages: Option<Vec<Package>>,
    pub search_results: Option<Vec<Package>>,
    pub package_info: Option<(String, Package)>,
    pub logs: Vec<String>,
    pub completed_package_info_loads: Vec<String>,
    pub install_completed: Option<(bool, String)>,
    pub uninstall_completed: Option<(bool, String)>,
    pub update_completed: Option<(bool, String)>,
    pub update_all_completed: Option<(bool, String)>,
    pub clean_cache_completed: Option<(bool, String)>,
    pub cleanup_old_versions_completed: Option<(bool, String)>,
    pub pin_completed: Option<(String, bool, String)>,
    pub unpin_completed: Option<(String, bool, String)>,
}

pub struct AsyncTaskManager {
    active_tasks: Vec<AsyncTask>,
    package_info_tasks: Vec<(String, AsyncTask)>,
    packages_loading_info: HashSet<String>,
    pending_package_info_loads: Vec<(String, PackageType)>,
}

impl AsyncTaskManager {
    pub fn new() -> Self {
        Self {
            active_tasks: Vec::new(),
            package_info_tasks: Vec::new(),
            packages_loading_info: HashSet::new(),
            pending_package_info_loads: Vec::new(),
        }
    }

    pub fn set_active_task(&mut self, task: AsyncTask) {
        let task_type = match &task {
            AsyncTask::LoadInstalled { .. } => "LoadInstalled",
            AsyncTask::LoadOutdated { .. } => "LoadOutdated",
            AsyncTask::Search { .. } => "Search",
            _ => "",
        };
        
        if !task_type.is_empty() && self.has_task_type(task_type) {
            tracing::warn!("{} task is already running, ignoring duplicate", task_type);
            return;
        }
        
        self.active_tasks.push(task);
    }

    pub fn has_task_type(&self, task_type: &str) -> bool {
         self.active_tasks.iter().any(|task| {
             match (task, task_type) {
                 (AsyncTask::LoadInstalled { .. }, "LoadInstalled") => true,
                 (AsyncTask::LoadOutdated { .. }, "LoadOutdated") => true,
                 (AsyncTask::Search { .. }, "Search") => true,
                 _ => false,
             }
         })
     }

    pub fn add_package_info_task(&mut self, package_name: String, task: AsyncTask) {
        self.packages_loading_info.insert(package_name.clone());
        self.package_info_tasks.push((package_name, task));
    }

    pub fn is_loading_package_info(&self, package_name: &str) -> bool {
        self.packages_loading_info.contains(package_name)
    }

    pub fn queue_package_info_load(&mut self, package_name: String, package_type: PackageType) {
        if self.packages_loading_info.contains(&package_name) {
            tracing::debug!("Already loading info for {}, skipping", package_name);
            return;
        }
        
        if self.pending_package_info_loads.iter().any(|(name, _)| name == &package_name) {
            tracing::debug!("Already queued for loading: {}", package_name);
            return;
        }
        
        self.pending_package_info_loads.push((package_name, package_type));
    }

    pub fn can_load_more_package_info(&self) -> bool {
        self.packages_loading_info.len() < 15
    }

    pub fn drain_pending_loads(&mut self, count: usize) -> Vec<(String, PackageType)> {
        self.pending_package_info_loads.drain(..count.min(self.pending_package_info_loads.len())).collect()
    }

    pub fn pending_loads_count(&self) -> usize {
        self.pending_package_info_loads.len()
    }

    pub fn poll(&mut self) -> TaskResult {
        let mut result = TaskResult {
            installed_packages: None,
            outdated_packages: None,
            search_results: None,
            package_info: None,
            logs: Vec::new(),
            completed_package_info_loads: Vec::new(),
            install_completed: None,
            uninstall_completed: None,
            update_completed: None,
            update_all_completed: None,
            clean_cache_completed: None,
            cleanup_old_versions_completed: None,
            pin_completed: None,
            unpin_completed: None,
        };

        let mut tasks_to_keep = Vec::new();
        
        for (pkg_name, task) in self.package_info_tasks.drain(..) {
            match task {
                AsyncTask::LoadPackageInfo { package_name, package_type, result: pkg_result, started_at } => {
                    let elapsed = started_at.elapsed();
                    
                    if elapsed > std::time::Duration::from_secs(10) {
                        tracing::warn!("Package info loading timed out for {} after {:?}", package_name, elapsed);
                        let failed_package = Package::new(package_name.clone(), package_type)
                            .set_version_load_failed(true);
                        result.package_info = Some((package_name.clone(), failed_package));
                        self.packages_loading_info.remove(&package_name);
                        result.completed_package_info_loads.push(package_name);
                        continue;
                    }
                    
                    let package_name_clone = package_name.clone();
                    let should_keep = match pkg_result.try_lock() {
                        Ok(pkg_opt) => {
                            if let Some(package) = pkg_opt.clone() {
                                tracing::info!("Updating search results with package info for {}", package_name_clone);
                                result.package_info = Some((package_name_clone.clone(), package));
                                self.packages_loading_info.remove(&package_name_clone);
                                result.completed_package_info_loads.push(package_name_clone);
                                false
                            } else {
                                true
                            }
                        }
                        Err(_) => true
                    };
                    
                    if should_keep {
                        tasks_to_keep.push((pkg_name, AsyncTask::LoadPackageInfo { package_name, package_type, result: pkg_result, started_at }));
                    }
                }
                _ => {}
            }
        }
        
        self.package_info_tasks = tasks_to_keep;

        let mut active_tasks_to_keep = Vec::new();
        
        for task in self.active_tasks.drain(..) {
            match task {
                AsyncTask::LoadInstalled { packages, logs } => {
                    let should_put_back = match logs.try_lock() {
                        Ok(log) => {
                            if !log.is_empty() {
                                if let Ok(pkgs) = packages.try_lock() {
                                    result.installed_packages = Some(pkgs.clone());
                                    result.logs.extend(log.clone());
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
                        active_tasks_to_keep.push(AsyncTask::LoadInstalled { packages, logs });
                    }
                }
                AsyncTask::LoadOutdated { packages, logs } => {
                    let should_put_back = match logs.try_lock() {
                        Ok(log) => {
                            if !log.is_empty() {
                                if let Ok(pkgs) = packages.try_lock() {
                                    result.outdated_packages = Some(pkgs.clone());
                                    result.logs.extend(log.clone());
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
                        active_tasks_to_keep.push(AsyncTask::LoadOutdated { packages, logs });
                    }
                }
                AsyncTask::Search { results, logs } => {
                    let should_put_back = match results.try_lock() {
                        Ok(res) => {
                            if let Ok(log) = logs.try_lock() {
                                if !log.is_empty() {
                                    tracing::info!("Search completed, found {} packages", res.len());
                                    result.search_results = Some(res.clone());
                                    result.logs.extend(log.clone());
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
                        active_tasks_to_keep.push(AsyncTask::Search { results, logs });
                    }
                }
                AsyncTask::Install { success, logs, message } => {
                    let should_put_back = match success.try_lock() {
                        Ok(success_opt) => {
                            if let Some(succeeded) = *success_opt {
                                if let (Ok(log), Ok(msg)) = (logs.try_lock(), message.try_lock()) {
                                    result.install_completed = Some((succeeded, msg.clone()));
                                    result.logs.extend(log.clone());
                                    false
                                } else {
                                    true
                                }
                            } else {
                                true
                            }
                        }
                        Err(_) => true,
                    };
                    
                    if should_put_back {
                        active_tasks_to_keep.push(AsyncTask::Install { success, logs, message });
                    }
                }
                AsyncTask::Uninstall { success, logs, message } => {
                    let should_put_back = match success.try_lock() {
                        Ok(success_opt) => {
                            if let Some(succeeded) = *success_opt {
                                if let (Ok(log), Ok(msg)) = (logs.try_lock(), message.try_lock()) {
                                    result.uninstall_completed = Some((succeeded, msg.clone()));
                                    result.logs.extend(log.clone());
                                    false
                                } else {
                                    true
                                }
                            } else {
                                true
                            }
                        }
                        Err(_) => true,
                    };
                    
                    if should_put_back {
                        active_tasks_to_keep.push(AsyncTask::Uninstall { success, logs, message });
                    }
                }
                AsyncTask::Update { success, logs, message } => {
                    let should_put_back = match success.try_lock() {
                        Ok(success_opt) => {
                            if let Some(succeeded) = *success_opt {
                                if let (Ok(log), Ok(msg)) = (logs.try_lock(), message.try_lock()) {
                                    result.update_completed = Some((succeeded, msg.clone()));
                                    result.logs.extend(log.clone());
                                    false
                                } else {
                                    true
                                }
                            } else {
                                true
                            }
                        }
                        Err(_) => true,
                    };
                    
                    if should_put_back {
                        active_tasks_to_keep.push(AsyncTask::Update { success, logs, message });
                    }
                }
                AsyncTask::UpdateAll { success, logs, message } => {
                    let should_put_back = match success.try_lock() {
                        Ok(success_opt) => {
                            if let Some(succeeded) = *success_opt {
                                if let (Ok(log), Ok(msg)) = (logs.try_lock(), message.try_lock()) {
                                    result.update_all_completed = Some((succeeded, msg.clone()));
                                    result.logs.extend(log.clone());
                                    false
                                } else {
                                    true
                                }
                            } else {
                                true
                            }
                        }
                        Err(_) => true,
                    };
                    
                    if should_put_back {
                        active_tasks_to_keep.push(AsyncTask::UpdateAll { success, logs, message });
                    }
                }
                AsyncTask::CleanCache { success, logs, message } => {
                    let should_put_back = match success.try_lock() {
                        Ok(success_opt) => {
                            if let Some(succeeded) = *success_opt {
                                if let (Ok(log), Ok(msg)) = (logs.try_lock(), message.try_lock()) {
                                    result.clean_cache_completed = Some((succeeded, msg.clone()));
                                    result.logs.extend(log.clone());
                                    false
                                } else {
                                    true
                                }
                            } else {
                                true
                            }
                        }
                        Err(_) => true,
                    };
                    
                    if should_put_back {
                        active_tasks_to_keep.push(AsyncTask::CleanCache { success, logs, message });
                    }
                }
                AsyncTask::CleanupOldVersions { success, logs, message } => {
                    let should_put_back = match success.try_lock() {
                        Ok(success_opt) => {
                            if let Some(succeeded) = *success_opt {
                                if let (Ok(log), Ok(msg)) = (logs.try_lock(), message.try_lock()) {
                                    result.cleanup_old_versions_completed = Some((succeeded, msg.clone()));
                                    result.logs.extend(log.clone());
                                    false
                                } else {
                                    true
                                }
                            } else {
                                true
                            }
                        }
                        Err(_) => true,
                    };
                    
                    if should_put_back {
                        active_tasks_to_keep.push(AsyncTask::CleanupOldVersions { success, logs, message });
                    }
                }
                AsyncTask::Pin { package_name, success, logs, message } => {
                    let should_put_back = match success.try_lock() {
                        Ok(success_opt) => {
                            if let Some(succeeded) = *success_opt {
                                if let (Ok(log), Ok(msg)) = (logs.try_lock(), message.try_lock()) {
                                    result.pin_completed = Some((package_name.clone(), succeeded, msg.clone()));
                                    result.logs.extend(log.clone());
                                    false
                                } else {
                                    true
                                }
                            } else {
                                true
                            }
                        }
                        Err(_) => true,
                    };
                    
                    if should_put_back {
                        active_tasks_to_keep.push(AsyncTask::Pin { package_name, success, logs, message });
                    }
                }
                AsyncTask::Unpin { package_name, success, logs, message } => {
                    let should_put_back = match success.try_lock() {
                        Ok(success_opt) => {
                            if let Some(succeeded) = *success_opt {
                                if let (Ok(log), Ok(msg)) = (logs.try_lock(), message.try_lock()) {
                                    result.unpin_completed = Some((package_name.clone(), succeeded, msg.clone()));
                                    result.logs.extend(log.clone());
                                    false
                                } else {
                                    true
                                }
                            } else {
                                true
                            }
                        }
                        Err(_) => true,
                    };
                    
                    if should_put_back {
                        active_tasks_to_keep.push(AsyncTask::Unpin { package_name, success, logs, message });
                    }
                }
                AsyncTask::LoadPackageInfo { .. } => {
                }
            }
        }
        
        self.active_tasks = active_tasks_to_keep;

        result
    }
}
