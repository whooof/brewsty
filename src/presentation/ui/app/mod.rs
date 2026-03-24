mod handlers;
mod polling;

use crate::application::UseCaseContainer;
use crate::domain::entities::{
    AppConfig, AppError, LoadState, MessageSeverity, OperationHistory, OperationState,
    OperationType, Package, Service, UserMessage,
};
use crate::infrastructure::config_repository::ConfigRepository;
use crate::presentation::components::{
    BrewfileSyncAction, BrewfileSyncModal, CleanupAction, CleanupModal, CleanupType, FilterState,
    InfoModal, InfoModalAction, LogManager, MergedPackageList, PackageList, ServiceList,
    ServiceModalAction, Tab, TabManager, ToastManager,
};
use crate::presentation::services::{AsyncExecutor, AsyncTaskManager};
use crate::presentation::ui::tabs::history::{HistoryAction, HistoryTab};
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
    pub(super) log_manager: LogManager,
    pub(super) toast_manager: ToastManager,
    pub(super) log_rx: Receiver<String>,

    pub(super) merged_packages: MergedPackageList,
    pub(super) search_results: PackageList,
    pub(super) service_list: ServiceList,
    pub(super) installed_state: LoadState<Vec<Package>>,
    pub(super) outdated_state: LoadState<Vec<Package>>,
    pub(super) search_state: LoadState<Vec<Package>>,
    pub(super) services_state: LoadState<Vec<Service>>,
    pub(super) taps_state: LoadState<Vec<String>>,
    pub(super) preflight_state: LoadState<()>,
    pub(super) installed_message: Option<UserMessage>,
    pub(super) search_message: Option<UserMessage>,
    pub(super) services_message: Option<UserMessage>,
    pub(super) settings_message: Option<UserMessage>,

    pub(super) auto_load_version_info: bool,
    pub(super) update_check_result: Option<crate::application::use_cases::UpdateCheckResult>,

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
    pub(super) loading_clean_orphans: bool,
    pub(super) loading_export: bool,
    pub(super) loading_import: bool,
    pub(super) loading_bundle_dump: bool,
    pub(super) loading_bundle_check: bool,
    pub(super) loading_bundle_apply: bool,

    pub(super) current_install_package: Option<String>,
    pub(super) current_uninstall_package: Option<String>,
    pub(super) current_update_package: Option<String>,
    pub(super) pending_updates: Vec<Package>,
    pub(super) confirm_action: Option<ConfirmAction>,
    pub(super) pending_settings_action: Option<SettingsDangerAction>,
    pub(super) packages_in_operation: std::collections::HashSet<String>,
    pub(super) services_in_operation: std::collections::HashSet<String>,

    pub(super) task_manager: AsyncTaskManager,

    pub(super) use_cases: Arc<UseCaseContainer>,
    pub(super) executor: AsyncExecutor,

    pub(super) loading: bool,
    pub(super) status_message: String,
    pub(super) operation_state: OperationState,
    pub(super) output_panel_height: f32,
    pub(super) show_bottom_log: bool,
    pub(super) doctor_output: Option<crate::domain::entities::DoctorOutput>,
    pub(super) taps: Vec<String>,
    pub(super) new_tap_name: String,
    pub(super) brewfile_sync_modal: BrewfileSyncModal,
    pub(super) current_brewfile_path: Option<String>,
    #[allow(clippy::type_complexity)]
    pub(super) pending_deps_load: Option<Arc<Mutex<Option<(String, String)>>>>,
    pub(super) operation_history: OperationHistory,
    pub(super) history_search_query: String,
}

#[derive(Clone, Debug)]
pub(super) enum ConfirmAction {
    Install(Package),
    Uninstall(Package),
    Update(Package),
}

#[derive(Clone, Debug)]
pub(super) enum SettingsDangerAction {
    UpdateAll,
    Untap(String),
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

        let operation_history = use_cases.load_history.execute().unwrap_or_else(|e| {
            tracing::error!("Failed to load operation history: {}", e);
            OperationHistory::default()
        });

