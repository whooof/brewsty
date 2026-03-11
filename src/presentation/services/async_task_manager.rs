use crate::domain::entities::brewfile::BrewfileSyncPreview;
use crate::domain::entities::{
    CleanupPreview, DoctorOutput, Package, PackageType, Service, ServiceInfo,
};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskKind {
    LoadInstalled,
    LoadOutdated,
    Search,
}

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
    CleanOrphans {
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
    LoadServices {
        services: Arc<Mutex<Vec<Service>>>,
        logs: Arc<Mutex<Vec<String>>>,
    },
    StartService {
        service_name: String,
        success: Arc<Mutex<Option<bool>>>,
        logs: Arc<Mutex<Vec<String>>>,
        message: Arc<Mutex<String>>,
    },
    StopService {
        service_name: String,
        success: Arc<Mutex<Option<bool>>>,
        logs: Arc<Mutex<Vec<String>>>,
        message: Arc<Mutex<String>>,
    },
    RestartService {
        service_name: String,
        success: Arc<Mutex<Option<bool>>>,
        logs: Arc<Mutex<Vec<String>>>,
        message: Arc<Mutex<String>>,
    },
    ExportPackages {
        success: Arc<Mutex<Option<bool>>>,
        logs: Arc<Mutex<Vec<String>>>,
        message: Arc<Mutex<String>>,
    },
    ImportPackages {
        success: Arc<Mutex<Option<bool>>>,
        logs: Arc<Mutex<Vec<String>>>,
        message: Arc<Mutex<String>>,
    },
    CleanupPreview {
        cleanup_type: crate::presentation::components::CleanupType,
        preview: Arc<Mutex<Option<CleanupPreview>>>,
        error: Arc<Mutex<Option<String>>>,
    },
    Doctor {
        result: Arc<Mutex<Option<DoctorOutput>>>,
        error: Arc<Mutex<Option<String>>>,
    },
    LoadTaps {
        taps: Arc<Mutex<Vec<String>>>,
        logs: Arc<Mutex<Vec<String>>>,
    },
    Tap {
        success: Arc<Mutex<Option<bool>>>,
        logs: Arc<Mutex<Vec<String>>>,
        message: Arc<Mutex<String>>,
    },
    Untap {
        success: Arc<Mutex<Option<bool>>>,
        logs: Arc<Mutex<Vec<String>>>,
        message: Arc<Mutex<String>>,
    },
    BundleDump {
        success: Arc<Mutex<Option<bool>>>,
        logs: Arc<Mutex<Vec<String>>>,
        message: Arc<Mutex<String>>,
    },
    BundleCheckPreview {
        preview: Arc<Mutex<Option<BrewfileSyncPreview>>>,
        error: Arc<Mutex<Option<String>>>,
    },
    BundleApply {
        success: Arc<Mutex<Option<bool>>>,
        logs: Arc<Mutex<Vec<String>>>,
        message: Arc<Mutex<String>>,
    },
    ServiceInfoLoad {
        service_name: String,
        result: Arc<Mutex<Option<ServiceInfo>>>,
        error: Arc<Mutex<Option<String>>>,
    },
    ServiceLogLoad {
        service_name: String,
        result: Arc<Mutex<Option<String>>>,
        error: Arc<Mutex<Option<String>>>,
    },
}

/// Shared state for async tasks that produce a success/failure result.
pub struct TaskSharedState {
    pub success: Arc<Mutex<Option<bool>>>,
    pub logs: Arc<Mutex<Vec<String>>>,
    pub message: Arc<Mutex<String>>,
}

impl TaskSharedState {
    pub fn new() -> Self {
        Self {
            success: Arc::new(Mutex::new(None)),
            logs: Arc::new(Mutex::new(Vec::new())),
            message: Arc::new(Mutex::new(String::new())),
        }
    }

    /// Write a successful result into the shared state.
    pub fn set_success(&self, msg: String) {
        if let Ok(mut s) = self.success.lock() {
            *s = Some(true);
        }
        if let Ok(mut m) = self.message.lock() {
            *m = msg.clone();
        }
        if let Ok(mut l) = self.logs.lock() {
            l.push(msg);
        }
    }

    /// Write a failure result into the shared state.
    pub fn set_failure(&self, msg: String) {
        if let Ok(mut s) = self.success.lock() {
            *s = Some(false);
        }
        if let Ok(mut m) = self.message.lock() {
            *m = msg.clone();
        }
        if let Ok(mut l) = self.logs.lock() {
            l.push(msg);
        }
    }
}

