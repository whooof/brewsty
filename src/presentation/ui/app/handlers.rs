use crate::domain::entities::OperationType;
use crate::domain::entities::{AppError, LoadState, Package, PackageType};
use crate::presentation::components::CleanupType;
use crate::presentation::services::{AsyncTask, LoadTaskSharedState, TaskSharedState};
use crate::presentation::ui::tabs::history::UndoRequest;
use std::sync::{Arc, Mutex};

use super::BrewstyApp;

impl BrewstyApp {
    pub(super) fn maybe_confirm_install(&mut self, package: Package) {
        if self.config.confirm_before_actions {
            self.confirm_action = Some(super::ConfirmAction::Install(package));
        } else {
            self.handle_install(package);
        }
    }

    pub(super) fn maybe_confirm_uninstall(&mut self, package: Package) {
        if self.config.confirm_before_actions {
            self.confirm_action = Some(super::ConfirmAction::Uninstall(package));
        } else {
            self.handle_uninstall(package);
        }
    }

    pub(super) fn maybe_confirm_update(&mut self, package: Package) {
        if self.config.confirm_before_actions {
            self.confirm_action = Some(super::ConfirmAction::Update(package));
        } else {
            self.handle_update(package);
        }
    }

    pub(super) fn load_installed_packages(&mut self, include_outdated: bool) {
        if self.loading_installed || self.loading_outdated {
            return;
        }

        self.loading_installed = true;
        self.installed_state = LoadState::Loading;
        if include_outdated {
            self.loading_outdated = true;
            self.outdated_state = LoadState::Loading;
        }
        self.installed_message = None;
        self.status_message = if include_outdated {
            "Loading installed and outdated packages...".to_string()
        } else {
            "Loading installed packages...".to_string()
        };
        self.set_operation_running("Loading packages", None);

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

        let installed_state = LoadTaskSharedState::new();
        let outdated_state = LoadTaskSharedState::new();

        self.task_manager.set_active_task(AsyncTask::LoadInstalled {
            state: Arc::clone(&installed_state.state),
            logs: Arc::clone(&installed_state.logs),
        });

        if include_outdated {
            self.task_manager.set_active_task(AsyncTask::LoadOutdated {
                state: Arc::clone(&outdated_state.state),
                logs: Arc::clone(&outdated_state.logs),
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
                let installed_formulae_error = installed_formulae_result
                    .as_ref()
                    .err()
                    .map(|error| AppError::classify_message(error.to_string()));
                let installed_casks_error = installed_casks_result
                    .as_ref()
                    .err()
                    .map(|error| AppError::classify_message(error.to_string()));
                let outdated_formulae_error = outdated_formulae_result
                    .as_ref()
                    .err()
                    .map(|error| AppError::classify_message(error.to_string()));
                let outdated_casks_error = outdated_casks_result
                    .as_ref()
                    .err()
                    .map(|error| AppError::classify_message(error.to_string()));

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

                installed_logs_vec.push("Finished loading installed packages".to_string());
                if include_outdated {
                    outdated_logs_vec.push("Finished loading outdated packages".to_string());
                    tracing::info!("Finished loading installed and outdated packages");
                } else {
                    tracing::info!("Finished loading installed packages");
                }

                for log in installed_logs_vec {
                    installed_state.push_log(log);
                }
                for log in outdated_logs_vec {
                    outdated_state.push_log(log);
                }

                let installed_load_state = match (installed_formulae_error, installed_casks_error) {
                    (None, None) => LoadState::Ready(installed),
                    (Some(error), None) | (None, Some(error)) => LoadState::Partial {
                        data: installed,
                        warning: error,
                    },
                    (Some(left), Some(right)) => LoadState::Error(AppError::Unknown(format!(
                        "{}; {}",
                        left.short_message(),
                        right.short_message()
                    ))),
                };
                installed_state.set_state(installed_load_state);

                if include_outdated {
                    let outdated_load_state =
                        match (outdated_formulae_error, outdated_casks_error) {
                            (None, None) => LoadState::Ready(outdated),
                            (Some(error), None) | (None, Some(error)) => LoadState::Partial {
                                data: outdated,
                                warning: error,
                            },
                            (Some(left), Some(right)) => LoadState::Error(AppError::Unknown(
                                format!("{}; {}", left.short_message(), right.short_message()),
                            )),
                        };
                    outdated_state.set_state(outdated_load_state);
                }

                tracing::debug!("Successfully updated mutexes");

                Ok::<(), anyhow::Error>(())
            }
            .await;

            if let Err(e) = task_result {
                tracing::error!("Error in load_installed_packages task: {}", e);
                installed_state.push_log(format!("Task error: {}", e));
                installed_state.set_state(LoadState::Error(AppError::from_anyhow(e)));
            }
            tracing::trace!("TASK ENDED: load_installed_packages");
        });
    }

