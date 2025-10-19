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
}

pub struct TaskResult {
    pub installed_packages: Option<Vec<Package>>,
    pub outdated_packages: Option<Vec<Package>>,
    pub search_results: Option<Vec<Package>>,
    pub package_info: Option<(String, Package)>,
    pub logs: Vec<String>,
    pub completed_package_info_loads: Vec<String>,
}

pub struct AsyncTaskManager {
    active_task: Option<AsyncTask>,
    package_info_tasks: Vec<(String, AsyncTask)>,
    packages_loading_info: HashSet<String>,
    pending_package_info_loads: Vec<(String, PackageType)>,
}

impl AsyncTaskManager {
    pub fn new() -> Self {
        Self {
            active_task: None,
            package_info_tasks: Vec::new(),
            packages_loading_info: HashSet::new(),
            pending_package_info_loads: Vec::new(),
        }
    }

    pub fn set_active_task(&mut self, task: AsyncTask) {
        self.active_task = Some(task);
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

    pub fn get_loading_info(&self) -> &HashSet<String> {
        &self.packages_loading_info
    }

    pub fn poll(&mut self) -> TaskResult {
        let mut result = TaskResult {
            installed_packages: None,
            outdated_packages: None,
            search_results: None,
            package_info: None,
            logs: Vec::new(),
            completed_package_info_loads: Vec::new(),
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

        if let Some(task) = self.active_task.take() {
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
                        self.active_task = Some(AsyncTask::LoadInstalled { packages, logs });
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
                        self.active_task = Some(AsyncTask::LoadOutdated { packages, logs });
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
                        self.active_task = Some(AsyncTask::Search { results, logs });
                    }
                }
                AsyncTask::LoadPackageInfo { .. } => {
                }
            }
        }

        result
    }
}
