use crate::domain::entities::ThemeMode;
use egui::{Color32, Context, FontFamily, FontId, Rounding, Stroke, TextStyle, Visuals};

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
    style.spacing.window_margin = egui::Margin::same(12.0);
    style.spacing.button_padding = egui::vec2(12.0, 8.0);
    style.spacing.indent = 24.0;
    style.spacing.interact_size = egui::vec2(60.0, 30.0);

    let mut visuals = match theme {
        ThemeMode::System => Visuals::dark(), // Default to dark for "System"
        ThemeMode::Light => Visuals::light(),
        ThemeMode::Dark => Visuals::dark(),
    };

    // Rounded corners
    visuals.widgets.noninteractive.rounding = Rounding::same(8.0);
    visuals.widgets.inactive.rounding = Rounding::same(8.0);
    visuals.widgets.hovered.rounding = Rounding::same(8.0);
    visuals.widgets.active.rounding = Rounding::same(8.0);
    visuals.widgets.open.rounding = Rounding::same(8.0);
    visuals.window_rounding = Rounding::same(12.0);
    visuals.menu_rounding = Rounding::same(8.0);

    if visuals.dark_mode {
        visuals.widgets.noninteractive.bg_fill = Color32::from_gray(32);
        visuals.window_fill = Color32::from_gray(20);
        visuals.panel_fill = Color32::from_gray(28);
        visuals.widgets.inactive.weak_bg_fill = Color32::from_gray(45);
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_gray(60));
        visuals.widgets.hovered.weak_bg_fill = Color32::from_gray(60);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_gray(100));
        visuals.selection.bg_fill = Color32::from_rgb(0, 122, 255);
        visuals.hyperlink_color = Color32::from_rgb(58, 150, 255);
    } else {
        visuals.widgets.noninteractive.bg_fill = Color32::from_gray(248);
        visuals.window_fill = Color32::WHITE;
        visuals.panel_fill = Color32::from_gray(242);
        visuals.widgets.inactive.weak_bg_fill = Color32::from_gray(230);
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_gray(200));
        visuals.selection.bg_fill = Color32::from_rgb(0, 122, 255);
    }

    ctx.set_style(style);
    ctx.set_visuals(visuals);
}
