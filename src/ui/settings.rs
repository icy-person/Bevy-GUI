use bevy_egui::egui;

use crate::{
    project::ProjectState,
    settings::{save_settings, EditorSettings, EditorSettingsState},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettingsPage {
    Appearance,
    Editor,
    Viewport,
    Input,
    Graphics,
    Project,
}

pub fn show_settings(ui: &mut egui::Ui, settings: &mut EditorSettingsState, project: &ProjectState) {
    let before = settings.settings.clone();
    let mut page = SettingsPage::Appearance;

    ui.heading("Editor Settings");
    ui.label("All editor preferences are versioned and stored under .bevy-gui/editor-settings.json.");
    ui.add_space(8.0);

    ui.horizontal_top(|ui| {
        egui::Frame::group(ui.style())
            .inner_margin(egui::Margin::same(8))
            .show(ui, |ui| {
                ui.set_width(170.0);
                ui.label(egui::RichText::new("Categories").strong());
                for (candidate, label) in [
                    (SettingsPage::Appearance, "Appearance"),
                    (SettingsPage::Editor, "Editor"),
                    (SettingsPage::Viewport, "Viewport & Grid"),
                    (SettingsPage::Input, "Input"),
                    (SettingsPage::Graphics, "Graphics"),
                    (SettingsPage::Project, "Project"),
                ] {
                    let selected = candidate == page;
                    if ui.selectable_label(selected, label).clicked() {
                        page = candidate;
                    }
                }
            });

        ui.separator();
        ui.allocate_ui(ui.available_size(), |content| match page {
            SettingsPage::Appearance => appearance_page(content, settings),
            SettingsPage::Editor => editor_page(content, settings),
            SettingsPage::Viewport => viewport_page(content, settings),
            SettingsPage::Input => input_page(content, settings),
            SettingsPage::Graphics => graphics_page(content, settings),
            SettingsPage::Project => project_page(content, settings, project),
        });
    });

    if before != settings.settings {
        settings.dirty = true;
        settings.last_error = None;
    }

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
        if ui.button("Reset Defaults").clicked() {
            settings.settings = EditorSettings::default();
            settings.dirty = true;
            settings.last_error = None;
        }
        if settings.dirty {
            ui.colored_label(egui::Color32::from_rgb(255, 190, 90), "Unsaved");
        } else {
            ui.weak("Saved");
        }
    });

    if let Some(error) = &settings.last_error {
        ui.colored_label(egui::Color32::from_rgb(255, 130, 130), error);
    }
}

fn section(ui: &mut egui::Ui, title: &str, contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::group(ui.style())
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(title).size(16.0).strong());
            ui.add_space(8.0);
            contents(ui);
        });
}

fn appearance_page(ui: &mut egui::Ui, settings: &mut EditorSettingsState) {
    section(ui, "Material appearance", |ui| {
        egui::ComboBox::from_label("Theme")
            .selected_text(&settings.settings.appearance.theme)
            .show_ui(ui, |ui| {
                for theme in ["Material Dark", "Material Light", "System"] {
                    ui.selectable_value(&mut settings.settings.appearance.theme, theme.into(), theme);
                }
            });
        ui.add(egui::Slider::new(&mut settings.settings.appearance.ui_scale, 0.75..=1.5).text("UI scale"));
        ui.checkbox(&mut settings.settings.appearance.compact_controls, "Compact controls");
        let [r, g, b] = settings.settings.appearance.accent;
        let mut rgb = [r as f32, g as f32, b as f32];
        if ui.color_edit_button_rgb(&mut rgb).changed() {
            settings.settings.appearance.accent = [rgb[0] as u8, rgb[1] as u8, rgb[2] as u8];
        }
    });
}

fn editor_page(ui: &mut egui::Ui, settings: &mut EditorSettingsState) {
    section(ui, "Editor behavior", |ui| {
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
}

fn viewport_page(ui: &mut egui::Ui, settings: &mut EditorSettingsState) {
    section(ui, "Grid", |ui| {
        ui.checkbox(&mut settings.settings.viewport.grid_2d, "2D grid");
        ui.checkbox(&mut settings.settings.viewport.grid_3d, "3D grid");
        ui.add(egui::Slider::new(&mut settings.settings.viewport.grid_size, 0.1..=10.0).text("Grid size"));
    });
    ui.add_space(8.0);
    section(ui, "Snapping", |ui| {
        ui.checkbox(&mut settings.settings.viewport.snap_enabled, "Enable snapping");
        ui.add_enabled(
            settings.settings.viewport.snap_enabled,
            egui::Slider::new(&mut settings.settings.viewport.snap_translation, 0.05..=10.0).text("Translation"),
        );
        ui.add_enabled(
            settings.settings.viewport.snap_enabled,
            egui::Slider::new(&mut settings.settings.viewport.snap_rotation_degrees, 1.0..=90.0).text("Rotation degrees"),
        );
        ui.add_enabled(
            settings.settings.viewport.snap_enabled,
            egui::Slider::new(&mut settings.settings.viewport.snap_scale, 0.05..=2.0).text("Scale"),
        );
    });
    ui.add_space(8.0);
    section(ui, "Camera", |ui| {
        ui.add(egui::Slider::new(&mut settings.settings.viewport.camera_move_speed, 0.5..=30.0).text("3D movement"));
        ui.add(egui::Slider::new(&mut settings.settings.viewport.camera_pan_speed, 0.5..=30.0).text("2D pan"));
        ui.add(egui::Slider::new(&mut settings.settings.viewport.camera_zoom_speed, 0.1..=5.0).text("Zoom"));
        ui.add(egui::Slider::new(&mut settings.settings.viewport.camera_orbit_speed, 0.1..=5.0).text("Orbit"));
    });
}

fn input_page(ui: &mut egui::Ui, settings: &mut EditorSettingsState) {
    section(ui, "Editor keymap", |ui| {
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
                ui.add_sized([180.0, 0.0], egui::TextEdit::singleline(value));
            });
        }
    });
}

fn graphics_page(ui: &mut egui::Ui, settings: &mut EditorSettingsState) {
    section(ui, "Renderer", |ui| {
        egui::ComboBox::from_label("Backend")
            .selected_text(&settings.settings.graphics.backend)
            .show_ui(ui, |ui| {
                for backend in ["Vulkan/Auto", "Vulkan", "OpenGL", "Auto"] {
                    ui.selectable_value(&mut settings.settings.graphics.backend, backend.into(), backend);
                }
            });
        egui::ComboBox::from_label("MSAA")
            .selected_text(format!("{}x", settings.settings.graphics.msaa_samples))
            .show_ui(ui, |ui| {
                for samples in [1, 2, 4, 8] {
                    ui.selectable_value(&mut settings.settings.graphics.msaa_samples, samples, format!("{}x", samples));
                }
            });
        ui.checkbox(&mut settings.settings.graphics.vsync, "VSync");
        ui.checkbox(&mut settings.settings.graphics.hdr, "HDR");
        ui.add(egui::Slider::new(&mut settings.settings.graphics.render_scale, 0.5..=2.0).text("Render scale"));
    });
}

fn project_page(ui: &mut egui::Ui, settings: &mut EditorSettingsState, project: &ProjectState) {
    section(ui, "Project paths", |ui| {
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
}