        Self {
            tab_manager: TabManager::new(),
            filter_state: FilterState::new(),

            config: config.clone(),
            config_repo,

            cleanup_modal: CleanupModal::new(),
            info_modal: InfoModal::new(),
            log_manager: LogManager::new(),
            toast_manager: ToastManager::new(),
            log_rx,
            merged_packages: MergedPackageList::new(),
            search_results: PackageList::new(),
            service_list: ServiceList::new(),
            installed_state: LoadState::Idle,
            outdated_state: LoadState::Idle,
            search_state: LoadState::Idle,
            services_state: LoadState::Idle,
            taps_state: LoadState::Idle,
            preflight_state: LoadState::Idle,
            installed_message: None,
            search_message: None,
            services_message: None,
            settings_message: None,
            auto_load_version_info: false,
            update_check_result: None,
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
            loading_clean_orphans: false,
            loading_export: false,
            loading_import: false,
            loading_bundle_dump: false,
            loading_bundle_check: false,
            loading_bundle_apply: false,
            current_install_package: None,
            current_uninstall_package: None,
            current_update_package: None,
            pending_updates: Vec::new(),
            confirm_action: None,
            pending_settings_action: None,
            packages_in_operation: std::collections::HashSet::new(),
            services_in_operation: std::collections::HashSet::new(),
            task_manager: AsyncTaskManager::new(),
            use_cases,
            executor,
            loading: false,
            status_message: String::new(),
            operation_state: OperationState::Idle,
            output_panel_height: 250.0,
            show_bottom_log: false,
            doctor_output: None,
            taps: Vec::new(),
            new_tap_name: String::new(),
            brewfile_sync_modal: BrewfileSyncModal::new(),
            current_brewfile_path: None,
            pending_deps_load: None,
            operation_history,
            history_search_query: String::new(),
        }
    }

    fn save_config(&self) {
        if let Err(e) = self.config_repo.save(&self.config) {
            tracing::error!("Failed to save config: {}", e);
        }
    }

    pub(super) fn record_operation(
        &mut self,
        operation: crate::domain::entities::OperationType,
        target: Option<String>,
        package_type: Option<crate::domain::entities::PackageType>,
        success: bool,
        detail: Option<String>,
    ) {
        if let Err(e) = self.use_cases.record_operation.execute(
            &mut self.operation_history,
            operation,
            target,
            package_type,
            success,
            detail,
        ) {
            tracing::error!("Failed to record operation: {}", e);
        }
    }

    fn apply_theme(&self, ctx: &egui::Context) {
        crate::presentation::style::configure_style(ctx, self.config.theme);
    }

    fn set_operation_running(
        &mut self,
        kind: impl Into<std::borrow::Cow<'static, str>>,
        target: Option<String>,
    ) {
        self.loading = true;
        self.operation_state = OperationState::Running {
            kind: kind.into(),
            target,
        };
    }

    fn set_operation_success(&mut self, message: impl Into<String>) {
        let message = message.into();
        self.loading = false;
        self.status_message = message.clone();
        self.operation_state = OperationState::Succeeded { message };
    }

    fn set_operation_failure(&mut self, error: AppError) {
        self.loading = false;
        self.status_message = error.short_message();
        self.operation_state = OperationState::Failed { error };
    }

    fn status_text(&self) -> String {
        match &self.operation_state {
            OperationState::Idle => {
                if self.status_message.is_empty() {
                    "Ready".to_string()
                } else {
                    self.status_message.clone()
                }
            }
            OperationState::Running { kind, target } => match target {
                Some(target) => format!("{kind}: {target}"),
                None => kind.to_string(),
            },
            OperationState::Succeeded { message } => message.clone(),
            OperationState::Failed { error } => error.short_message(),
        }
    }

    fn is_busy(&self) -> bool {
        self.loading || matches!(self.operation_state, OperationState::Running { .. })
    }

    fn inline_message_ui(
        ui: &mut egui::Ui,
        message: &UserMessage,
        allow_retry: bool,
        allow_open_logs: bool,
    ) -> (bool, bool) {
        let dark_mode = ui.visuals().dark_mode;
        let (fill, stroke, title_color, body_color, details_fill) = match message.severity {
            MessageSeverity::Info => (
                if dark_mode {
                    egui::Color32::from_rgb(16, 55, 82)
                } else {
                    egui::Color32::from_rgb(218, 237, 250)
                },
                egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 181, 246)),
                if dark_mode {
                    egui::Color32::from_rgb(235, 245, 255)
                } else {
                    egui::Color32::from_rgb(19, 61, 86)
                },
                if dark_mode {
                    egui::Color32::from_rgb(219, 237, 248)
                } else {
                    egui::Color32::from_rgb(32, 75, 101)
                },
                if dark_mode {
                    egui::Color32::from_rgb(11, 37, 54)
                } else {
                    egui::Color32::from_rgb(242, 248, 252)
                },
            ),
            MessageSeverity::Success => (
                if dark_mode {
                    egui::Color32::from_rgb(25, 74, 47)
                } else {
                    egui::Color32::from_rgb(225, 242, 229)
                },
                egui::Stroke::new(1.0, egui::Color32::from_rgb(76, 175, 80)),
                if dark_mode {
                    egui::Color32::from_rgb(235, 250, 238)
                } else {
                    egui::Color32::from_rgb(31, 87, 40)
                },
                if dark_mode {
                    egui::Color32::from_rgb(216, 243, 222)
                } else {
                    egui::Color32::from_rgb(43, 99, 52)
                },
                if dark_mode {
                    egui::Color32::from_rgb(17, 49, 31)
                } else {
                    egui::Color32::from_rgb(243, 249, 244)
                },
            ),
            MessageSeverity::Warning => (
                if dark_mode {
                    egui::Color32::from_rgb(89, 60, 8)
                } else {
                    egui::Color32::from_rgb(255, 244, 214)
                },
                egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 179, 0)),
                if dark_mode {
                    egui::Color32::from_rgb(255, 244, 218)
                } else {
                    egui::Color32::from_rgb(112, 74, 0)
                },
                if dark_mode {
                    egui::Color32::from_rgb(255, 236, 194)
                } else {
                    egui::Color32::from_rgb(126, 87, 10)
                },
                if dark_mode {
                    egui::Color32::from_rgb(58, 40, 7)
                } else {
                    egui::Color32::from_rgb(255, 250, 239)
                },
            ),
            MessageSeverity::Error => (
                if dark_mode {
                    egui::Color32::from_rgb(94, 24, 28)
                } else {
                    egui::Color32::from_rgb(252, 228, 230)
                },
                egui::Stroke::new(1.0, egui::Color32::from_rgb(229, 57, 53)),
                if dark_mode {
                    egui::Color32::from_rgb(255, 236, 238)
                } else {
                    egui::Color32::from_rgb(112, 24, 24)
                },
                if dark_mode {
                    egui::Color32::from_rgb(255, 225, 228)
                } else {
                    egui::Color32::from_rgb(127, 37, 37)
                },
                if dark_mode {
                    egui::Color32::from_rgb(63, 17, 21)
                } else {
                    egui::Color32::from_rgb(255, 244, 245)
                },
            ),
        };

        let mut retry = false;
        let mut open_logs = false;

        egui::Frame::group(ui.style())
            .fill(fill)
            .stroke(stroke)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new(&message.title)
                            .strong()
                            .color(title_color),
                    );
                    ui.label(egui::RichText::new(&message.body).color(body_color));
                    if let Some(details) = &message.details {
                        egui::CollapsingHeader::new("Details")
                            .id_salt(format!("details_{}", message.title))
                            .show(ui, |ui| {
                                egui::Frame::group(ui.style())
                                    .fill(details_fill)
                                    .show(ui, |ui| {
                                        ui.monospace(
                                            egui::RichText::new(details).color(body_color),
                                        );
                                    });
                            });
                    }
                    ui.horizontal(|ui| {
                        if allow_retry
                            && ui
                                .button(
                                    message
                                        .recovery_action
                                        .clone()
                                        .unwrap_or_else(|| "Retry".to_string()),
                                )
                                .clicked()
                        {
                            retry = true;
                        }
                        if allow_open_logs && ui.button("Open Logs").clicked() {
                            open_logs = true;
                        }
                    });
                });
            });

        (retry, open_logs)
    }

    fn run_preflight_checks(&mut self) {
        self.preflight_state = LoadState::Loading;

        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("brewsty");

        let ensure_dir = std::fs::create_dir_all(&config_dir).map_err(|error| {
            AppError::Config(format!("Cannot create config directory: {}", error))
        });
        if let Err(error) = ensure_dir {
            self.preflight_state = LoadState::Error(error);
            return;
        }

        let brew_check = crate::infrastructure::brew::command::BrewCommand::list_taps()
            .map(|_| ())
            .map_err(AppError::from_anyhow);
        if let Err(error) = brew_check {
            self.preflight_state = LoadState::Error(error);
            return;
        }

        let services_check =
            crate::infrastructure::brew::command::BrewCommand::list_services_json()
                .map(|_| ())
                .map_err(AppError::from_anyhow);
        if let Err(error) = services_check {
            self.preflight_state = LoadState::Error(error);
            return;
        }

        self.preflight_state = LoadState::Ready(());
    }
}

