use crate::domain::entities::brewfile::BrewfileSyncPreview;
use crate::domain::entities::{
    AppError, CleanupPreview, DoctorOutput, LoadState, Package, PackageType, Service, ServiceInfo,
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
        state: Arc<Mutex<Option<LoadState<Vec<Package>>>>>,
        logs: Arc<Mutex<Vec<String>>>,
    },
    LoadOutdated {
        state: Arc<Mutex<Option<LoadState<Vec<Package>>>>>,
        logs: Arc<Mutex<Vec<String>>>,
    },
    Search {
        state: Arc<Mutex<Option<LoadState<Vec<Package>>>>>,
        logs: Arc<Mutex<Vec<String>>>,
    },
    LoadPackageInfo {
        package_name: String,
        package_type: PackageType,
        result: Arc<Mutex<Option<Package>>>,
        started_at: std::time::Instant,
    },
    Install {
        result: Arc<Mutex<Option<Result<String, AppError>>>>,
        logs: Arc<Mutex<Vec<String>>>,
    },
    Uninstall {
        result: Arc<Mutex<Option<Result<String, AppError>>>>,
        logs: Arc<Mutex<Vec<String>>>,
    },
    Update {
        result: Arc<Mutex<Option<Result<String, AppError>>>>,
        logs: Arc<Mutex<Vec<String>>>,
    },
    UpdateAll {
        result: Arc<Mutex<Option<Result<String, AppError>>>>,
        logs: Arc<Mutex<Vec<String>>>,
    },
    CleanCache {
        result: Arc<Mutex<Option<Result<String, AppError>>>>,
        logs: Arc<Mutex<Vec<String>>>,
    },
    CleanupOldVersions {
        result: Arc<Mutex<Option<Result<String, AppError>>>>,
        logs: Arc<Mutex<Vec<String>>>,
    },
    CleanOrphans {
        result: Arc<Mutex<Option<Result<String, AppError>>>>,
        logs: Arc<Mutex<Vec<String>>>,
    },
    Pin {
        package_name: String,
        result: Arc<Mutex<Option<Result<String, AppError>>>>,
        logs: Arc<Mutex<Vec<String>>>,
    },
    Unpin {
        package_name: String,
        result: Arc<Mutex<Option<Result<String, AppError>>>>,
        logs: Arc<Mutex<Vec<String>>>,
    },
    LoadServices {
        state: Arc<Mutex<Option<LoadState<Vec<Service>>>>>,
        logs: Arc<Mutex<Vec<String>>>,
    },
    StartService {
        service_name: String,
        result: Arc<Mutex<Option<Result<String, AppError>>>>,
        logs: Arc<Mutex<Vec<String>>>,
    },
    StopService {
        service_name: String,
        result: Arc<Mutex<Option<Result<String, AppError>>>>,
        logs: Arc<Mutex<Vec<String>>>,
    },
    RestartService {
        service_name: String,
        result: Arc<Mutex<Option<Result<String, AppError>>>>,
        logs: Arc<Mutex<Vec<String>>>,
    },
    ExportPackages {
        result: Arc<Mutex<Option<Result<String, AppError>>>>,
        logs: Arc<Mutex<Vec<String>>>,
    },
    ImportPackages {
        result: Arc<Mutex<Option<Result<String, AppError>>>>,
        logs: Arc<Mutex<Vec<String>>>,
    },
    CheckUpdates {
        result: Arc<Mutex<Option<Result<String, AppError>>>>,
        logs: Arc<Mutex<Vec<String>>>,
    },
    CleanupPreview {
        cleanup_type: crate::presentation::components::CleanupType,
        result: Arc<Mutex<Option<Result<CleanupPreview, AppError>>>>,
    },
    Doctor {
        result: Arc<Mutex<Option<Result<DoctorOutput, AppError>>>>,
    },
    LoadTaps {
        state: Arc<Mutex<Option<LoadState<Vec<String>>>>>,
        logs: Arc<Mutex<Vec<String>>>,
    },
    Tap {
        result: Arc<Mutex<Option<Result<String, AppError>>>>,
        logs: Arc<Mutex<Vec<String>>>,
    },
    Untap {
        result: Arc<Mutex<Option<Result<String, AppError>>>>,
        logs: Arc<Mutex<Vec<String>>>,
    },
    BundleDump {
        result: Arc<Mutex<Option<Result<String, AppError>>>>,
        logs: Arc<Mutex<Vec<String>>>,
    },
    BundleCheckPreview {
        result: Arc<Mutex<Option<Result<BrewfileSyncPreview, AppError>>>>,
    },
    BundleApply {
        result: Arc<Mutex<Option<Result<String, AppError>>>>,
        logs: Arc<Mutex<Vec<String>>>,
    },
    ServiceInfoLoad {
        service_name: String,
        result: Arc<Mutex<Option<Result<ServiceInfo, AppError>>>>,
    },
    ServiceLogLoad {
        service_name: String,
        result: Arc<Mutex<Option<Result<String, AppError>>>>,
    },
}

