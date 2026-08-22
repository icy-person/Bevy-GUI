use bevy_egui::egui;

use crate::{
    command::{EditorCommandBus, EditorCommandId},
    editor::EditorUiState,
    project::{EditorMode, ProjectState},
};

use super::welcome::WelcomeState;

pub fn show_app_bar(
    ui: &mut egui::Ui,
    project: &mut ProjectState,
    state: &mut EditorUiState,
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
                ui.label(project.name.as_str());
                if project.dirty {
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
                    for (mode, label) in [
                        (EditorMode::Play, "▶"),
                        (EditorMode::Paused, "Ⅱ"),
                        (EditorMode::Edit, "■"),
                    ] {
                        if ui
                            .selectable_label(project.mode == mode, label)
                            .on_hover_text(match mode {
                                EditorMode::Play => "Play",
                                EditorMode::Paused => "Pause",
                                EditorMode::Edit => "Stop",
                            })
                            .clicked()
                        {
                            project.mode = mode;
                        }
                    }
                    ui.separator();
                    if state.status.is_empty() {
                        state.status = "Ready".into();
                    }
                    ui.small(state.status.as_str());
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