    pub(super) fn handle_update_selected(&mut self, package_names: Vec<String>) {
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

        self.pending_updates = packages_to_update;
        self.loading_update_all = true;

        self.process_next_pending_update();
    }

    pub(super) fn process_next_pending_update(&mut self) {
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

    pub(super) fn handle_install(&mut self, package: Package) {
        if self.loading_install {
            return;
        }

        let package_name = package.name.clone();
        self.loading_install = true;
        self.set_operation_running("Installing", Some(package_name.clone()));
        self.current_install_package = Some(package_name.clone());
        self.packages_in_operation.insert(package_name.clone());
        self.status_message = format!("Installing {}...", package.name);

        let initial_msg = format!(
            "Installing package: {} ({:?})",
            package_name, package.package_type
        );
        self.log_manager.push(initial_msg.clone());
        tracing::info!("{}", initial_msg);

        let shared = TaskSharedState::new();

        self.task_manager.set_active_task(AsyncTask::Install {
            result: Arc::clone(&shared.result),
            logs: Arc::clone(&shared.logs),
        });

        let use_case = Arc::clone(&self.use_cases.install);

        self.executor.spawn(async move {
            match use_case.execute(package).await {
                Ok(_) => shared.set_success(format!("Successfully installed {}", package_name)),
                Err(e) => shared.set_failure(AppError::from_anyhow(e)),
            }
        });
    }

    pub(super) fn handle_uninstall(&mut self, package: Package) {
        if self.loading_uninstall {
            return;
        }

        let package_name = package.name.clone();
        self.loading_uninstall = true;
        self.set_operation_running("Uninstalling", Some(package_name.clone()));
        self.current_uninstall_package = Some(package_name.clone());
        self.packages_in_operation.insert(package_name.clone());
        self.status_message = format!("Uninstalling {}...", package.name);

        let initial_msg = format!(
            "Uninstalling package: {} ({:?})",
            package_name, package.package_type
        );
        self.log_manager.push(initial_msg.clone());
        tracing::info!("{}", initial_msg);

        let shared = TaskSharedState::new();

        self.task_manager.set_active_task(AsyncTask::Uninstall {
            result: Arc::clone(&shared.result),
            logs: Arc::clone(&shared.logs),
        });

        let use_case = Arc::clone(&self.use_cases.uninstall);

        self.executor.spawn(async move {
            match use_case.execute(package).await {
                Ok(_) => shared.set_success(format!("Successfully uninstalled {}", package_name)),
                Err(e) => shared.set_failure(AppError::from_anyhow(e)),
            }
        });
    }

    pub(super) fn handle_update(&mut self, package: Package) {
        if self.loading_update {
            return;
        }

        let package_name = package.name.clone();
        self.loading_update = true;
        self.set_operation_running("Updating", Some(package_name.clone()));
        self.current_update_package = Some(package_name.clone());
        self.packages_in_operation.insert(package_name.clone());
        self.status_message = format!("Updating {}...", package.name);

        let initial_msg = format!(
            "Updating package: {} ({:?})",
            package_name, package.package_type
        );
        self.log_manager.push(initial_msg.clone());
        tracing::info!("{}", initial_msg);

        let shared = TaskSharedState::new();

        self.task_manager.set_active_task(AsyncTask::Update {
            result: Arc::clone(&shared.result),
            logs: Arc::clone(&shared.logs),
        });

        let use_case = Arc::clone(&self.use_cases.update);

        self.executor.spawn(async move {
            match use_case.execute(&package).await {
                Ok(_) => shared.set_success(format!("Successfully updated {}", package_name)),
                Err(e) => shared.set_failure(AppError::from_anyhow(e)),
            }
        });
    }

    pub(super) fn handle_pin(&mut self, package: Package) {
        self.set_operation_running("Pinning", Some(package.name.clone()));
        self.packages_in_operation.insert(package.name.clone());
        self.status_message = format!("Pinning {}...", package.name);

        let package_name = package.name.clone();
        let initial_msg = format!(
            "Pinning package: {} ({:?})",
            package_name, package.package_type
        );
        self.log_manager.push(initial_msg.clone());
        tracing::info!("{}", initial_msg);

        let shared = TaskSharedState::new();

        self.task_manager.set_active_task(AsyncTask::Pin {
            package_name: package.name.clone(),
            result: Arc::clone(&shared.result),
            logs: Arc::clone(&shared.logs),
        });

        let use_case = Arc::clone(&self.use_cases.pin);

        self.executor.spawn(async move {
            match use_case.execute(package).await {
                Ok(_) => shared.set_success(format!("Successfully pinned {}", package_name)),
                Err(e) => shared.set_failure(AppError::from_anyhow(e)),
            }
        });
    }

    pub(super) fn handle_unpin(&mut self, package: Package) {
        self.set_operation_running("Unpinning", Some(package.name.clone()));
        self.packages_in_operation.insert(package.name.clone());
        self.status_message = format!("Unpinning {}...", package.name);

        let package_name = package.name.clone();
        let initial_msg = format!(
            "Unpinning package: {} ({:?})",
            package_name, package.package_type
        );
        self.log_manager.push(initial_msg.clone());
        tracing::info!("{}", initial_msg);

        let shared = TaskSharedState::new();

        self.task_manager.set_active_task(AsyncTask::Unpin {
            package_name: package.name.clone(),
            result: Arc::clone(&shared.result),
            logs: Arc::clone(&shared.logs),
        });

        let use_case = Arc::clone(&self.use_cases.unpin);

        self.executor.spawn(async move {
            match use_case.execute(package).await {
                Ok(_) => shared.set_success(format!("Successfully unpinned {}", package_name)),
                Err(e) => shared.set_failure(AppError::from_anyhow(e)),
            }
        });
    }

    pub(super) fn load_services(&mut self) {
        if self.loading_services {
            return;
        }

        self.loading_services = true;
        self.services_state = LoadState::Loading;
        self.services_message = None;
        self.set_operation_running("Loading services", None);
        self.status_message = "Loading services...".to_string();
        self.log_manager.push("Loading brew services".to_string());
        tracing::info!("Loading brew services");

        let use_case = Arc::clone(&self.use_cases.list_services);

        let shared = LoadTaskSharedState::new();

        self.task_manager.set_active_task(AsyncTask::LoadServices {
            state: Arc::clone(&shared.state),
            logs: Arc::clone(&shared.logs),
        });

        self.executor.spawn(async move {
            match use_case.execute().await {
                Ok(service_list) => {
                    let msg = format!("Loaded {} services", service_list.len());
                    tracing::info!("{}", msg);
                    shared.push_log(msg);
                    shared.set_state(LoadState::Ready(service_list));
                }
                Err(e) => {
                    let msg = format!("Error loading services: {}", e);
                    tracing::error!("{}", msg);
                    shared.push_log(msg);
                    shared.set_state(LoadState::Error(AppError::from_anyhow(e)));
                }
            }
        });
    }

    pub(super) fn handle_start_service(&mut self, service_name: String) {
        self.services_in_operation.insert(service_name.clone());
        self.set_operation_running("Starting service", Some(service_name.clone()));
        self.status_message = format!("Starting service {}...", service_name);

        let initial_msg = format!("Starting service: {}", service_name);
        self.log_manager.push(initial_msg.clone());
        tracing::info!("{}", initial_msg);

        let shared = TaskSharedState::new();

        self.task_manager.set_active_task(AsyncTask::StartService {
            service_name: service_name.clone(),
            result: Arc::clone(&shared.result),
            logs: Arc::clone(&shared.logs),
        });

        let use_case = Arc::clone(&self.use_cases.start_service);
        let name = service_name.clone();

        self.executor.spawn(async move {
            match use_case.execute(&name).await {
                Ok(_) => {
                    shared.set_success(format!("Successfully started service {}", service_name))
                }
                Err(e) => shared.set_failure(AppError::from_anyhow(e)),
            }
        });
    }

    pub(super) fn handle_stop_service(&mut self, service_name: String) {
        self.services_in_operation.insert(service_name.clone());
        self.set_operation_running("Stopping service", Some(service_name.clone()));
        self.status_message = format!("Stopping service {}...", service_name);

        let initial_msg = format!("Stopping service: {}", service_name);
        self.log_manager.push(initial_msg.clone());
        tracing::info!("{}", initial_msg);

        let shared = TaskSharedState::new();

        self.task_manager.set_active_task(AsyncTask::StopService {
            service_name: service_name.clone(),
            result: Arc::clone(&shared.result),
            logs: Arc::clone(&shared.logs),
        });

        let use_case = Arc::clone(&self.use_cases.stop_service);
        let name = service_name.clone();

        self.executor.spawn(async move {
            match use_case.execute(&name).await {
                Ok(_) => {
                    shared.set_success(format!("Successfully stopped service {}", service_name))
                }
                Err(e) => shared.set_failure(AppError::from_anyhow(e)),
            }
        });
    }

    pub(super) fn handle_restart_service(&mut self, service_name: String) {
        self.services_in_operation.insert(service_name.clone());
        self.set_operation_running("Restarting service", Some(service_name.clone()));
        self.status_message = format!("Restarting service {}...", service_name);

        let initial_msg = format!("Restarting service: {}", service_name);
        self.log_manager.push(initial_msg.clone());
        tracing::info!("{}", initial_msg);

        let shared = TaskSharedState::new();

        self.task_manager
            .set_active_task(AsyncTask::RestartService {
                service_name: service_name.clone(),
                result: Arc::clone(&shared.result),
                logs: Arc::clone(&shared.logs),
            });

        let use_case = Arc::clone(&self.use_cases.restart_service);
        let name = service_name.clone();

        self.executor.spawn(async move {
            match use_case.execute(&name).await {
                Ok(_) => {
                    shared.set_success(format!("Successfully restarted service {}", service_name))
                }
                Err(e) => shared.set_failure(AppError::from_anyhow(e)),
            }
        });
    }

    pub(super) fn handle_cleanup_orphans(&mut self) {
        if self.loading_clean_orphans {
            return;
        }

        self.loading_clean_orphans = true;
        self.set_operation_running("Cleaning orphaned dependencies", None);
        self.status_message = "Cleaning up orphaned dependencies...".to_string();
        self.log_manager
            .push("Cleaning up orphaned dependencies".to_string());
        tracing::info!("Cleaning up orphaned dependencies");

        let shared = TaskSharedState::new();

        self.task_manager.set_active_task(AsyncTask::CleanOrphans {
            result: Arc::clone(&shared.result),
            logs: Arc::clone(&shared.logs),
        });

        let use_case = Arc::clone(&self.use_cases.clean_orphans);

        self.executor.spawn(async move {
            match use_case.execute().await {
                Ok(_) => {
                    shared.set_success("Successfully cleaned up orphaned dependencies".to_string())
                }
                Err(e) => shared.set_failure(AppError::from_anyhow(e)),
            }
        });
    }

    pub(super) fn handle_export_packages(&mut self) {
        if self.loading_export {
            return;
        }

        let file_dialog = rfd::FileDialog::new()
            .add_filter("JSON files", &["json"])
            .set_file_name("brewsty_packages.json");

        if let Some(path) = file_dialog.save_file() {
            self.loading_export = true;
            self.set_operation_running("Exporting packages", None);
            self.status_message = "Exporting packages...".to_string();
            self.log_manager
                .push(format!("Exporting packages to: {}", path.display()));
            tracing::info!("Exporting packages to: {}", path.display());

            let shared = TaskSharedState::new();

            self.task_manager
                .set_active_task(AsyncTask::ExportPackages {
                    result: Arc::clone(&shared.result),
                    logs: Arc::clone(&shared.logs),
                });

            let use_case = Arc::clone(&self.use_cases.export_packages);
            let path_display = path.display().to_string();

            self.executor.spawn(async move {
                let result: anyhow::Result<crate::domain::entities::PackageList> =
                    use_case.execute(&path).await;

                match result {
                    Ok(package_list) => shared.set_success(format!(
                        "Successfully exported {} packages to {}",
                        package_list.total_count(),
                        path_display
                    )),
                    Err(e) => shared.set_failure(AppError::from_anyhow(e)),
                }
            });
        }
    }

    pub(super) fn handle_import_packages(&mut self) {
        if self.loading_import {
            return;
        }

        let file_dialog = rfd::FileDialog::new()
            .add_filter("JSON files", &["json"])
            .set_file_name("brewsty_packages.json");

        if let Some(path) = file_dialog.pick_file() {
            self.loading_import = true;
            self.set_operation_running("Importing packages", None);
            self.status_message = "Importing packages...".to_string();
            self.log_manager
                .push(format!("Importing packages from: {}", path.display()));
            tracing::info!("Importing packages from: {}", path.display());

            let shared = TaskSharedState::new();

            self.task_manager
                .set_active_task(AsyncTask::ImportPackages {
                    result: Arc::clone(&shared.result),
                    logs: Arc::clone(&shared.logs),
                });

            let use_case = Arc::clone(&self.use_cases.import_packages);
            let path_display = path.display().to_string();

            self.executor.spawn(async move {
                match use_case.execute(&path).await {
                    Ok(_) => shared.set_success(format!(
                        "Successfully imported packages from {}",
                        path_display
                    )),
                    Err(e) => shared.set_failure(AppError::from_anyhow(e)),
                }
            });
        }
    }

    pub(super) fn handle_update_all(&mut self) {
        if self.loading_update_all {
            return;
        }

        self.loading_update_all = true;
        self.set_operation_running("Updating all packages", None);
        self.status_message = "Updating all packages...".to_string();
        self.log_manager.push("Updating all packages".to_string());
        tracing::info!("Updating all packages");

        let shared = TaskSharedState::new();

        self.task_manager.set_active_task(AsyncTask::UpdateAll {
            result: Arc::clone(&shared.result),
            logs: Arc::clone(&shared.logs),
        });

        let use_case = Arc::clone(&self.use_cases.update_all);

        self.executor.spawn(async move {
            match use_case.execute().await {
                Ok(_) => shared.set_success("Successfully updated all packages".to_string()),
                Err(e) => shared.set_failure(AppError::from_anyhow(e)),
            }
        });
    }

    pub(super) fn show_cleanup_preview(&mut self, cleanup_type: CleanupType) {
        self.set_operation_running("Loading cleanup preview", None);
        self.status_message = "Loading cleanup preview...".to_string();
        self.log_manager.push("Loading cleanup preview".to_string());

        let result = Arc::new(Mutex::new(None));

        self.task_manager
            .set_active_task(AsyncTask::CleanupPreview {
                cleanup_type,
                result: Arc::clone(&result),
            });

        match cleanup_type {
            CleanupType::Cache => {
                let use_case = Arc::clone(&self.use_cases.clean_cache);
                self.executor.spawn(async move {
                    match use_case.preview().await {
                        Ok(p) => {
                            if let Ok(mut preview_result) = result.lock() {
                                *preview_result = Some(Ok(p));
                            }
                        }
                        Err(e) => {
                            if let Ok(mut preview_result) = result.lock() {
                                *preview_result = Some(Err(AppError::from_anyhow(e)));
                            }
                        }
                    }
                });
            }
            CleanupType::OldVersions => {
                let use_case = Arc::clone(&self.use_cases.cleanup_old_versions);
                self.executor.spawn(async move {
                    match use_case.preview().await {
                        Ok(p) => {
                            if let Ok(mut preview_result) = result.lock() {
                                *preview_result = Some(Ok(p));
                            }
                        }
                        Err(e) => {
                            if let Ok(mut preview_result) = result.lock() {
                                *preview_result = Some(Err(AppError::from_anyhow(e)));
                            }
                        }
                    }
                });
            }
            CleanupType::Orphans => {
                let use_case = Arc::clone(&self.use_cases.clean_orphans);
                self.executor.spawn(async move {
                    match use_case.preview().await {
                        Ok(p) => {
                            if let Ok(mut preview_result) = result.lock() {
                                *preview_result = Some(Ok(p));
                            }
                        }
                        Err(e) => {
                            if let Ok(mut preview_result) = result.lock() {
                                *preview_result = Some(Err(AppError::from_anyhow(e)));
                            }
                        }
                    }
                });
            }
        }
    }

    pub(super) fn handle_clean_cache(&mut self) {
        if self.loading_clean_cache {
            return;
        }

        self.loading_clean_cache = true;
        self.set_operation_running("Cleaning cache", None);
        self.status_message = "Cleaning cache...".to_string();
        self.log_manager.push("Cleaning Homebrew cache".to_string());
        tracing::info!("Cleaning Homebrew cache");

        let shared = TaskSharedState::new();

        self.task_manager.set_active_task(AsyncTask::CleanCache {
            result: Arc::clone(&shared.result),
            logs: Arc::clone(&shared.logs),
        });

        let use_case = Arc::clone(&self.use_cases.clean_cache);

        self.executor.spawn(async move {
            match use_case.execute().await {
                Ok(_) => shared.set_success("Successfully cleaned cache".to_string()),
                Err(e) => shared.set_failure(AppError::from_anyhow(e)),
            }
        });
    }

    pub(super) fn handle_cleanup_old_versions(&mut self) {
        if self.loading_cleanup_old_versions {
            return;
        }

        self.loading_cleanup_old_versions = true;
        self.set_operation_running("Cleaning old versions", None);
        self.status_message = "Cleaning up old versions...".to_string();
        self.log_manager
            .push("Cleaning up old versions".to_string());
        tracing::info!("Cleaning up old versions");

        let shared = TaskSharedState::new();

        self.task_manager
            .set_active_task(AsyncTask::CleanupOldVersions {
                result: Arc::clone(&shared.result),
                logs: Arc::clone(&shared.logs),
            });

        let use_case = Arc::clone(&self.use_cases.cleanup_old_versions);

        self.executor.spawn(async move {
            match use_case.execute().await {
                Ok(_) => shared.set_success("Successfully cleaned up old versions".to_string()),
                Err(e) => shared.set_failure(AppError::from_anyhow(e)),
            }
        });
    }

    pub(super) fn handle_doctor(&mut self) {
        self.set_operation_running("Running brew doctor", None);
        self.status_message = "Running brew doctor...".to_string();
        self.log_manager.push("Running brew doctor".to_string());

        let result = Arc::new(Mutex::new(None));

        self.task_manager.set_active_task(AsyncTask::Doctor {
            result: Arc::clone(&result),
        });

        self.executor.spawn(async move {
            match tokio::task::spawn_blocking(|| {
                crate::infrastructure::brew::command::BrewCommand::doctor()
            })
            .await
            {
                Ok(Ok(output)) => {
                    let parsed = crate::domain::entities::DoctorOutput::parse(&output);
                    if let Ok(mut r) = result.lock() {
                        *r = Some(Ok(parsed));
                    }
                }
                Ok(Err(e)) => {
                    if let Ok(mut doctor_result) = result.lock() {
                        *doctor_result = Some(Err(AppError::from_anyhow(e)));
                    }
                }
                Err(e) => {
                    if let Ok(mut doctor_result) = result.lock() {
                        *doctor_result =
                            Some(Err(AppError::Unknown(format!("Task join error: {}", e))));
                    }
                }
            }
        });
    }

    pub(super) fn load_taps(&mut self) {
        self.taps_state = LoadState::Loading;
        self.settings_message = None;
        self.set_operation_running("Loading taps", None);
        self.status_message = "Loading taps...".to_string();

        let shared = LoadTaskSharedState::new();

        self.task_manager.set_active_task(AsyncTask::LoadTaps {
            state: Arc::clone(&shared.state),
            logs: Arc::clone(&shared.logs),
        });

        self.executor.spawn(async move {
            match tokio::task::spawn_blocking(|| {
                crate::infrastructure::brew::command::BrewCommand::list_taps()
            })
            .await
            {
                Ok(Ok(output)) => {
                    let tap_list: Vec<String> = output
                        .lines()
                        .filter(|l| !l.is_empty())
                        .map(|l| l.trim().to_string())
                        .collect();
                    let msg = format!("Loaded {} taps", tap_list.len());
                    shared.push_log(msg);
                    shared.set_state(LoadState::Ready(tap_list));
                }
                Ok(Err(e)) => {
                    shared.push_log(format!("Error loading taps: {}", e));
                    shared.set_state(LoadState::Error(AppError::from_anyhow(e)));
                }
                Err(e) => {
                    shared.push_log(format!("Task error: {}", e));
                    shared.set_state(LoadState::Error(AppError::Unknown(format!(
                        "Task error: {}",
                        e
                    ))));
                }
            }
        });
    }

    pub(super) fn handle_tap(&mut self, name: String) {
        self.set_operation_running("Adding tap", Some(name.clone()));
        self.status_message = format!("Tapping {}...", name);
        self.log_manager.push(format!("Tapping: {}", name));

        let shared = TaskSharedState::new();
        self.task_manager.set_active_task(AsyncTask::Tap {
            result: Arc::clone(&shared.result),
            logs: Arc::clone(&shared.logs),
        });

        let tap_name = name.clone();
        self.executor.spawn(async move {
            match tokio::task::spawn_blocking(move || {
                crate::infrastructure::brew::command::BrewCommand::tap(&tap_name)
            })
            .await
            {
                Ok(Ok(_)) => shared.set_success(format!("Successfully tapped {}", name)),
                Ok(Err(e)) => shared.set_failure(AppError::from_anyhow(e)),
                Err(e) => shared.set_failure(AppError::Unknown(format!("Task error: {}", e))),
            }
        });
    }

    pub(super) fn handle_untap(&mut self, name: String) {
        self.set_operation_running("Removing tap", Some(name.clone()));
        self.status_message = format!("Untapping {}...", name);
        self.log_manager.push(format!("Untapping: {}", name));

        let shared = TaskSharedState::new();
        self.task_manager.set_active_task(AsyncTask::Untap {
            result: Arc::clone(&shared.result),
            logs: Arc::clone(&shared.logs),
        });

        let tap_name = name.clone();
        self.executor.spawn(async move {
            match tokio::task::spawn_blocking(move || {
                crate::infrastructure::brew::command::BrewCommand::untap(&tap_name)
            })
            .await
            {
                Ok(Ok(_)) => shared.set_success(format!("Successfully untapped {}", name)),
                Ok(Err(e)) => shared.set_failure(AppError::from_anyhow(e)),
                Err(e) => shared.set_failure(AppError::Unknown(format!("Task error: {}", e))),
            }
        });
    }

    pub(super) fn handle_search(&mut self) {
        if self.filter_state.search_query().is_empty() {
            return;
        }

        if self.loading_search {
            return;
        }

        self.loading_search = true;
        self.search_state = LoadState::Loading;
        self.search_message = None;
        self.set_operation_running(
            "Searching packages",
            Some(self.filter_state.search_query().to_string()),
        );
        self.status_message = format!("Searching for '{}'...", self.filter_state.search_query());
        let msg = format!("Searching for: {}", self.filter_state.search_query());
        self.log_manager.push(msg.clone());
        tracing::info!("{}", msg);

        let use_case_formulae = Arc::clone(&self.use_cases.search);
        let use_case_casks = Arc::clone(&self.use_cases.search);
        let query = self.filter_state.search_query().to_string();

        let search_state = LoadTaskSharedState::new();
        let query_clone = query.clone();

        self.task_manager.set_active_task(AsyncTask::Search {
            state: Arc::clone(&search_state.state),
            logs: Arc::clone(&search_state.logs),
        });

        self.executor.spawn(async move {
            let (formulae_result, casks_result) = tokio::join!(
                use_case_formulae.execute(&query, PackageType::Formula),
                use_case_casks.execute(&query_clone, PackageType::Cask)
            );

            let mut results = Vec::new();
            let mut logs = Vec::new();
            let formulae_error = formulae_result
                .as_ref()
                .err()
                .map(|error| AppError::classify_message(error.to_string()));
            let casks_error = casks_result
                .as_ref()
                .err()
                .map(|error| AppError::classify_message(error.to_string()));

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

            for log in logs {
                search_state.push_log(log);
            }
            let state = match (formulae_error, casks_error) {
                (None, None) => LoadState::Ready(results),
                (Some(error), None) | (None, Some(error)) => LoadState::Partial {
                    data: results,
                    warning: error,
                },
                (Some(left), Some(right)) => LoadState::Error(AppError::Unknown(format!(
                    "{}; {}",
                    left.short_message(),
                    right.short_message()
                ))),
            };
            search_state.set_state(state);
        });
    }

    pub(super) fn handle_bundle_dump(&mut self) {
        let file_dialog = rfd::FileDialog::new()
            .add_filter("Brewfile", &[""])
            .set_file_name("Brewfile");

        if let Some(path) = file_dialog.save_file() {
            let path_str = path.display().to_string();
            self.loading_bundle_dump = true;
            self.set_operation_running("Exporting Brewfile", None);
            self.status_message = "Exporting Brewfile...".to_string();
            self.log_manager
                .push(format!("Exporting Brewfile to: {}", path_str));
            tracing::info!("Exporting Brewfile to: {}", path_str);

            let shared = TaskSharedState::new();

            self.task_manager.set_active_task(AsyncTask::BundleDump {
                result: Arc::clone(&shared.result),
                logs: Arc::clone(&shared.logs),
            });

            let use_case = Arc::clone(&self.use_cases.bundle_dump);
            let path_display = path_str.clone();

            self.executor.spawn(async move {
                match use_case.execute(&path_display).await {
                    Ok(_) => shared.set_success(format!(
                        "Successfully exported Brewfile to {}",
                        path_display
                    )),
                    Err(e) => shared.set_failure(AppError::from_anyhow(e)),
                }
            });
        }
    }

    pub(super) fn handle_bundle_check_preview(&mut self) {
        let file_dialog = rfd::FileDialog::new()
            .add_filter("Brewfile", &[""])
            .set_file_name("Brewfile");

        if let Some(path) = file_dialog.pick_file() {
            let path_str = path.display().to_string();
            self.loading_bundle_check = true;
            self.set_operation_running("Checking Brewfile", Some(path_str.clone()));
            self.current_brewfile_path = Some(path_str.clone());
            self.status_message = "Checking Brewfile sync status...".to_string();
            self.log_manager
                .push(format!("Checking Brewfile: {}", path_str));
            tracing::info!("Checking Brewfile sync: {}", path_str);

            let result = Arc::new(Mutex::new(None));

            self.task_manager
                .set_active_task(AsyncTask::BundleCheckPreview {
                    result: Arc::clone(&result),
                });

            let use_case = Arc::clone(&self.use_cases.bundle_check_preview);

            self.executor.spawn(async move {
                match use_case.execute(&path_str).await {
                    Ok(p) => {
                        if let Ok(mut preview_result) = result.lock() {
                            *preview_result = Some(Ok(p));
                        }
                    }
                    Err(e) => {
                        if let Ok(mut preview_result) = result.lock() {
                            *preview_result = Some(Err(AppError::from_anyhow(e)));
                        }
                    }
                }
            });
        }
    }

    pub(super) fn handle_bundle_apply(&mut self, path: String, install: bool, cleanup: bool) {
        self.loading_bundle_apply = true;
        self.set_operation_running("Applying Brewfile", Some(path.clone()));
        self.status_message = "Applying Brewfile changes...".to_string();
        self.log_manager.push(format!(
            "Applying Brewfile: {} (install={}, cleanup={})",
            path, install, cleanup
        ));
        tracing::info!(
            "Applying Brewfile: {} (install={}, cleanup={})",
            path,
            install,
            cleanup
        );

        let shared = TaskSharedState::new();

        self.task_manager.set_active_task(AsyncTask::BundleApply {
            result: Arc::clone(&shared.result),
            logs: Arc::clone(&shared.logs),
        });

        let use_case = Arc::clone(&self.use_cases.bundle_apply);

        self.executor.spawn(async move {
            match use_case.execute(&path, install, cleanup).await {
                Ok(_) => shared.set_success("Successfully applied Brewfile changes".to_string()),
                Err(e) => shared.set_failure(AppError::from_anyhow(e)),
            }
        });
    }

    pub(super) fn load_package_info(&mut self, package_name: String, package_type: PackageType) {
        if self.task_manager.can_load_more_package_info() {
            self.load_package_info_immediate(package_name, package_type);
        } else {
            self.task_manager
                .queue_package_info_load(package_name, package_type);
        }
    }

    pub(super) fn load_package_info_immediate(
        &mut self,
        package_name: String,
        package_type: PackageType,
    ) {
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
        let package_type_clone = package_type;
        let package_type_clone2 = package_type;

        let task = AsyncTask::LoadPackageInfo {
            package_name: package_name.clone(),
            package_type,
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

    pub(super) fn handle_service_info(&mut self, service_name: String) {
        self.service_list.show_info_modal(service_name.clone());

        self.log_manager
            .push(format!("Loading info for service: {}", service_name));
        tracing::info!("Loading info for service: {}", service_name);

        let result = Arc::new(Mutex::new(None));

        self.task_manager
            .set_active_task(AsyncTask::ServiceInfoLoad {
                service_name: service_name.clone(),
                result: Arc::clone(&result),
            });

        let use_case = Arc::clone(&self.use_cases.get_service_info);
        let name = service_name.clone();

        self.executor.spawn(async move {
            match use_case.execute(&name).await {
                Ok(info) => {
                    if let Ok(mut r) = result.lock() {
                        *r = Some(Ok(info));
                    }
                }
                Err(e) => {
                    if let Ok(mut service_result) = result.lock() {
                        *service_result = Some(Err(AppError::from_anyhow(e)));
                    }
                }
            }
        });
    }

    pub(super) fn handle_service_log(&mut self, service_name: String) {
        self.service_list.show_log_modal(service_name.clone());

        self.log_manager
            .push(format!("Loading log for service: {}", service_name));
        tracing::info!("Loading log for service: {}", service_name);

        let result = Arc::new(Mutex::new(None));

        self.task_manager
            .set_active_task(AsyncTask::ServiceLogLoad {
                service_name: service_name.clone(),
                result: Arc::clone(&result),
            });

        let use_case = Arc::clone(&self.use_cases.get_service_log);
        let name = service_name.clone();

        self.executor.spawn(async move {
            match use_case.execute(&name, 100).await {
                Ok(log_text) => {
                    if let Ok(mut r) = result.lock() {
                        *r = Some(Ok(log_text));
                    }
                }
                Err(e) => {
                    if let Ok(mut service_result) = result.lock() {
                        *service_result = Some(Err(AppError::from_anyhow(e)));
                    }
                }
            }
        });
    }

    pub(super) fn handle_open_path(&mut self, path: String) {
        match std::process::Command::new("open").arg(&path).spawn() {
            Ok(_) => {
                self.set_operation_success(format!("Opened {}", path));
            }
            Err(error) => {
                let app_error = AppError::Io(format!("Failed to open {}: {}", path, error));
                self.set_operation_failure(app_error.clone());
                self.services_message = Some(app_error.to_user_message("Failed to open path"));
            }
        }
    }

    pub(super) fn handle_undo(&mut self, request: UndoRequest) {
        let pkg_type = request.package_type.unwrap_or(PackageType::Formula);
        let package = Package::new(request.target.clone(), pkg_type).set_installed(matches!(
            request.reverse_operation,
            OperationType::Uninstall | OperationType::Unpin
        ));

        match request.reverse_operation {
            OperationType::Install => {
                self.toast_manager
                    .info(format!("Undoing: re-installing {}", request.target));
                self.handle_install(package);
            }
            OperationType::Uninstall => {
                self.toast_manager
                    .info(format!("Undoing: uninstalling {}", request.target));
                self.handle_uninstall(package);
            }
            OperationType::Pin => {
                self.toast_manager
                    .info(format!("Undoing: pinning {}", request.target));
                self.handle_pin(package);
            }
            OperationType::Unpin => {
                self.toast_manager
                    .info(format!("Undoing: unpinning {}", request.target));
                self.handle_unpin(package);
            }
            _ => {
                self.toast_manager.error("This operation cannot be undone");
            }
        }
    }

    pub(super) fn check_for_updates_async(&mut self) {
        let shared = TaskSharedState::new();
        let result = Arc::clone(&shared.result);
        let logs = Arc::clone(&shared.logs);

        self.executor.spawn(async move {
            match crate::application::use_cases::check_for_updates().await {
                Ok(Some(result)) => shared.set_success(format!(
                    "New version available: {} → {}",
                    result.current_version, result.latest_version
                )),
                Ok(None) => shared.set_success("No updates available".to_string()),
                Err(e) => shared.set_failure(AppError::from_anyhow(e)),
            }
        });

        self.task_manager.set_active_task(AsyncTask::CheckUpdates {
            result,
            logs,
        });
    }
}
