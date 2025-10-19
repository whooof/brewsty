use crate::domain::entities::Package;

pub struct InfoModal {
    show: bool,
    package: Option<Package>,
}

impl InfoModal {
    pub fn new() -> Self {
        Self {
            show: false,
            package: None,
        }
    }

    pub fn show(&mut self, package: Package) {
        self.package = Some(package);
        self.show = true;
    }

    pub fn close(&mut self) {
        self.show = false;
        self.package = None;
    }

    pub fn render(&mut self, ctx: &egui::Context) {
        if !self.show {
            return;
        }

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
                        if ui.button("Close").clicked() {
                            self.close();
                        }
                    });
                });

            if !open {
                self.close();
            }
        }
    }
}

impl Default for InfoModal {
    fn default() -> Self {
        Self::new()
    }
}
