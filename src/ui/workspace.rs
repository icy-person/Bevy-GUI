use bevy_egui::egui;

use crate::{
    command::{EditorCommandBus, EditorCommandId},
    project::EditorMode,
};

use super::welcome::WelcomeState;

pub fn show_app_bar(
    ui: &mut egui::Ui,
    project_name: &str,
    dirty: bool,
    mode: EditorMode,
    status: &str,
    welcome: &mut WelcomeState,
    commands: &mut EditorCommandBus,
) {
    egui::Frame::new()
        .fill(egui::Color32::from_rgb(23, 23, 28))
        .inner_margin(egui::Margin::symmetric(14, 8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.button("⌂").on_hover_text("Home").clicked() {
                    welcome.visible = true;
                }
                ui.separator();
                ui.strong("Bevy-GUI");
                ui.label("/");
                ui.label(project_name);
                if dirty {
                    ui.label("•");
                    ui.small("Unsaved");
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Build").clicked() {
                        commands.emit(EditorCommandId("project.export"));
                    }
                    if ui.button("Save").clicked() {
                        commands.emit(EditorCommandId("project.save"));
                    }
                    if ui.button("Assets").clicked() {
                        commands.emit(EditorCommandId("assets.refresh"));
                    }
                    ui.separator();
                    for (target_mode, label, command_id, tooltip) in [
                        (EditorMode::Play, "▶", "project.play", "Play"),
                        (EditorMode::Paused, "Ⅱ", "project.pause", "Pause"),
                        (EditorMode::Edit, "■", "project.stop", "Stop"),
                    ] {
                        if ui
                            .selectable_label(mode == target_mode, label)
                            .on_hover_text(tooltip)
                            .clicked()
                        {
                            commands.emit(EditorCommandId(command_id));
                        }
                    }
                    ui.separator();
                    ui.small(if status.is_empty() { "Ready" } else { status });
                });
            });
        });
}

pub fn show_navigation_rail(ui: &mut egui::Ui, welcome: &mut WelcomeState) {
    egui::Frame::new()
        .fill(egui::Color32::from_rgb(24, 24, 30))
        .inner_margin(egui::Margin::symmetric(8, 12))
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new("✦").size(22.0));
                ui.add_space(8.0);
                for (icon, label) in [("⌂", "Home"), ("▣", "Scene"), ("◫", "Assets"), ("⚙", "Settings")] {
                    let button = ui
                        .add_sized([58.0, 48.0], egui::Button::new(egui::RichText::new(icon).size(18.0)))
                        .on_hover_text(label);
                    if label == "Home" && button.clicked() {
                        welcome.visible = true;
                    }
                }
                ui.add_space(ui.available_height().max(0.0));
                ui.add_sized([58.0, 36.0], egui::Button::new("?")).on_hover_text("Help");
            });
        });
}
