use bevy_egui::egui;

use crate::settings::EditorSettings;

pub fn apply_material_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style_of(egui::Theme::Dark)).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(12.0, 8.0);
    style.spacing.window_margin = egui::Margin::same(16);
    style.visuals = egui::Visuals::dark();
    apply_dark_palette(&mut style.visuals);
    ctx.set_style_of(egui::Theme::Dark, style);
}

pub fn apply_material_settings(ctx: &egui::Context, settings: &EditorSettings) {
    let theme = match settings.appearance.theme.as_str() {
        "Material Light" => egui::Theme::Light,
        _ => egui::Theme::Dark,
    };
    let mut style = (*ctx.style_of(theme)).clone();
    style.visuals = match theme {
        egui::Theme::Light => egui::Visuals::light(),
        egui::Theme::Dark => egui::Visuals::dark(),
    };

    match theme {
        egui::Theme::Dark => apply_dark_palette(&mut style.visuals),
        egui::Theme::Light => apply_light_palette(&mut style.visuals),
    }

    let [r, g, b] = settings.appearance.accent;
    let accent = egui::Color32::from_rgb(r, g, b);
    style.visuals.selection.bg_fill = accent;
    style.visuals.selection.stroke = egui::Stroke::new(1.0, accent.gamma_multiply(1.35));
    if settings.appearance.compact_controls {
        style.spacing.item_spacing = egui::vec2(6.0, 5.0);
        style.spacing.button_padding = egui::vec2(10.0, 6.0);
    }

    ctx.set_style_of(theme, style);
    ctx.set_pixels_per_point(settings.appearance.ui_scale.clamp(0.75, 1.5));
}

fn apply_dark_palette(visuals: &mut egui::Visuals) {
    visuals.panel_fill = egui::Color32::from_rgb(18, 18, 22);
    visuals.window_fill = egui::Color32::from_rgb(22, 22, 27);
    visuals.extreme_bg_color = egui::Color32::from_rgb(12, 12, 15);
    visuals.faint_bg_color = egui::Color32::from_rgb(28, 28, 34);
    visuals.override_text_color = Some(egui::Color32::from_rgb(232, 227, 236));
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(42, 42, 49);
    visuals.widgets.inactive.fg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(205, 198, 210));
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(61, 58, 68);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(79, 73, 88);
    visuals.widgets.open.bg_fill = egui::Color32::from_rgb(61, 58, 68);
}

fn apply_light_palette(visuals: &mut egui::Visuals) {
    visuals.panel_fill = egui::Color32::from_rgb(247, 245, 249);
    visuals.window_fill = egui::Color32::from_rgb(255, 251, 255);
    visuals.extreme_bg_color = egui::Color32::from_rgb(238, 235, 240);
    visuals.faint_bg_color = egui::Color32::from_rgb(232, 228, 234);
    visuals.override_text_color = Some(egui::Color32::from_rgb(38, 35, 40));
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(235, 231, 237);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(78, 73, 80));
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(224, 218, 227);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(212, 205, 217);
    visuals.widgets.open.bg_fill = egui::Color32::from_rgb(224, 218, 227);
}

pub fn surface(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::group(ui.style())
        .fill(ui.style().visuals.widgets.inactive.bg_fill)
        .stroke(egui::Stroke::new(1.0, ui.style().visuals.widgets.noninteractive.fg_stroke.color.gamma_multiply(0.35)))
        .inner_margin(egui::Margin::same(12))
        .show(ui, add_contents);
}
