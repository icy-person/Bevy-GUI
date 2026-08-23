use bevy_egui::egui;

use crate::settings::EditorSettings;

pub fn apply_material_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style_of(egui::Theme::Dark)).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(12.0, 8.0);
    style.spacing.window_margin = egui::Margin::same(16);
    style.visuals = egui::Visuals::dark();
    style.visuals.panel_fill = egui::Color32::from_rgb(18, 18, 22);
    style.visuals.window_fill = egui::Color32::from_rgb(22, 22, 27);
    style.visuals.extreme_bg_color = egui::Color32::from_rgb(12, 12, 15);
    style.visuals.faint_bg_color = egui::Color32::from_rgb(28, 28, 34);
    style.visuals.override_text_color = Some(egui::Color32::from_rgb(232, 227, 236));
    style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(42, 42, 49);
    style.visuals.widgets.inactive.fg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(205, 198, 210));
    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(61, 58, 68);
    style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(79, 73, 88);
    style.visuals.widgets.open.bg_fill = egui::Color32::from_rgb(61, 58, 68);
    style.visuals.selection.bg_fill = egui::Color32::from_rgb(74, 78, 105);
    style.visuals.selection.stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(205, 214, 255));
    ctx.set_style_of(egui::Theme::Dark, style);
}

pub fn apply_material_settings(ctx: &egui::Context, settings: &EditorSettings) {
    let mut style = (*ctx.style_of(egui::Theme::Dark)).clone();
    let [r, g, b] = settings.appearance.accent;
    let accent = egui::Color32::from_rgb(r, g, b);
    style.visuals.selection.bg_fill = accent;
    style.visuals.selection.stroke = egui::Stroke::new(1.0, accent.gamma_multiply(1.35));
    if settings.appearance.compact_controls {
        style.spacing.item_spacing = egui::vec2(6.0, 5.0);
        style.spacing.button_padding = egui::vec2(10.0, 6.0);
    }
    ctx.set_style_of(egui::Theme::Dark, style);
}

pub fn surface(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::group(ui.style())
        .fill(egui::Color32::from_rgb(28, 28, 34))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(54, 54, 63)))
        .inner_margin(egui::Margin::same(12))
        .show(ui, add_contents);
}
