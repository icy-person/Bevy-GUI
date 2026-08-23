use bevy_egui::egui;

use crate::settings::EditorSettings;

pub const SPACING_XS: f32 = 4.0;
pub const SPACING_SM: f32 = 8.0;
pub const SPACING_MD: f32 = 12.0;
pub const SPACING_LG: f32 = 16.0;
pub const SPACING_XL: f32 = 24.0;

pub fn apply_material_theme(ctx: &egui::Context) {
    apply_material_style(ctx, egui::Theme::Dark, &EditorSettings::default());
}

pub fn apply_material_settings(ctx: &egui::Context, settings: &EditorSettings) {
    let theme = match settings.appearance.theme.as_str() {
        "Material Light" => egui::Theme::Light,
        _ => egui::Theme::Dark,
    };
    apply_material_style(ctx, theme, settings);
    ctx.set_pixels_per_point(settings.appearance.ui_scale.clamp(0.75, 1.5));
}

fn apply_material_style(ctx: &egui::Context, theme: egui::Theme, settings: &EditorSettings) {
    let mut style = (*ctx.style_of(theme)).clone();
    style.spacing.item_spacing = if settings.appearance.compact_controls {
        egui::vec2(6.0, 5.0)
    } else {
        egui::vec2(8.0, 8.0)
    };
    style.spacing.button_padding = if settings.appearance.compact_controls {
        egui::vec2(10.0, 6.0)
    } else {
        egui::vec2(12.0, 8.0)
    };
    style.spacing.menu_margin = egui::Margin::same(8);
    style.spacing.window_margin = egui::Margin::same(16);
    style.interaction.selectable_labels = true;
    style.visuals = match theme {
        egui::Theme::Dark => egui::Visuals::dark(),
        egui::Theme::Light => egui::Visuals::light(),
    };

    match theme {
        egui::Theme::Dark => apply_dark_palette(&mut style.visuals),
        egui::Theme::Light => apply_light_palette(&mut style.visuals),
    }

    let [r, g, b] = settings.appearance.accent;
    let accent = egui::Color32::from_rgb(r, g, b);
    style.visuals.selection.bg_fill = accent.gamma_multiply(0.75);
    style.visuals.selection.stroke = egui::Stroke::new(1.0, accent);
    style.visuals.hyperlink_color = accent;
    style.visuals.widgets.hovered.bg_fill = accent.gamma_multiply(0.15);
    style.visuals.widgets.active.bg_fill = accent.gamma_multiply(0.22);
    style.visuals.widgets.open.bg_fill = accent.gamma_multiply(0.18);

    ctx.set_style_of(theme, style);
}

fn apply_dark_palette(visuals: &mut egui::Visuals) {
    visuals.panel_fill = egui::Color32::from_rgb(16, 17, 21);
    visuals.window_fill = egui::Color32::from_rgb(20, 21, 26);
    visuals.extreme_bg_color = egui::Color32::from_rgb(10, 11, 14);
    visuals.faint_bg_color = egui::Color32::from_rgb(27, 28, 34);
    visuals.code_bg_color = egui::Color32::from_rgb(13, 14, 18);
    visuals.override_text_color = Some(egui::Color32::from_rgb(235, 231, 239));
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(31, 32, 39);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(205, 200, 211));
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(48, 48, 58);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(58, 57, 70);
    visuals.widgets.open.bg_fill = egui::Color32::from_rgb(48, 48, 58);
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(23, 24, 30);
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(175, 170, 181));
    visuals.window_shadow = egui::Shadow {
        offset: [0, 10],
        blur: 24,
        spread: 0,
        color: egui::Color32::from_black_alpha(110),
    };
}

fn apply_light_palette(visuals: &mut egui::Visuals) {
    visuals.panel_fill = egui::Color32::from_rgb(247, 245, 249);
    visuals.window_fill = egui::Color32::from_rgb(255, 251, 255);
    visuals.extreme_bg_color = egui::Color32::from_rgb(238, 235, 240);
    visuals.faint_bg_color = egui::Color32::from_rgb(231, 228, 234);
    visuals.code_bg_color = egui::Color32::from_rgb(242, 239, 244);
    visuals.override_text_color = Some(egui::Color32::from_rgb(32, 30, 34));
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(237, 233, 239);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(77, 72, 80));
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(225, 219, 228);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(215, 208, 218);
    visuals.widgets.open.bg_fill = egui::Color32::from_rgb(225, 219, 228);
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(249, 246, 250);
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(83, 77, 86));
    visuals.window_shadow = egui::Shadow {
        offset: [0, 8],
        blur: 20,
        spread: 0,
        color: egui::Color32::from_black_alpha(55),
    };
}

pub fn surface(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    let visuals = &ui.style().visuals;
    egui::Frame::new()
        .fill(visuals.widgets.noninteractive.bg_fill)
        .stroke(egui::Stroke::new(1.0, visuals.widgets.noninteractive.fg_stroke.color.gamma_multiply(0.35)))
        .corner_radius(egui::CornerRadius::same(12))
        .inner_margin(egui::Margin::same(12))
        .show(ui, add_contents);
}
