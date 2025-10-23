use egui::Key;

pub struct PasswordModal {
    show: bool,
    password_input: String,
    operation_name: String,
    confirmed: bool,
    cancelled: bool,
    show_password: bool,
}

impl PasswordModal {
    pub fn new() -> Self {
        Self {
            show: false,
            password_input: String::new(),
            operation_name: String::new(),
            confirmed: false,
            cancelled: false,
            show_password: false,
        }
    }

    pub fn show(&mut self, operation_name: String) {
        self.show = true;
        self.password_input.clear();
        self.operation_name = operation_name;
        self.confirmed = false;
        self.cancelled = false;
        self.show_password = false;
    }

    pub fn is_open(&self) -> bool {
        self.show
    }

    pub fn take_result(&mut self) -> Option<(bool, String)> {
        if self.confirmed {
            self.confirmed = false;
            let password = self.password_input.clone();
            self.password_input.clear();
            self.show = false;
            Some((true, password))
        } else if self.cancelled {
            self.cancelled = false;
            self.password_input.clear();
            self.show = false;
            Some((false, String::new()))
        } else {
            None
        }
    }

    pub fn close(&mut self) {
        self.show = false;
        self.password_input.clear();
        self.cancelled = true;
    }

    pub fn render(&mut self, ctx: &egui::Context) {
        if !self.show {
            return;
        }

        let mut open = self.show;
        egui::Window::new(format!("Password Required: {}", self.operation_name))
            .collapsible(false)
            .resizable(false)
            .default_width(350.0)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.label("This operation requires administrator password.");
                    ui.add_space(12.0);

                    ui.label("Password:");
                    let password_field = if self.show_password {
                        egui::TextEdit::singleline(&mut self.password_input)
                            .desired_width(f32::INFINITY)
                    } else {
                        egui::TextEdit::singleline(&mut self.password_input)
                            .password(true)
                            .desired_width(f32::INFINITY)
                    };

                    let response = ui.add(password_field);

                    // Request focus for the password field
                    if response.gained_focus() {
                        response.request_focus();
                    }

                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.show_password, "Show password");
                    });

                    ui.add_space(12.0);

                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Cancel").clicked() {
                                self.cancelled = true;
                            }

                            if ui.button("OK").clicked() {
                                self.confirmed = true;
                            }

                            // Handle Enter key to submit
                            if ui.input(|i| i.key_pressed(Key::Enter)) {
                                self.confirmed = true;
                            }
                        });
                    });
                });
            });

        if !open {
            self.close();
        }
    }
}

impl Default for PasswordModal {
    fn default() -> Self {
        Self::new()
    }
}
