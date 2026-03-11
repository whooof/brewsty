use crate::domain::entities::ThemeMode;
use egui::{Color32, Context, CornerRadius, FontFamily, FontId, Stroke, TextStyle, Visuals};

/// Detects macOS dark mode via `defaults read -g AppleInterfaceStyle`.
fn is_system_dark_mode() -> bool {
    std::process::Command::new("defaults")
        .args(["read", "-g", "AppleInterfaceStyle"])
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .trim()
                .eq_ignore_ascii_case("dark")
        })
        .unwrap_or(true) // default to dark if detection fails
}

/// Configures egui style with custom fonts, spacing, and theme-aware colors.
pub fn configure_style(ctx: &Context, theme: ThemeMode) {
    let mut style = (*ctx.style()).clone();

    style.text_styles = [
        (
            TextStyle::Small,
            FontId::new(14.0, FontFamily::Proportional),
        ),
        (TextStyle::Body, FontId::new(16.0, FontFamily::Proportional)),
        (
            TextStyle::Button,
            FontId::new(16.0, FontFamily::Proportional),
        ),
        (
            TextStyle::Heading,
            FontId::new(24.0, FontFamily::Proportional),
        ),
        (
            TextStyle::Monospace,
            FontId::new(15.0, FontFamily::Monospace),
        ),
    ]
    .into();

    style.spacing.item_spacing = egui::vec2(10.0, 10.0);
    style.spacing.window_margin = egui::Margin::same(14);
    style.spacing.button_padding = egui::vec2(14.0, 9.0);
    style.spacing.indent = 24.0;
    style.spacing.interact_size = egui::vec2(60.0, 30.0);

    let mut visuals = match theme {
        ThemeMode::System => {
            if is_system_dark_mode() {
                Visuals::dark()
            } else {
                Visuals::light()
            }
        }
        ThemeMode::Light => Visuals::light(),
        ThemeMode::Dark => Visuals::dark(),
    };

    // Rounded corners
    let corner_8 = CornerRadius::same(8);
    visuals.widgets.noninteractive.corner_radius = corner_8;
    visuals.widgets.inactive.corner_radius = corner_8;
    visuals.widgets.hovered.corner_radius = corner_8;
    visuals.widgets.active.corner_radius = corner_8;
    visuals.widgets.open.corner_radius = corner_8;
    visuals.window_corner_radius = CornerRadius::same(12);
    visuals.menu_corner_radius = corner_8;

    if visuals.dark_mode {
        visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(26, 30, 34);
        visuals.window_fill = Color32::from_rgb(16, 19, 22);
        visuals.panel_fill = Color32::from_rgb(22, 26, 30);
        visuals.faint_bg_color = Color32::from_rgb(28, 34, 40);
        visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(38, 45, 52);
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(67, 79, 90));
        visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(53, 66, 77);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(115, 148, 171));
        visuals.widgets.active.weak_bg_fill = Color32::from_rgb(60, 77, 90);
        visuals.widgets.active.bg_fill = Color32::from_rgb(28, 109, 144);
        visuals.widgets.active.bg_stroke = Stroke::new(1.0, Color32::from_rgb(129, 196, 226));
        visuals.selection.bg_fill = Color32::from_rgb(34, 151, 203);
        visuals.hyperlink_color = Color32::from_rgb(102, 196, 235);
    } else {
        visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(246, 245, 239);
        visuals.window_fill = Color32::WHITE;
        visuals.panel_fill = Color32::from_rgb(239, 236, 227);
        visuals.faint_bg_color = Color32::from_rgb(243, 240, 233);
        visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(231, 227, 217);
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(194, 187, 171));
        visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(220, 231, 234);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(96, 149, 173));
        visuals.widgets.active.weak_bg_fill = Color32::from_rgb(197, 224, 231);
        visuals.widgets.active.bg_fill = Color32::from_rgb(57, 139, 173);
        visuals.widgets.active.bg_stroke = Stroke::new(1.0, Color32::from_rgb(34, 96, 121));
        visuals.selection.bg_fill = Color32::from_rgb(57, 139, 173);
    }

    ctx.set_style(style);
    ctx.set_visuals(visuals);
}