/// Shared state for async tasks that produce a single structured result.
pub struct TaskSharedState {
    pub result: Arc<Mutex<Option<Result<String, AppError>>>>,
    pub logs: Arc<Mutex<Vec<String>>>,
}

impl TaskSharedState {
    pub fn new() -> Self {
        Self {
            result: Arc::new(Mutex::new(None)),
            logs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Write a successful result into the shared state.
    pub fn set_success(&self, msg: String) {
        if let Ok(mut result) = self.result.lock() {
            *result = Some(Ok(msg.clone()));
        }
        if let Ok(mut l) = self.logs.lock() {
            l.push(msg);
        }
    }

    /// Write a failure result into the shared state.
    pub fn set_failure(&self, error: AppError) {
        if let Ok(mut result) = self.result.lock() {
            *result = Some(Err(error.clone()));
        }
        if let Ok(mut l) = self.logs.lock() {
            l.push(error.short_message());
        }
    }
}

pub struct LoadTaskSharedState<T> {
    pub state: Arc<Mutex<Option<LoadState<T>>>>,
    pub logs: Arc<Mutex<Vec<String>>>,
}

impl<T> LoadTaskSharedState<T> {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(None)),
            logs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn set_state(&self, state: LoadState<T>)
    where
        T: Clone,
    {
        if let Ok(mut result) = self.state.lock() {
            *result = Some(state);
        }
    }

    pub fn push_log(&self, message: String) {
        if let Ok(mut logs) = self.logs.lock() {
            logs.push(message);
        }
    }
}

pub struct TaskResult {
    pub installed_packages: Option<LoadState<Vec<Package>>>,
    pub outdated_packages: Option<LoadState<Vec<Package>>>,
    pub search_results: Option<LoadState<Vec<Package>>>,
    pub package_info: Option<(String, Package)>,
    pub logs: Vec<String>,
    pub completed_package_info_loads: Vec<String>,
    pub install_completed: Option<Result<String, AppError>>,
    pub uninstall_completed: Option<Result<String, AppError>>,
    pub update_completed: Option<Result<String, AppError>>,
    pub update_all_completed: Option<Result<String, AppError>>,
    pub clean_cache_completed: Option<Result<String, AppError>>,
    pub cleanup_old_versions_completed: Option<Result<String, AppError>>,
    pub clean_orphans_completed: Option<Result<String, AppError>>,
    pub pin_completed: Option<(String, Result<String, AppError>)>,
    pub unpin_completed: Option<(String, Result<String, AppError>)>,
    pub services: Option<LoadState<Vec<Service>>>,
    pub start_service_completed: Option<(String, Result<String, AppError>)>,
    pub stop_service_completed: Option<(String, Result<String, AppError>)>,
    pub restart_service_completed: Option<(String, Result<String, AppError>)>,
    pub export_packages_completed: Option<Result<String, AppError>>,
    pub import_packages_completed: Option<Result<String, AppError>>,
    pub cleanup_preview_result: Option<(
        crate::presentation::components::CleanupType,
        Result<CleanupPreview, AppError>,
    )>,
    pub doctor_result: Option<Result<DoctorOutput, AppError>>,
    pub taps: Option<LoadState<Vec<String>>>,
    pub tap_completed: Option<Result<String, AppError>>,
    pub untap_completed: Option<Result<String, AppError>>,
    pub bundle_dump_completed: Option<Result<String, AppError>>,
    pub bundle_check_preview_result: Option<Result<BrewfileSyncPreview, AppError>>,
    pub bundle_apply_completed: Option<Result<String, AppError>>,
    pub service_info_result: Option<(String, Result<ServiceInfo, AppError>)>,
    pub service_log_result: Option<(String, Result<String, AppError>)>,
}

/// Try to poll a completed result/logs task.
fn poll_result_task<T: Clone>(
    result: &Arc<Mutex<Option<Result<T, AppError>>>>,
    logs: &Arc<Mutex<Vec<String>>>,
) -> Option<(Result<T, AppError>, Vec<String>)> {
    let result = result.try_lock().ok()?.clone()?;
    let logs = logs.try_lock().ok()?.clone();
    Some((result, logs))
}

/// Try to poll a completed load-state/logs task.
fn poll_load_state_task<T: Clone>(
    state: &Arc<Mutex<Option<LoadState<T>>>>,
    logs: &Arc<Mutex<Vec<String>>>,
) -> Option<(LoadState<T>, Vec<String>)> {
    let state = state.try_lock().ok()?.clone()?;
    let logs = logs.try_lock().ok()?.clone();
    Some((state, logs))
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
                AsyncTask::LoadInstalled { state, logs } => {
                    if let Some((state_result, log)) = poll_load_state_task(&state, &logs) {
                        result.installed_packages = Some(state_result);
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::LoadInstalled { state, logs });
                    }
                }
                AsyncTask::LoadOutdated { state, logs } => {
                    if let Some((state_result, log)) = poll_load_state_task(&state, &logs) {
                        result.outdated_packages = Some(state_result);
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::LoadOutdated { state, logs });
                    }
                }
                AsyncTask::Search { state, logs } => {
                    if let Some((state_result, log)) = poll_load_state_task(&state, &logs) {
                        tracing::info!("Search task completed");
                        result.search_results = Some(state_result);
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::Search { state, logs });
                    }
                }
                AsyncTask::Install {
                    result: task_result,
                    logs,
                } => {
                    if let Some((task_result, log)) = poll_result_task(&task_result, &logs) {
                        result.install_completed = Some(task_result);
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::Install {
                            result: task_result,
                            logs,
                        });
                    }
                }
                AsyncTask::Uninstall {
                    result: task_result,
                    logs,
                } => {
                    if let Some((task_result, log)) = poll_result_task(&task_result, &logs) {
                        result.uninstall_completed = Some(task_result);
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::Uninstall {
                            result: task_result,
                            logs,
                        });
                    }
                }
                AsyncTask::Update {
                    result: task_result,
                    logs,
                } => {
                    if let Some((task_result, log)) = poll_result_task(&task_result, &logs) {
                        result.update_completed = Some(task_result);
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::Update {
                            result: task_result,
                            logs,
                        });
                    }
                }
                AsyncTask::UpdateAll {
                    result: task_result,
                    logs,
                } => {
                    if let Some((task_result, log)) = poll_result_task(&task_result, &logs) {
                        result.update_all_completed = Some(task_result);
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::UpdateAll {
                            result: task_result,
                            logs,
                        });
                    }
                }
                AsyncTask::CleanCache {
                    result: task_result,
                    logs,
                } => {
                    if let Some((task_result, log)) = poll_result_task(&task_result, &logs) {
                        result.clean_cache_completed = Some(task_result);
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::CleanCache {
                            result: task_result,
                            logs,
                        });
                    }
                }
                AsyncTask::CleanupOldVersions {
                    result: task_result,
                    logs,
                } => {
                    if let Some((task_result, log)) = poll_result_task(&task_result, &logs) {
                        result.cleanup_old_versions_completed = Some(task_result);
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::CleanupOldVersions {
                            result: task_result,
                            logs,
                        });
                    }
                }
                AsyncTask::CleanOrphans {
                    result: task_result,
                    logs,
                } => {
                    if let Some((task_result, log)) = poll_result_task(&task_result, &logs) {
                        result.clean_orphans_completed = Some(task_result);
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::CleanOrphans {
                            result: task_result,
                            logs,
                        });
                    }
                }
                AsyncTask::Pin {
                    package_name,
                    result: task_result,
                    logs,
                } => {
                    if let Some((task_result, log)) = poll_result_task(&task_result, &logs) {
                        result.pin_completed = Some((package_name, task_result));
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::Pin {
                            package_name,
                            result: task_result,
                            logs,
                        });
                    }
                }
                AsyncTask::Unpin {
                    package_name,
                    result: task_result,
                    logs,
                } => {
                    if let Some((task_result, log)) = poll_result_task(&task_result, &logs) {
                        result.unpin_completed = Some((package_name, task_result));
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::Unpin {
                            package_name,
                            result: task_result,
                            logs,
                        });
                    }
                }
                AsyncTask::LoadServices { state, logs } => {
                    if let Some((state_result, log)) = poll_load_state_task(&state, &logs) {
                        result.services = Some(state_result);
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::LoadServices { state, logs });
                    }
                }
                AsyncTask::StartService {
                    service_name,
                    result: task_result,
                    logs,
                } => {
                    if let Some((task_result, log)) = poll_result_task(&task_result, &logs) {
                        result.start_service_completed = Some((service_name, task_result));
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::StartService {
                            service_name,
                            result: task_result,
                            logs,
                        });
                    }
                }
                AsyncTask::StopService {
                    service_name,
                    result: task_result,
                    logs,
                } => {
                    if let Some((task_result, log)) = poll_result_task(&task_result, &logs) {
                        result.stop_service_completed = Some((service_name, task_result));
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::StopService {
                            service_name,
                            result: task_result,
                            logs,
                        });
                    }
                }
                AsyncTask::RestartService {
                    service_name,
                    result: task_result,
                    logs,
                } => {
                    if let Some((task_result, log)) = poll_result_task(&task_result, &logs) {
                        result.restart_service_completed = Some((service_name, task_result));
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::RestartService {
                            service_name,
                            result: task_result,
                            logs,
                        });
                    }
                }
                AsyncTask::ExportPackages {
                    result: task_result,
                    logs,
                } => {
                    if let Some((task_result, log)) = poll_result_task(&task_result, &logs) {
                        result.export_packages_completed = Some(task_result);
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::ExportPackages {
                            result: task_result,
                            logs,
                        });
                    }
                }
                AsyncTask::ImportPackages {
                    result: task_result,
                    logs,
                } => {
                    if let Some((task_result, log)) = poll_result_task(&task_result, &logs) {
                        result.import_packages_completed = Some(task_result);
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::ImportPackages {
                            result: task_result,
                            logs,
                        });
                    }
                }
                AsyncTask::CleanupPreview {
                    cleanup_type,
                    result: task_result,
                } => {
                    let done = if let Ok(task_result) = task_result.try_lock() {
                        if let Some(task_result) = task_result.as_ref() {
                            result.cleanup_preview_result =
                                Some((cleanup_type, task_result.clone()));
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    };
                    if !done {
                        active_tasks_to_keep.push(AsyncTask::CleanupPreview {
                            cleanup_type,
                            result: task_result,
                        });
                    }
                }
                AsyncTask::Doctor { result: doc_result } => {
                    let done = if let Ok(doc_result) = doc_result.try_lock() {
                        if let Some(doc_result) = doc_result.as_ref() {
                            result.doctor_result = Some(doc_result.clone());
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    };
                    if !done {
                        active_tasks_to_keep.push(AsyncTask::Doctor { result: doc_result });
                    }
                }
                AsyncTask::LoadTaps { state, logs } => {
                    if let Some((state_result, log)) = poll_load_state_task(&state, &logs) {
                        result.taps = Some(state_result);
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::LoadTaps { state, logs });
                    }
                }
                AsyncTask::Tap {
                    result: task_result,
                    logs,
                } => {
                    if let Some((task_result, log)) = poll_result_task(&task_result, &logs) {
                        result.tap_completed = Some(task_result);
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::Tap {
                            result: task_result,
                            logs,
                        });
                    }
                }
                AsyncTask::Untap {
                    result: task_result,
                    logs,
                } => {
                    if let Some((task_result, log)) = poll_result_task(&task_result, &logs) {
                        result.untap_completed = Some(task_result);
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::Untap {
                            result: task_result,
                            logs,
                        });
                    }
                }
                AsyncTask::BundleDump {
                    result: task_result,
                    logs,
                } => {
                    if let Some((task_result, log)) = poll_result_task(&task_result, &logs) {
                        result.bundle_dump_completed = Some(task_result);
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::BundleDump {
                            result: task_result,
                            logs,
                        });
                    }
                }
                AsyncTask::BundleCheckPreview {
                    result: task_result,
                } => {
                    let done = if let Ok(task_result) = task_result.try_lock() {
                        if let Some(task_result) = task_result.as_ref() {
                            result.bundle_check_preview_result = Some(task_result.clone());
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    };
                    if !done {
                        active_tasks_to_keep.push(AsyncTask::BundleCheckPreview {
                            result: task_result,
                        });
                    }
                }
                AsyncTask::BundleApply {
                    result: task_result,
                    logs,
                } => {
                    if let Some((task_result, log)) = poll_result_task(&task_result, &logs) {
                        result.bundle_apply_completed = Some(task_result);
                        result.logs.extend(log);
                    } else {
                        active_tasks_to_keep.push(AsyncTask::BundleApply {
                            result: task_result,
                            logs,
                        });
                    }
                }
                AsyncTask::ServiceInfoLoad {
                    service_name,
                    result: info_result,
                } => {
                    let done = if let Ok(info_result) = info_result.try_lock() {
                        if let Some(info_result) = info_result.as_ref() {
                            result.service_info_result =
                                Some((service_name.clone(), info_result.clone()));
                            true
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
                        });
                    }
                }
                AsyncTask::ServiceLogLoad {
                    service_name,
                    result: log_result,
                } => {
                    let done = if let Ok(log_result) = log_result.try_lock() {
                        if let Some(log_result) = log_result.as_ref() {
                            result.service_log_result =
                                Some((service_name.clone(), log_result.clone()));
                            true
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
                        });
                    }
                }
                AsyncTask::LoadPackageInfo { .. } => {}
                AsyncTask::CheckUpdates { .. } => {}
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
