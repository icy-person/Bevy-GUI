use std::path::PathBuf;

use bevy::prelude::*;
use bevy_egui::egui;

use crate::{editor::EditorUiState, project::{load_project, ProjectState}};

#[derive(Resource, Debug, Clone)]
pub struct WelcomeState {
    pub visible: bool,
    pub project_path: String,
}

impl Default for WelcomeState {
    fn default() -> Self {
        Self {
            visible: true,
            project_path: ".".into(),
        }
    }
}

pub fn show_welcome(
    ui: &mut egui::Ui,
    welcome: &mut WelcomeState,
    project: &mut ProjectState,
    editor: &mut EditorUiState,
) {
    let available = ui.available_size();
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(18, 18, 22))
        .show(ui, |ui| {
            ui.set_min_size(available);
            ui.vertical_centered(|ui| {
                ui.add_space(70.0);
                ui.heading(egui::RichText::new("Bevy-GUI").size(34.0).strong());
                ui.label(egui::RichText::new("A Material 3 game editor for Bevy").size(17.0));
                ui.add_space(30.0);

                ui.horizontal(|ui| {
                    egui::Frame::group(ui.style())
                        .fill(egui::Color32::from_rgb(31, 31, 38))
                        .inner_margin(egui::Margin::same(20))
                        .show(ui, |ui| {
                            ui.set_width(280.0);
                            ui.heading("Create");
                            ui.add_space(8.0);
                            ui.label("Start a clean Bevy project workspace.");
                            ui.add_space(16.0);
                            if ui.button("＋ New Project").clicked() {
                                *project = ProjectState::default();
                                project.name = "New Project".into();
                                project.root = PathBuf::from(".");
                                editor.status = "New project workspace ready".into();
                                welcome.visible = false;
                            }
                        });
                    egui::Frame::group(ui.style())
                        .fill(egui::Color32::from_rgb(31, 31, 38))
                        .inner_margin(egui::Margin::same(20))
                        .show(ui, |ui| {
                            ui.set_width(280.0);
                            ui.heading("Open");
                            ui.add_space(8.0);
                            ui.label("Open a folder containing project.godot-rs.json");
                            ui.add_space(8.0);
                            ui.text_edit_singleline(&mut welcome.project_path);
                            if ui.button("Open Project").clicked() {
                                match load_project(PathBuf::from(&welcome.project_path).as_path()) {
                                    Ok(loaded) => {
                                        *project = loaded;
                                        editor.status = "Project loaded".into();
                                        welcome.visible = false;
                                    }
                                    Err(error) => {
                                        editor.status = format!("Open failed: {error}");
                                    }
                                }
                            }
                        });
                });

                ui.add_space(26.0);
                egui::Frame::group(ui.style())
                    .fill(egui::Color32::from_rgb(24, 24, 30))
                    .inner_margin(egui::Margin::same(16))
                    .show(ui, |ui| {
                        ui.set_width(590.0);
                        ui.heading("Quick Start");
                        ui.label("Viewport • Hierarchy • Inspector • Assets • Console • Plugins");
                        ui.add_space(8.0);
                        ui.small("Use this screen as the project entry point; the full editor opens after project creation or loading.");
                    });

                ui.add_space(24.0);
                ui.label(egui::RichText::new("Material 3 inspired • plugin-first • Bevy 0.19").weak());
            });
        });
}
