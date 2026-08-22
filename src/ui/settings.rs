use bevy_egui::egui;

use crate::{project::ProjectState, settings::{save_settings, EditorSettingsState}};

pub fn show_settings(ui: &mut egui::Ui, settings: &mut EditorSettingsState, project: &ProjectState) {
    ui.heading("Settings");
    ui.label("Editor-wide preferences are stored in .bevy-gui/editor-settings.json");
    ui.separator();

    egui::CollapsingHeader::new("Appearance")
        .default_open(true)
        .show(ui, |ui| {
            egui::ComboBox::from_label("Theme")
                .selected_text(&settings.settings.appearance.theme)
                .show_ui(ui, |ui| {
                    for theme in ["Material Dark", "Material Light", "System"] {
                        if ui.selectable_label(settings.settings.appearance.theme == theme, theme).clicked() {
                            settings.settings.appearance.theme = theme.into();
                            settings.dirty = true;
                        }
                    }
                });
            ui.add(egui::Slider::new(&mut settings.settings.appearance.ui_scale, 0.75..=1.5).text("UI scale"));
            ui.checkbox(&mut settings.settings.appearance.compact_controls, "Compact controls");
        });

    egui::CollapsingHeader::new("Editor")
        .default_open(true)
        .show(ui, |ui| {
            ui.checkbox(&mut settings.settings.editor.autosave, "Enable autosave");
            ui.add_enabled(
                settings.settings.editor.autosave,
                egui::Slider::new(&mut settings.settings.editor.autosave_seconds, 15.0..=1800.0)
                    .logarithmic(true)
                    .text("Autosave interval (s)"),
            );
            ui.checkbox(&mut settings.settings.editor.confirm_delete, "Confirm entity deletion");
            ui.checkbox(&mut settings.settings.editor.restore_layout, "Restore editor layout");
            ui.checkbox(&mut settings.settings.editor.show_fps, "Show FPS");
            ui.checkbox(&mut settings.settings.editor.start_in_2d, "Start in 2D mode");
        });

    egui::CollapsingHeader::new("Viewport & Grid")
        .default_open(true)
        .show(ui, |ui| {
            ui.checkbox(&mut settings.settings.viewport.grid_2d, "2D grid");
            ui.checkbox(&mut settings.settings.viewport.grid_3d, "3D grid");
            ui.add(egui::Slider::new(&mut settings.settings.viewport.grid_size, 0.1..=10.0).text("Grid size"));
            ui.checkbox(&mut settings.settings.viewport.snap_enabled, "Enable snapping");
            ui.add_enabled(
                settings.settings.viewport.snap_enabled,
                egui::Slider::new(&mut settings.settings.viewport.snap_translation, 0.05..=10.0).text("Translation snap"),
            );
            ui.add_enabled(
                settings.settings.viewport.snap_enabled,
                egui::Slider::new(&mut settings.settings.viewport.snap_rotation_degrees, 1.0..=90.0).text("Rotation snap°"),
            );
            ui.add_enabled(
                settings.settings.viewport.snap_enabled,
                egui::Slider::new(&mut settings.settings.viewport.snap_scale, 0.05..=2.0).text("Scale snap"),
            );
            ui.add(egui::Slider::new(&mut settings.settings.viewport.camera_move_speed, 0.5..=30.0).text("3D move speed"));
            ui.add(egui::Slider::new(&mut settings.settings.viewport.camera_pan_speed, 0.5..=30.0).text("Pan speed"));
            ui.add(egui::Slider::new(&mut settings.settings.viewport.camera_zoom_speed, 0.1..=5.0).text("Zoom speed"));
            ui.add(egui::Slider::new(&mut settings.settings.viewport.camera_orbit_speed, 0.1..=5.0).text("Orbit speed"));
        });

    egui::CollapsingHeader::new("Input")
        .default_open(false)
        .show(ui, |ui| {
            let input = &mut settings.settings.input;
            for (label, value) in [
                ("Forward", &mut input.forward),
                ("Backward", &mut input.backward),
                ("Left", &mut input.left),
                ("Right", &mut input.right),
                ("Up", &mut input.up),
                ("Down", &mut input.down),
                ("Focus", &mut input.focus),
                ("Duplicate", &mut input.duplicate),
                ("Delete", &mut input.delete),
                ("Save", &mut input.save),
            ] {
                ui.horizontal(|ui| {
                    ui.label(label);
                    ui.text_edit_singleline(value);
                });
            }
        });

    egui::CollapsingHeader::new("Graphics")
        .default_open(false)
        .show(ui, |ui| {
            ui.label(format!("Backend: {}", settings.settings.graphics.backend));
            egui::ComboBox::from_label("MSAA")
                .selected_text(format!("{}x", settings.settings.graphics.msaa_samples))
                .show_ui(ui, |ui| {
                    for samples in [1, 2, 4, 8] {
                        if ui.selectable_label(settings.settings.graphics.msaa_samples == samples, format!("{}x", samples)).clicked() {
                            settings.settings.graphics.msaa_samples = samples;
                            settings.dirty = true;
                        }
                    }
                });
            ui.checkbox(&mut settings.settings.graphics.vsync, "VSync");
            ui.checkbox(&mut settings.settings.graphics.hdr, "HDR");
            ui.add(egui::Slider::new(&mut settings.settings.graphics.render_scale, 0.5..=2.0).text("Render scale"));
        });

    egui::CollapsingHeader::new("Project")
        .default_open(false)
        .show(ui, |ui| {
            ui.label(format!("Root: {}", project.root.display()));
            ui.horizontal(|ui| {
                ui.label("Main scene");
                ui.text_edit_singleline(&mut settings.settings.project.main_scene);
            });
            ui.horizontal(|ui| {
                ui.label("Assets directory");
                ui.text_edit_singleline(&mut settings.settings.project.assets_directory);
            });
            ui.horizontal(|ui| {
                ui.label("Build directory");
                ui.text_edit_singleline(&mut settings.settings.project.build_directory);
            });
        });

    ui.separator();
    ui.horizontal(|ui| {
        if ui.button("Save Settings").clicked() {
            match save_settings(&project.root, &settings.settings) {
                Ok(()) => {
                    settings.dirty = false;
                    settings.last_error = None;
                    settings.path = Some(crate::settings::settings_path(&project.root));
                }
                Err(error) => settings.last_error = Some(error.to_string()),
            }
        }
        if ui.button("Reset to Defaults").clicked() {
            let preserved_path = settings.path.clone();
            settings.settings = Default::default();
            settings.dirty = true;
            settings.path = preserved_path;
        }
    });

    if let Some(error) = &settings.last_error {
        ui.colored_label(egui::Color32::from_rgb(255, 130, 130), error);
    } else if settings.dirty {
        ui.weak("Unsaved settings changes");
    } else if settings.path.is_some() {
        ui.weak("Settings saved");
    }
}
