use crate::domain::entities::ThemeMode;
use egui::{
    FontData, FontDefinitions, FontFamily, FontId, Rounding, Stroke, TextStyle, Visuals, Color32,
    Context,
};

pub fn configure_style(ctx: &Context, theme: ThemeMode) {
    // 1. Configure Fonts
    let mut fonts = FontDefinitions::default();

    // Add emoji font
    fonts.font_data.insert(
        "emoji".to_owned(),
        FontData::from_static(include_bytes!("/System/Library/Fonts/Apple Color Emoji.ttc")),
    );

    // Fallback priority
    fonts.families.entry(FontFamily::Proportional).or_default().push("emoji".to_owned());
    fonts.families.entry(FontFamily::Monospace).or_default().push("emoji".to_owned());

    // Install custom fonts or prioritize system fonts
    // Mac specific: prioritize San Francisco if possible, but egui defaults are limited.
    // We'll insert a "best effort" nice font configuration.
    // Actually, egui's default fonts are "Hack" for monospace and "Ubuntu-Light" for proportional.
    // We can try to rely on the OS fonts if we had a font loader, but for now let's just
    // stick to standard egui fonts but configured larger.

    // Let's just create larger styles for now.
    
    // We use the default fonts but we will reconfigure the text styles sizes.
    // If we wanted to load a font file we would need the bytes.
    // For a "fresh" look, standard egui fonts are a bit "gamey/tooly".
    // Since I can't easily curl a font file right now without internet access or ensuring it exists,
    // I will focus on sizing and visual settings which make a huge difference.
    
    // However, if the user is on Mac, we might want to try to use the system font if we can load it.
    // But safely, let's stick to tuning sizes.
    
    // Scale up everything a bit
    let mut style = (*ctx.style()).clone();
    
    // Text Styles
    style.text_styles = [
        (TextStyle::Small, FontId::new(14.0, FontFamily::Proportional)),
        (TextStyle::Body, FontId::new(16.0, FontFamily::Proportional)),
        (TextStyle::Button, FontId::new(16.0, FontFamily::Proportional)),
        (TextStyle::Heading, FontId::new(24.0, FontFamily::Proportional)),
        (TextStyle::Monospace, FontId::new(15.0, FontFamily::Monospace)),
    ]
    .into();

    // Spacing
    style.spacing.item_spacing = egui::vec2(10.0, 10.0); // More breathing room
    style.spacing.window_margin = egui::Margin::same(12.0);
    style.spacing.button_padding = egui::vec2(12.0, 8.0); // Larger buttons
    style.spacing.indent = 24.0;
    style.spacing.interact_size = egui::vec2(60.0, 30.0); // Larger interaction areas
    
    // Visuals
    let mut visuals = match theme {
        ThemeMode::System => {
             // Basic heuristic, though egui doesn't auto-detect system theme perfectly without invalidation
             // Default to Dark for "Premium" feel if system detection is tricky, 
             // but let's trust standard egui behavior or just default to Dark for now as it often looks "fresher".
             // Actually, let's respect the passed theme properly.
             // If system, we might need to check system config if possible, 
             // but for now let's fallback to Dark as the "default" cool look.
             Visuals::dark()
        }
        ThemeMode::Light => Visuals::light(),
        ThemeMode::Dark => Visuals::dark(),
    };

    // Global widget rounding
    visuals.widgets.noninteractive.rounding = Rounding::same(8.0);
    visuals.widgets.inactive.rounding = Rounding::same(8.0);
    visuals.widgets.hovered.rounding = Rounding::same(8.0);
    visuals.widgets.active.rounding = Rounding::same(8.0);
    visuals.widgets.open.rounding = Rounding::same(8.0);
    visuals.window_rounding = Rounding::same(12.0);
    visuals.menu_rounding = Rounding::same(8.0);

    // Modernize colors for Dark Mode
    if visuals.dark_mode {
        visuals.widgets.noninteractive.bg_fill = Color32::from_gray(32); // Slightly lighter background for panels
        visuals.window_fill = Color32::from_gray(20); // Darker window background
        visuals.panel_fill = Color32::from_gray(28); 
        
        // Button Styling
        visuals.widgets.inactive.weak_bg_fill = Color32::from_gray(45);
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_gray(60));
        
        visuals.widgets.hovered.weak_bg_fill = Color32::from_gray(60);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_gray(100));

        visuals.selection.bg_fill = Color32::from_rgb(0, 122, 255); // System Blue
        visuals.hyperlink_color = Color32::from_rgb(58, 150, 255);
    } else {
        // Modernize colors for Light Mode
        visuals.widgets.noninteractive.bg_fill = Color32::from_gray(248);
        visuals.window_fill = Color32::WHITE;
        visuals.panel_fill = Color32::from_gray(242);
        
         // Button Styling
        visuals.widgets.inactive.weak_bg_fill = Color32::from_gray(230);
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_gray(200));
        
        visuals.selection.bg_fill = Color32::from_rgb(0, 122, 255);
    }

    ctx.set_style(style);
    ctx.set_visuals(visuals);
    ctx.set_fonts(fonts); // Actually this resets fonts if we don't be careful, but default is fine.
}