pub struct TaskResult {
    pub installed_packages: Option<(Vec<Package>, Vec<String>)>,
    pub outdated_packages: Option<(Vec<Package>, Vec<String>)>,
    pub search_results: Option<(Vec<Package>, Vec<String>)>,
    pub package_info: Option<(String, Package)>,
    pub logs: Vec<String>,
    pub completed_package_info_loads: Vec<String>,
    pub install_completed: Option<(bool, String)>,
    pub uninstall_completed: Option<(bool, String)>,
    pub update_completed: Option<(bool, String)>,
    pub update_all_completed: Option<(bool, String)>,
    pub clean_cache_completed: Option<(bool, String)>,
    pub cleanup_old_versions_completed: Option<(bool, String)>,
    pub clean_orphans_completed: Option<(bool, String)>,
    pub pin_completed: Option<(String, bool, String)>,
    pub unpin_completed: Option<(String, bool, String)>,
    pub services: Option<(Vec<Service>, Vec<String>)>,
    pub start_service_completed: Option<(String, bool, String)>,
    pub stop_service_completed: Option<(String, bool, String)>,
    pub restart_service_completed: Option<(String, bool, String)>,
    pub export_packages_completed: Option<(bool, String)>,
    pub import_packages_completed: Option<(bool, String)>,
    pub cleanup_preview_result: Option<(
        crate::presentation::components::CleanupType,
        Result<CleanupPreview, String>,
    )>,
    pub doctor_result: Option<Result<DoctorOutput, String>>,
    pub taps: Option<(Vec<String>, Vec<String>)>,
    pub tap_completed: Option<(bool, String)>,
    pub untap_completed: Option<(bool, String)>,
    pub bundle_dump_completed: Option<(bool, String)>,
    pub bundle_check_preview_result: Option<Result<BrewfileSyncPreview, String>>,
    pub bundle_apply_completed: Option<(bool, String)>,
    pub service_info_result: Option<(String, Result<ServiceInfo, String>)>,
    pub service_log_result: Option<(String, Result<String, String>)>,
}

/// Try to poll a completed success/logs/message task. Returns Some((succeeded, message, logs)) if done.
fn poll_success_task(
    success: &Arc<Mutex<Option<bool>>>,
    logs: &Arc<Mutex<Vec<String>>>,
    message: &Arc<Mutex<String>>,
) -> Option<(bool, String, Vec<String>)> {
    let success_opt = success.try_lock().ok()?;
    let succeeded = (*success_opt)?;
    let log = logs.try_lock().ok()?;
    let msg = message.try_lock().ok()?;
    Some((succeeded, msg.clone(), log.clone()))
}