pub fn format_size(bytes: u64) -> String {
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

        // Check search debounce
        if self.config.search_debounce_enabled
            && self
                .filter_state
                .check_debounce_trigger(self.config.search_debounce_delay)
        {
            self.handle_search();
        }

        if !self.initialized {
            self.initialized = true;
            self.run_preflight_checks();
            if matches!(self.preflight_state, LoadState::Ready(_)) {
                self.load_installed_packages(self.config.auto_update_check);
                self.load_services();
                // Check for updates in background
                self.check_for_updates_async();
            }
            self.apply_theme(ctx);
        }

        // Keyboard shortcuts (Cmd on macOS)
        let modifiers = ctx.input(|i| i.modifiers);
        if modifiers.command {
            // Cmd+Q: Quit application
            if ctx.input(|i| i.key_pressed(egui::Key::Q)) {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            // Cmd+F: Focus search (handled by filter_state in UI)
            // Cmd+R: Refresh current tab
            if ctx.input(|i| i.key_pressed(egui::Key::R)) {
                match self.tab_manager.current() {
                    Tab::Installed => self.load_installed_packages(true),
                    Tab::Services => self.load_services(),
                    Tab::SearchInstall => self.handle_search(),
                    Tab::History => { /* Refresh history */ }
                    _ => {}
                }
            }
            // Cmd+S: Save configuration
            if ctx.input(|i| i.key_pressed(egui::Key::S)) {
                self.config_repo.save(&self.config).ok();
                self.toast_manager
                    .success("Configuration saved".to_string());
            }
            // Cmd+Z: Undo last operation
            if ctx.input(|i| i.key_pressed(egui::Key::Z)) {
                if let Some(last_op) = self.operation_history.records.first() {
                    if last_op.operation == OperationType::Install
                        || last_op.operation == OperationType::Uninstall
                        || last_op.operation == OperationType::Update
                    {
                        self.toast_manager
                            .info(format!("Undo not available for {:?}", last_op.operation));
                    }
                }
            }
            // Cmd+1-6: Switch tabs
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
                self.tab_manager.switch_to(Tab::History);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Num5)) {
                self.tab_manager.switch_to(Tab::Settings);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Num6)) {
                self.tab_manager.switch_to(Tab::Log);
            }
            // Cmd+A: Select all (in Outdated tab)
            if ctx.input(|i| i.key_pressed(egui::Key::A))
                && self.tab_manager.is_current(Tab::Installed)
            {
                // Select all outdated packages
            }
        }

        // Delete/Backspace: Uninstall selected package
        if (ctx.input(|i| i.key_pressed(egui::Key::Delete))
            || ctx.input(|i| i.key_pressed(egui::Key::Backspace)))
            && self.tab_manager.is_current(Tab::Installed)
        {
            // Handle uninstall of selected package
        }

        // Escape: Close modals
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.confirm_action = None;
            self.cleanup_modal = crate::presentation::components::CleanupModal::new();
        }

        // ?: Show keyboard shortcuts help
        if ctx.input(|i| i.key_pressed(egui::Key::Slash))
            || ctx.input(|i| i.key_pressed(egui::Key::F1))
        {
            // Show shortcuts help modal
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
                    .selectable_label(
                        self.tab_manager.is_current(Tab::History),
                        format!("History ({})", self.operation_history.records.len()),
                    )
                    .clicked()
                {
                    self.tab_manager.switch_to(Tab::History);
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
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if self.is_busy() {
                    ui.spinner();
                }
                ui.label(egui::RichText::new(self.status_text()).small());
                ui.separator();
                let toggle_label = if self.show_bottom_log {
                    "Hide Bottom Log"
                } else {
                    "Show Bottom Log"
                };
                if ui.small_button(toggle_label).clicked() {
                    self.show_bottom_log = !self.show_bottom_log;
                }
            });
            ui.add_space(8.0);
        });

        if self.tab_manager.current() != Tab::Log && self.show_bottom_log {
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
                        if ui.button("Copy Output").clicked() {
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
                            ui.spacing_mut().item_spacing.y = 2.0;

                            for entry in self.log_manager.filtered_logs() {
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

                    self.output_panel_height = ui.min_rect().height();
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if let LoadState::Error(error) = self.preflight_state.clone() {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    ui.heading("Startup checks failed");
                    ui.add_space(8.0);
                    let message = error
                        .to_user_message("Brewsty could not start")
                        .with_recovery_action("Retry startup checks");
                    let (retry, open_logs) =
                        Self::inline_message_ui(ui, &message, true, true);
                    if retry {
                        self.run_preflight_checks();
                        if matches!(self.preflight_state, LoadState::Ready(_)) {
                            self.load_installed_packages(self.config.auto_update_check);
                            self.load_services();
                        }
                    }
                    if open_logs {
                        self.tab_manager.switch_to(Tab::Log);
                    }
                });
                self.toast_manager.show(ctx);
                return;
            }

            match self.tab_manager.current() {
                Tab::Installed => {
                    if let Some(message) = self.installed_message.clone() {
                        let (retry, open_logs) =
                            Self::inline_message_ui(ui, &message, true, true);
                        ui.add_space(8.0);
                        if retry {
                            self.load_installed_packages(true);
                        }
                        if open_logs {
                            self.tab_manager.switch_to(Tab::Log);
                        }
                    }
                    let actions = InstalledTab::show(
                        ui,
                        &mut self.merged_packages,
                        &mut self.filter_state,
                        &self.packages_in_operation,
                        self.installed_state.is_loading(),
                        self.outdated_state.is_loading(),
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
                    if let Some(message) = self.search_message.clone() {
                        let (retry, open_logs) =
                            Self::inline_message_ui(ui, &message, true, true);
                        ui.add_space(8.0);
                        if retry {
                            self.handle_search();
                        }
                        if open_logs {
                            self.tab_manager.switch_to(Tab::Log);
                        }
                    }
                    let actions = SearchTab::show(
                        ui,
                        &mut self.search_results,
                        &mut self.filter_state,
                        &self.packages_in_operation,
                        self.search_state.is_loading(),
                        &mut self.auto_load_version_info,
                        &mut self.info_modal,
                    );

                    for action in actions {
                        match action {
                            SearchAction::Search => self.handle_search(),
                            SearchAction::QueryChanged => {
                                if self.config.search_debounce_enabled {
                                    self.filter_state.mark_typing();
                                }
                            }
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
                    if let Some(message) = self.services_message.clone() {
                        let (retry, open_logs) =
                            Self::inline_message_ui(ui, &message, true, true);
                        ui.add_space(8.0);
                        if retry {
                            self.load_services();
                        }
                        if open_logs {
                            self.tab_manager.switch_to(Tab::Log);
                        }
                    }
                    let actions = ServicesTab::show(
                        ui,
                        &mut self.service_list,
                        &self.services_in_operation,
                        self.services_state.is_loading(),
                    );

                    for action in actions {
                        match action {
                            ServiceAction::Refresh => self.load_services(),
                            ServiceAction::Start(name) => self.handle_start_service(name),
                            ServiceAction::Stop(name) => self.handle_stop_service(name),
                            ServiceAction::Restart(name) => self.handle_restart_service(name),
                            ServiceAction::ViewInfo(name) => self.handle_service_info(name),
                            ServiceAction::ViewLog(name) => self.handle_service_log(name),
                        }
                    }
                }

                Tab::History => {
                    let actions = HistoryTab::show(ui, &self.operation_history, &mut self.history_search_query);

                    for action in actions {
                        match action {
                            HistoryAction::Undo(request) => self.handle_undo(request),
                            HistoryAction::ClearHistory => {
                                if let Err(e) = self
                                    .use_cases
                                    .clear_history
                                    .execute(&mut self.operation_history)
                                {
                                    tracing::error!("Failed to clear history: {}", e);
                                    self.toast_manager.error("Failed to clear history");
                                } else {
                                    self.toast_manager.success("History cleared");
                                }
                            }
                        }
                    }
                }

                Tab::Settings => {
                    tracing::trace!("Rendering Settings Tab");
                    if let Some(message) = self.settings_message.clone() {
                        let (retry, open_logs) =
                            Self::inline_message_ui(ui, &message, true, true);
                        ui.add_space(8.0);
                        if retry {
                            self.load_taps();
                        }
                        if open_logs {
                            self.tab_manager.switch_to(Tab::Log);
                        }
                    }
                    let actions = SettingsTab::show(
                        ui,
                        &mut self.config,
                        &mut self.log_manager,
                        self.loading_export,
                        self.loading_import,
                        self.loading_bundle_dump,
                        self.loading_bundle_check,
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
                            SettingsAction::UpdateAll => {
                                self.pending_settings_action = Some(SettingsDangerAction::UpdateAll)
                            }
                            SettingsAction::ExportPackages => self.handle_export_packages(),
                            SettingsAction::ImportPackages => self.handle_import_packages(),
                            SettingsAction::RunDoctor => self.handle_doctor(),
                            SettingsAction::LoadTaps => self.load_taps(),
                            SettingsAction::Tap(name) => self.handle_tap(name),
                            SettingsAction::Untap(name) => {
                                self.pending_settings_action =
                                    Some(SettingsDangerAction::Untap(name))
                            }
                            SettingsAction::ExportBrewfile => self.handle_bundle_dump(),
                            SettingsAction::SyncBrewfile => self.handle_bundle_check_preview(),
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
                        CleanupType::Orphans => self.handle_cleanup_orphans(),
                    },
                    CleanupAction::Cancel => {
                        self.cleanup_modal.close();
                    }
                }
            }

            // Handle brewfile sync modal
            if let Some(action) = self.brewfile_sync_modal.render(ctx) {
                match action {
                    BrewfileSyncAction::Apply { install, cleanup } => {
                        if let Some(path) = self.current_brewfile_path.clone() {
                            self.handle_bundle_apply(path, install, cleanup);
                        }
                    }
                    BrewfileSyncAction::Cancel => {
                        self.brewfile_sync_modal.close();
                    }
                }
            }

            // Handle service detail modal (info + log)
            if let Some(action) = self.service_list.render_detail_modal(ctx) {
                match action {
                    ServiceModalAction::ReloadInfo(name) => self.handle_service_info(name),
                    ServiceModalAction::LoadLog(name) => self.handle_service_log(name),
                    ServiceModalAction::OpenPath(path) => self.handle_open_path(path),
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

            if let Some(action) = self.pending_settings_action.clone() {
                let (title, description) = match &action {
                    SettingsDangerAction::UpdateAll => (
                        "Confirm Update All".to_string(),
                        "Update all installed packages? This can take a while and may require your administrator password.".to_string(),
                    ),
                    SettingsDangerAction::Untap(name) => (
                        "Confirm Untap".to_string(),
                        format!("Untap {}? This may affect packages that depend on that tap.", name),
                    ),
                };

                egui::Window::new(title)
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ctx, |ui| {
                        ui.label(description);
                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            if ui.button("Confirm").clicked() {
                                match action {
                                    SettingsDangerAction::UpdateAll => self.handle_update_all(),
                                    SettingsDangerAction::Untap(name) => self.handle_untap(name),
                                }
                                self.pending_settings_action = None;
                            }
                            if ui.button("Cancel").clicked() {
                                self.pending_settings_action = None;
                            }
                        });
                    });
            }
        });

        self.toast_manager.show(ctx);
    }
}
