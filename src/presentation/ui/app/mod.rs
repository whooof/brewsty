mod handlers;
mod polling;

use crate::application::UseCaseContainer;
use crate::domain::entities::{AppConfig, Package};
use crate::infrastructure::config_repository::ConfigRepository;
use crate::presentation::components::{
    CleanupAction, CleanupModal, CleanupType, FilterState, InfoModal, InfoModalAction, LogManager,
    MergedPackageList, PackageList, PasswordModal, ServiceList, Tab, TabManager,
};
use crate::presentation::services::{AsyncExecutor, AsyncTaskManager};
use crate::presentation::ui::tabs::installed::{InstalledAction, InstalledTab};
use crate::presentation::ui::tabs::log::{LogAction, LogTab};
use crate::presentation::ui::tabs::search::{SearchAction, SearchTab};
use crate::presentation::ui::tabs::services::{ServiceAction, ServicesTab};
use crate::presentation::ui::tabs::settings::{SettingsAction, SettingsTab};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

pub struct BrewstyApp {
    pub(super) tab_manager: TabManager,
    pub(super) filter_state: FilterState,

    pub(super) config: AppConfig,
    pub(super) config_repo: ConfigRepository,

    pub(super) cleanup_modal: CleanupModal,
    pub(super) info_modal: InfoModal,
    pub(super) password_modal: PasswordModal,
    pub(super) log_manager: LogManager,
    pub(super) log_rx: Receiver<String>,

    pub(super) merged_packages: MergedPackageList,
    pub(super) search_results: PackageList,
    pub(super) service_list: ServiceList,

    pub(super) auto_load_version_info: bool,

    pub(super) initialized: bool,

    pub(super) loading_installed: bool,
    pub(super) loading_outdated: bool,
    pub(super) loading_search: bool,
    pub(super) loading_services: bool,

    pub(super) loading_install: bool,
    pub(super) loading_uninstall: bool,
    pub(super) loading_update: bool,
    pub(super) loading_update_all: bool,
    pub(super) loading_clean_cache: bool,
    pub(super) loading_cleanup_old_versions: bool,
    pub(super) loading_export: bool,
    pub(super) loading_import: bool,

    pub(super) current_install_package: Option<String>,
    pub(super) current_uninstall_package: Option<String>,
    pub(super) current_update_package: Option<String>,
    pub(super) pending_updates: Vec<Package>,
    pub(super) pending_operation: Option<PendingOperation>,
    pub(super) confirm_action: Option<ConfirmAction>,
    pub(super) packages_in_operation: std::collections::HashSet<String>,
    pub(super) services_in_operation: std::collections::HashSet<String>,

    pub(super) task_manager: AsyncTaskManager,

    pub(super) use_cases: Arc<UseCaseContainer>,
    pub(super) executor: AsyncExecutor,

    pub(super) loading: bool,
    pub(super) status_message: String,
    pub(super) output_panel_height: f32,
    pub(super) doctor_output: Option<String>,
    pub(super) taps: Vec<String>,
    pub(super) new_tap_name: String,
    #[allow(clippy::type_complexity)]
    pub(super) pending_deps_load: Option<Arc<Mutex<Option<(String, String)>>>>,
}

#[derive(Clone, Debug)]
pub(super) enum PendingOperation {
    Install(Package),
    Uninstall(Package),
}

#[derive(Clone, Debug)]
pub(super) enum ConfirmAction {
    Install(Package),
    Uninstall(Package),
    Update(Package),
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
            confirm_action: None,
            packages_in_operation: std::collections::HashSet::new(),
            services_in_operation: std::collections::HashSet::new(),
            task_manager: AsyncTaskManager::new(),
            use_cases,
            executor,
            loading: false,
            status_message: String::new(),
            output_panel_height: 250.0,
            doctor_output: None,
            taps: Vec::new(),
            new_tap_name: String::new(),
            pending_deps_load: None,
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
}