/// Try to poll a completed data/logs task. Returns Some((data, logs)) if done.
fn poll_data_task<T: Clone>(
    data: &Arc<Mutex<Vec<T>>>,
    logs: &Arc<Mutex<Vec<String>>>,
) -> Option<(Vec<T>, Vec<String>)> {
    let log = logs.try_lock().ok()?;
    if log.is_empty() {
        return None;
    }
    let d = data.try_lock().ok()?;
    Some((d.clone(), log.clone()))
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
        if let Some(kind) = task.kind()
            && self.has_task_kind(kind)
        {
            tracing::warn!("{:?} task is already running, ignoring duplicate", kind);
            return;
        }

        self.active_tasks.push(task);
    }

    pub fn has_task_kind(&self, kind: TaskKind) -> bool {
        self.active_tasks
            .iter()
            .any(|task| task.kind() == Some(kind))
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

        if self
            .pending_package_info_loads
            .iter()
            .any(|(name, _)| name == &package_name)
        {
            tracing::debug!("Already queued for loading: {}", package_name);
            return;
        }

        self.pending_package_info_loads
            .push((package_name, package_type));
    }

    pub fn can_load_more_package_info(&self) -> bool {
        self.packages_loading_info.len() < 15
    }

    pub fn available_package_info_slots(&self) -> usize {
        15usize.saturating_sub(self.packages_loading_info.len())
    }

    pub fn drain_pending_loads(&mut self, count: usize) -> Vec<(String, PackageType)> {
        self.pending_package_info_loads
            .drain(..count.min(self.pending_package_info_loads.len()))
            .collect()
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
            clean_orphans_completed: None,
            pin_completed: None,
            unpin_completed: None,
            services: None,
            start_service_completed: None,
            stop_service_completed: None,
            restart_service_completed: None,
            export_packages_completed: None,
            import_packages_completed: None,
            cleanup_preview_result: None,
            doctor_result: None,
            taps: None,
            tap_completed: None,
            untap_completed: None,
            bundle_dump_completed: None,
            bundle_check_preview_result: None,
            bundle_apply_completed: None,
            service_info_result: None,
            service_log_result: None,
        };

        let mut tasks_to_keep = Vec::new();

        for (pkg_name, task) in self.package_info_tasks.drain(..) {
            if let AsyncTask::LoadPackageInfo {
                package_name,
                package_type,
                result: pkg_result,
                started_at,
            } = task
            {
                let elapsed = started_at.elapsed();

                if elapsed > std::time::Duration::from_secs(10) {
                    tracing::warn!(
                        "Package info loading timed out for {} after {:?}",
                        package_name,
                        elapsed
                    );
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
                            tracing::info!(
                                "Updating search results with package info for {}",
                                package_name_clone
                            );
                            result.package_info = Some((package_name_clone.clone(), package));
                            self.packages_loading_info.remove(&package_name_clone);
                            result.completed_package_info_loads.push(package_name_clone);
                            false
                        } else {
                            true
                        }
                    }
                    Err(_) => true,
                };

                if should_keep {
                    tasks_to_keep.push((
                        pkg_name,
                        AsyncTask::LoadPackageInfo {
                            package_name,
                            package_type,
                            result: pkg_result,
                            started_at,
                        },
                    ));
                }
            }
        }

        self.package_info_tasks = tasks_to_keep;

        let mut active_tasks_to_keep = Vec::new();

        for task in self.active_tasks.drain(..) {
            match task {
                AsyncTask::LoadInstalled { packages, logs } => {
                    if let Some((data, log)) = poll_data_task(&packages, &logs) {
                        result.installed_packages = Some((data, log.clone()));
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::LoadInstalled { packages, logs });
                    }
                }
                AsyncTask::LoadOutdated { packages, logs } => {
                    if let Some((data, log)) = poll_data_task(&packages, &logs) {
                        result.outdated_packages = Some((data, log.clone()));
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::LoadOutdated { packages, logs });
                    }
                }
                AsyncTask::Search { results, logs } => {
                    if let Some((data, log)) = poll_data_task(&results, &logs) {
                        tracing::info!("Search completed, found {} packages", data.len());
                        result.search_results = Some((data, log.clone()));
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::Search { results, logs });
                    }
                }
                AsyncTask::Install {
                    success,
                    logs,
                    message,
                } => {
                    if let Some((ok, msg, log)) = poll_success_task(&success, &logs, &message) {
                        result.install_completed = Some((ok, msg));
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::Install {
                            success,
                            logs,
                            message,
                        });
                    }
                }
                AsyncTask::Uninstall {
                    success,
                    logs,
                    message,
                } => {
                    if let Some((ok, msg, log)) = poll_success_task(&success, &logs, &message) {
                        result.uninstall_completed = Some((ok, msg));
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::Uninstall {
                            success,
                            logs,
                            message,
                        });
                    }
                }
                AsyncTask::Update {
                    success,
                    logs,
                    message,
                } => {
                    if let Some((ok, msg, log)) = poll_success_task(&success, &logs, &message) {
                        result.update_completed = Some((ok, msg));
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::Update {
                            success,
                            logs,
                            message,
                        });
                    }
                }
                AsyncTask::UpdateAll {
                    success,
                    logs,
                    message,
                } => {
                    if let Some((ok, msg, log)) = poll_success_task(&success, &logs, &message) {
                        result.update_all_completed = Some((ok, msg));
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::UpdateAll {
                            success,
                            logs,
                            message,
                        });
                    }
                }
                AsyncTask::CleanCache {
                    success,
                    logs,
                    message,
                } => {
                    if let Some((ok, msg, log)) = poll_success_task(&success, &logs, &message) {
                        result.clean_cache_completed = Some((ok, msg));
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::CleanCache {
                            success,
                            logs,
                            message,
                        });
                    }
                }
                AsyncTask::CleanupOldVersions {
                    success,
                    logs,
                    message,
                } => {
                    if let Some((ok, msg, log)) = poll_success_task(&success, &logs, &message) {
                        result.cleanup_old_versions_completed = Some((ok, msg));
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::CleanupOldVersions {
                            success,
                            logs,
                            message,
                        });
                    }
                }
                AsyncTask::CleanOrphans {
                    success,
                    logs,
                    message,
                } => {
                    if let Some((ok, msg, log)) = poll_success_task(&success, &logs, &message) {
                        result.clean_orphans_completed = Some((ok, msg));
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::CleanOrphans {
                            success,
                            logs,
                            message,
                        });
                    }
                }
                AsyncTask::Pin {
                    package_name,
                    success,
                    logs,
                    message,
                } => {
                    if let Some((ok, msg, log)) = poll_success_task(&success, &logs, &message) {
                        result.pin_completed = Some((package_name, ok, msg));
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::Pin {
                            package_name,
                            success,
                            logs,
                            message,
                        });
                    }
                }
                AsyncTask::Unpin {
                    package_name,
                    success,
                    logs,
                    message,
                } => {
                    if let Some((ok, msg, log)) = poll_success_task(&success, &logs, &message) {
                        result.unpin_completed = Some((package_name, ok, msg));
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::Unpin {
                            package_name,
                            success,
                            logs,
                            message,
                        });
                    }
                }
                AsyncTask::LoadServices { services, logs } => {
                    if let Some((data, log)) = poll_data_task(&services, &logs) {
                        result.services = Some((data, log.clone()));
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::LoadServices { services, logs });
                    }
                }
                AsyncTask::StartService {
                    service_name,
                    success,
                    logs,
                    message,
                } => {
                    if let Some((ok, msg, log)) = poll_success_task(&success, &logs, &message) {
                        result.start_service_completed = Some((service_name, ok, msg));
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::StartService {
                            service_name,
                            success,
                            logs,
                            message,
                        });
                    }
                }
                AsyncTask::StopService {
                    service_name,
                    success,
                    logs,
                    message,
                } => {
                    if let Some((ok, msg, log)) = poll_success_task(&success, &logs, &message) {
                        result.stop_service_completed = Some((service_name, ok, msg));
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::StopService {
                            service_name,
                            success,
                            logs,
                            message,
                        });
                    }
                }
                AsyncTask::RestartService {
                    service_name,
                    success,
                    logs,
                    message,
                } => {
                    if let Some((ok, msg, log)) = poll_success_task(&success, &logs, &message) {
                        result.restart_service_completed = Some((service_name, ok, msg));
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::RestartService {
                            service_name,
                            success,
                            logs,
                            message,
                        });
                    }
                }
                AsyncTask::ExportPackages {
                    success,
                    logs,
                    message,
                } => {
                    if let Some((ok, msg, log)) = poll_success_task(&success, &logs, &message) {
                        result.export_packages_completed = Some((ok, msg));
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::ExportPackages {
                            success,
                            logs,
                            message,
                        });
                    }
                }
                AsyncTask::ImportPackages {
                    success,
                    logs,
                    message,
                } => {
                    if let Some((ok, msg, log)) = poll_success_task(&success, &logs, &message) {
                        result.import_packages_completed = Some((ok, msg));
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::ImportPackages {
                            success,
                            logs,
                            message,
                        });
                    }
                }
                AsyncTask::CleanupPreview {
                    cleanup_type,
                    preview,
                    error,
                } => {
                    let done = if let Ok(err) = error.try_lock() {
                        if let Some(err_msg) = err.as_ref() {
                            result.cleanup_preview_result =
                                Some((cleanup_type, Err(err_msg.clone())));
                            true
                        } else if let Ok(prev) = preview.try_lock() {
                            if let Some(p) = prev.as_ref() {
                                result.cleanup_preview_result = Some((cleanup_type, Ok(p.clone())));
                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    } else {
                        false
                    };
                    if !done {
                        active_tasks_to_keep.push(AsyncTask::CleanupPreview {
                            cleanup_type,
                            preview,
                            error,
                        });
                    }
                }
                AsyncTask::Doctor {
                    result: doc_result,
                    error,
                } => {
                    let done = if let Ok(err) = error.try_lock() {
                        if let Some(err_msg) = err.as_ref() {
                            result.doctor_result = Some(Err(err_msg.clone()));
                            true
                        } else if let Ok(res) = doc_result.try_lock() {
                            if let Some(r) = res.as_ref() {
                                result.doctor_result = Some(Ok(r.clone()));
                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    } else {
                        false
                    };
                    if !done {
                        active_tasks_to_keep.push(AsyncTask::Doctor {
                            result: doc_result,
                            error,
                        });
                    }
                }
                AsyncTask::LoadTaps { taps, logs } => {
                    if let Some((data, log)) = poll_data_task(&taps, &logs) {
                        result.taps = Some((data, log.clone()));
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::LoadTaps { taps, logs });
                    }
                }
                AsyncTask::Tap {
                    success,
                    logs,
                    message,
                } => {
                    if let Some((ok, msg, log)) = poll_success_task(&success, &logs, &message) {
                        result.tap_completed = Some((ok, msg));
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::Tap {
                            success,
                            logs,
                            message,
                        });
                    }
                }
                AsyncTask::Untap {
                    success,
                    logs,
                    message,
                } => {
                    if let Some((ok, msg, log)) = poll_success_task(&success, &logs, &message) {
                        result.untap_completed = Some((ok, msg));
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::Untap {
                            success,
                            logs,
                            message,
                        });
                    }
                }
                AsyncTask::BundleDump {
                    success,
                    logs,
                    message,
                } => {
                    if let Some((ok, msg, log)) = poll_success_task(&success, &logs, &message) {
                        result.bundle_dump_completed = Some((ok, msg));
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::BundleDump {
                            success,
                            logs,
                            message,
                        });
                    }
                }
                AsyncTask::BundleCheckPreview { preview, error } => {
                    let done = if let Ok(err) = error.try_lock() {
                        if let Some(err_msg) = err.as_ref() {
                            result.bundle_check_preview_result = Some(Err(err_msg.clone()));
                            true
                        } else if let Ok(prev) = preview.try_lock() {
                            if let Some(p) = prev.as_ref() {
                                result.bundle_check_preview_result = Some(Ok(p.clone()));
                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    } else {
                        false
                    };
                    if !done {
                        active_tasks_to_keep.push(AsyncTask::BundleCheckPreview { preview, error });
                    }
                }
                AsyncTask::BundleApply {
                    success,
                    logs,
                    message,
                } => {
                    if let Some((ok, msg, log)) = poll_success_task(&success, &logs, &message) {
                        result.bundle_apply_completed = Some((ok, msg));
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::BundleApply {
                            success,
                            logs,
                            message,
                        });
                    }
                }
                AsyncTask::ServiceInfoLoad {
                    service_name,
                    result: info_result,
                    error,
                } => {
                    let done = if let Ok(err) = error.try_lock() {
                        if let Some(err_msg) = err.as_ref() {
                            result.service_info_result =
                                Some((service_name.clone(), Err(err_msg.clone())));
                            true
                        } else if let Ok(res) = info_result.try_lock() {
                            if let Some(info) = res.as_ref() {
                                result.service_info_result =
                                    Some((service_name.clone(), Ok(info.clone())));
                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    } else {
                        false
                    };
                    if !done {
                        active_tasks_to_keep.push(AsyncTask::ServiceInfoLoad {
                            service_name,
                            result: info_result,
                            error,
                        });
                    }
                }
                AsyncTask::ServiceLogLoad {
                    service_name,
                    result: log_result,
                    error,
                } => {
                    let done = if let Ok(err) = error.try_lock() {
                        if let Some(err_msg) = err.as_ref() {
                            result.service_log_result =
                                Some((service_name.clone(), Err(err_msg.clone())));
                            true
                        } else if let Ok(res) = log_result.try_lock() {
                            if let Some(log_text) = res.as_ref() {
                                result.service_log_result =
                                    Some((service_name.clone(), Ok(log_text.clone())));
                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    } else {
                        false
                    };
                    if !done {
                        active_tasks_to_keep.push(AsyncTask::ServiceLogLoad {
                            service_name,
                            result: log_result,
                            error,
                        });
                    }
                }
                AsyncTask::LoadPackageInfo { .. } => {}
            }
        }

        self.active_tasks = active_tasks_to_keep;

        result
    }
}

impl AsyncTask {
    pub fn kind(&self) -> Option<TaskKind> {
        match self {
            AsyncTask::LoadInstalled { .. } => Some(TaskKind::LoadInstalled),
            AsyncTask::LoadOutdated { .. } => Some(TaskKind::LoadOutdated),
            AsyncTask::Search { .. } => Some(TaskKind::Search),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn available_package_info_slots_saturates_at_zero() {
        let mut manager = AsyncTaskManager::new();
        for i in 0..20 {
            manager.packages_loading_info.insert(format!("pkg-{i}"));
        }

        assert_eq!(manager.available_package_info_slots(), 0);
    }

    #[test]
    fn available_package_info_slots_reflects_remaining_capacity() {
        let mut manager = AsyncTaskManager::new();
        for i in 0..4 {
            manager.packages_loading_info.insert(format!("pkg-{i}"));
        }

        assert_eq!(manager.available_package_info_slots(), 11);
    }

    #[test]
    fn queue_package_info_load_deduplicates_entries() {
        let mut manager = AsyncTaskManager::new();

        manager.queue_package_info_load("wget".to_string(), PackageType::Formula);
        manager.queue_package_info_load("wget".to_string(), PackageType::Formula);

        assert_eq!(manager.pending_loads_count(), 1);
    }
}
