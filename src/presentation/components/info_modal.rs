use crate::domain::entities::Package;

pub enum InfoModalAction {
    LoadDeps(String),
}

pub struct InfoModal {
    show: bool,
    package: Option<Package>,
    deps_tree: Option<String>,
    used_by: Option<String>,
    loading_deps: bool,
}

impl InfoModal {
    pub fn new() -> Self {
        Self {
            show: false,
            package: None,
            deps_tree: None,
            used_by: None,
            loading_deps: false,
        }
    }

    pub fn show(&mut self, package: Package) {
        self.package = Some(package);
        self.show = true;
        self.deps_tree = None;
        self.used_by = None;
        self.loading_deps = false;
    }

    pub fn close(&mut self) {
        self.show = false;
        self.package = None;
        self.deps_tree = None;
        self.used_by = None;
        self.loading_deps = false;
    }

    pub fn set_deps(&mut self, deps: String, used_by: String) {
        self.deps_tree = Some(deps);
        self.used_by = Some(used_by);
        self.loading_deps = false;
    }

    pub fn render(&mut self, ctx: &egui::Context) -> Option<InfoModalAction> {
        if !self.show {
            return None;
        }

        let mut action = None;

        if let Some(package) = self.package.clone() {
            let mut open = self.show;
            egui::Window::new(format!("Info: {}", package.name))
                .collapsible(false)
                .resizable(true)
                .default_width(400.0)
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.vertical(|ui| {
                        ui.heading(&package.name);
                        ui.separator();

                        ui.label(egui::RichText::new("Type:").strong());
                        ui.label(package.package_type.to_string());

                        ui.add_space(8.0);

                        if let Some(version) = &package.version {
                            ui.label(egui::RichText::new("Version:").strong());
                            ui.label(version);
                            ui.add_space(8.0);
                        }

                        if let Some(desc) = &package.description {
                            ui.label(egui::RichText::new("Description:").strong());
                            ui.label(desc);
                            ui.add_space(8.0);
                        }

                        ui.separator();

                        if self.loading_deps {
                            ui.horizontal(|ui| {
                                ui.spinner();
                                ui.label("Loading dependencies...");
                            });
                        } else if let Some(deps) = &self.deps_tree {
                            ui.label(egui::RichText::new("Dependencies:").strong());
                            egui::ScrollArea::vertical()
                                .id_salt("deps_tree")
                                .max_height(150.0)
                                .show(ui, |ui| {
                                    ui.monospace(if deps.is_empty() { "None" } else { deps });
                                });
                            ui.add_space(8.0);
                            if let Some(used) = &self.used_by {
                                ui.label(egui::RichText::new("Used by:").strong());
                                ui.monospace(if used.is_empty() { "None" } else { used });
                            }
                        } else if ui.button("Show Dependencies").clicked() {
                            self.loading_deps = true;
                            action = Some(InfoModalAction::LoadDeps(package.name.clone()));
                        }

                        ui.separator();
                        if ui.button("Close").clicked() {
                            self.close();
                        }
                    });
                });

            if !open {
                self.close();
            }
        }

        action
    }
}

impl Default for InfoModal {
    fn default() -> Self {
        Self::new()
    }
}