pub(super) fn format_size(bytes: u64) -> String {
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

        if self.loading {
            ctx.request_repaint();
        }

        if !self.initialized {
            self.initialized = true;
            self.load_installed_packages(self.config.auto_update_check);
            self.apply_theme(ctx);
        }

        // Keyboard shortcuts (Cmd on macOS)
        let modifiers = ctx.input(|i| i.modifiers);
        if modifiers.command {
            if ctx.input(|i| i.key_pressed(egui::Key::R)) {
                match self.tab_manager.current() {
                    Tab::Installed => self.load_installed_packages(true),
                    Tab::Services => self.load_services(),
                    _ => {}
                }
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Num1)) {
                self.tab_manager.switch_to(Tab::Installed);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Num2)) {
                self.tab_manager.switch_to(Tab::SearchInstall);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Num3)) {
                self.tab_manager.switch_to(Tab::Services);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Num4)) {
                self.tab_manager.switch_to(Tab::Settings);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Num5)) {
                self.tab_manager.switch_to(Tab::Log);
            }
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
                        format!("Installed ({})", self.merged_packages.installed_count()),
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
                    .selectable_label(
                        self.tab_manager.is_current(Tab::Services),
                        format!("Services ({})", self.service_list.service_count()),
                    )
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
                            InstalledAction::Install(pkg) => self.maybe_confirm_install(pkg),
                            InstalledAction::Uninstall(pkg) => self.maybe_confirm_uninstall(pkg),
                            InstalledAction::Update(pkg) => self.maybe_confirm_update(pkg),
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
                            SearchAction::Install(pkg) => self.maybe_confirm_install(pkg),
                            SearchAction::Uninstall(pkg) => self.maybe_confirm_uninstall(pkg),
                            SearchAction::Update(pkg) => self.maybe_confirm_update(pkg),
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
                        &self.doctor_output,
                        &self.taps,
                        &mut self.new_tap_name,
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
                            SettingsAction::RunDoctor => self.handle_doctor(),
                            SettingsAction::LoadTaps => self.load_taps(),
                            SettingsAction::Tap(name) => self.handle_tap(name),
                            SettingsAction::Untap(name) => self.handle_untap(name),
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

            // Handle info modal dependency loading
            if let Some(InfoModalAction::LoadDeps(name)) = self.info_modal.render(ctx) {
                let info_modal_deps = Arc::new(Mutex::new(None::<(String, String)>));
                let deps_clone = Arc::clone(&info_modal_deps);
                let name_clone = name.clone();
                self.executor.spawn(async move {
                    let deps = tokio::task::spawn_blocking({
                        let n = name_clone.clone();
                        move || crate::infrastructure::brew::command::BrewCommand::deps(&n)
                    })
                    .await
                    .unwrap_or(Err(anyhow::anyhow!("task failed")));
                    let uses = tokio::task::spawn_blocking({
                        let n = name_clone;
                        move || crate::infrastructure::brew::command::BrewCommand::uses(&n)
                    })
                    .await
                    .unwrap_or(Err(anyhow::anyhow!("task failed")));
                    if let Ok(mut guard) = deps_clone.lock() {
                        *guard = Some((deps.unwrap_or_default(), uses.unwrap_or_default()));
                    }
                });
                self.pending_deps_load = Some(info_modal_deps);
            }

            // Confirmation dialog for destructive actions
            if let Some(action) = self.confirm_action.clone() {
                let (title, description) = match &action {
                    ConfirmAction::Install(pkg) => (
                        "Confirm Install".to_string(),
                        format!("Install {} ({})?", pkg.name, pkg.package_type),
                    ),
                    ConfirmAction::Uninstall(pkg) => (
                        "Confirm Uninstall".to_string(),
                        format!(
                            "Uninstall {} ({})? This cannot be undone.",
                            pkg.name, pkg.package_type
                        ),
                    ),
                    ConfirmAction::Update(pkg) => (
                        "Confirm Update".to_string(),
                        format!("Update {} ({})?", pkg.name, pkg.package_type),
                    ),
                };

                egui::Window::new(title)
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ctx, |ui| {
                        ui.label(&description);
                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            if ui.button("Confirm").clicked() {
                                match action {
                                    ConfirmAction::Install(pkg) => self.handle_install(pkg),
                                    ConfirmAction::Uninstall(pkg) => self.handle_uninstall(pkg),
                                    ConfirmAction::Update(pkg) => self.handle_update(pkg),
                                }
                                self.confirm_action = None;
                            }
                            if ui.button("Cancel").clicked() {
                                self.confirm_action = None;
                            }
                        });
                    });
            }

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
