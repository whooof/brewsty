use egui::{Align2, Area, Color32, CornerRadius, Frame, Id, Margin, Order, RichText};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ToastType {
    Info,
    Success,
    Error,
}

pub struct Toast {
    id: Id,
    message: String,
    details: Option<String>,
    toast_type: ToastType,
    expires_at: Instant,
    occurrences: usize,
    expanded: bool,
}

pub struct ToastManager {
    toasts: Vec<Toast>,
    duration: Duration,
    next_id: u64,
}

impl Default for ToastManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ToastManager {
    pub fn new() -> Self {
        Self {
            toasts: Vec::new(),
            duration: Duration::from_secs(4),
            next_id: 0,
        }
    }

    pub fn success(&mut self, message: impl Into<String>) {
        self.add(message, None::<String>, ToastType::Success);
    }

    pub fn error(&mut self, message: impl Into<String>) {
        self.add(message, None::<String>, ToastType::Error);
    }

    #[allow(dead_code)]
    pub fn error_with_details(&mut self, message: impl Into<String>, details: impl Into<String>) {
        self.add(message, Some(details.into()), ToastType::Error);
    }

    pub fn info(&mut self, message: impl Into<String>) {
        self.add(message, None::<String>, ToastType::Info);
    }

    fn add(&mut self, message: impl Into<String>, details: Option<String>, toast_type: ToastType) {
        let message = message.into();
        let now = Instant::now();

        if let Some(existing) = self
            .toasts
            .iter_mut()
            .find(|toast| toast.message == message && toast.toast_type == toast_type)
        {
            existing.expires_at = now
                + if toast_type == ToastType::Error {
                    self.duration + Duration::from_secs(4)
                } else {
                    self.duration
                };
            existing.occurrences += 1;
            if existing.details.is_none() {
                existing.details = details;
            }
            return;
        }

        self.toasts.push(Toast {
            id: Id::new(format!("toast_{}", self.next_id)),
            message,
            details,
            toast_type,
            expires_at: now
                + if toast_type == ToastType::Error {
                    self.duration + Duration::from_secs(4)
                } else {
                    self.duration
                },
            occurrences: 1,
            expanded: false,
        });
        self.next_id = self.next_id.wrapping_add(1);
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        let now = Instant::now();
        self.toasts.retain(|t| t.expires_at > now);

        if self.toasts.is_empty() {
            return;
        }

        let mut y_offset = 50.0; // Start offset from top right

        let mut dismiss_ids = Vec::new();

        for toast in self.toasts.iter_mut().rev() {
            // Show newest at top
            let (bg_color, text_color, icon) = match toast.toast_type {
                ToastType::Success => (Color32::from_rgb(46, 125, 50), Color32::WHITE, "✅ "),
                ToastType::Error => (Color32::from_rgb(198, 40, 40), Color32::WHITE, "❌ "),
                ToastType::Info => (Color32::from_rgb(21, 101, 192), Color32::WHITE, ""),
            };

            let frame = Frame {
                inner_margin: Margin::same(12),
                corner_radius: CornerRadius::same(8),
                fill: bg_color,
                stroke: egui::Stroke::new(1.0, bg_color.linear_multiply(0.8)),
                shadow: egui::epaint::Shadow {
                    blur: 8,
                    spread: 4,
                    color: Color32::from_black_alpha(50),
                    offset: [0, 4],
                },
                ..Default::default()
            };

            // Calculate fade out alpha
            let remaining = toast.expires_at.duration_since(now).as_secs_f32();
            let alpha = if remaining < 0.5 {
                (remaining / 0.5).clamp(0.0, 1.0)
            } else {
                1.0
            };

            Area::new(toast.id)
                .order(Order::Tooltip)
                .anchor(Align2::RIGHT_TOP, [-20.0, y_offset])
                .show(ctx, |ui| {
                    let mut frame = frame;
                    frame.fill = frame.fill.linear_multiply(alpha);
                    frame.stroke.color = frame.stroke.color.linear_multiply(alpha);

                    let response = frame
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(icon)
                                            .color(text_color.linear_multiply(alpha)),
                                    );
                                    let message = if toast.occurrences > 1 {
                                        format!("{} ({})", toast.message, toast.occurrences)
                                    } else {
                                        toast.message.clone()
                                    };
                                    ui.label(
                                        RichText::new(message)
                                            .color(text_color.linear_multiply(alpha))
                                            .strong(),
                                    );
                                    if ui.small_button("Dismiss").clicked() {
                                        dismiss_ids.push(toast.id);
                                    }
                                });
                                if let Some(details) = &toast.details {
                                    let toggle_label = if toast.expanded {
                                        "Hide details"
                                    } else {
                                        "Show details"
                                    };
                                    if ui.small_button(toggle_label).clicked() {
                                        toast.expanded = !toast.expanded;
                                    }
                                    if toast.expanded {
                                        ui.add_space(4.0);
                                        ui.label(
                                            RichText::new(details)
                                                .color(text_color.linear_multiply(alpha))
                                                .monospace(),
                                        );
                                    }
                                }
                            });
                        })
                        .response;

                    y_offset += response.rect.height() + 10.0;
                });
        }

        if !dismiss_ids.is_empty() {
            self.toasts.retain(|toast| !dismiss_ids.contains(&toast.id));
        }

        ctx.request_repaint(); // Keep repainting for smooth fade out
    }
}
