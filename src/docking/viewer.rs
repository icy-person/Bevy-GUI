use bevy::prelude::*;
use bevy_egui::egui;
use egui_dock::{DockArea, TabViewer};

use crate::{
    editor::{EditorUiState, ViewportMode},
    profiler::EditorProfiler,
    project::{EditorMode, ProjectState},
    selection::SelectionState,
    settings::EditorSettingsState,
    AssetDatabase,
};

use super::state::{EditorDockState, EditorTab, TransformEdit};

/// The central UI presenter for the editor workspace.
///
/// The viewer deliberately owns UI-local mutations only. World mutations are emitted as
/// `TransformEdit`, selection changes, and entity actions and are applied by the UI system after
/// the docking pass. This keeps egui callbacks out of ECS borrowing paths.
pub struct DockViewer<'a> {
    pub project: &'a mut ProjectState,
    pub selection: &'a mut SelectionState,
    pub ui_state: &'a mut EditorUiState,
    pub settings: &'a mut EditorSettingsState,
    pub profiler: &'a EditorProfiler,
    pub assets: &'a mut AssetDatabase,
    pub entities: &'a [(Entity, String)],
    pub parents: &'a [(Entity, Option<Entity>)],
    pub selected_transform: Option<TransformEdit>,
    pub selected_name: Option<String>,
    pub selected_visible: Option<bool>,
    pub plugin_names: &'a [String],
    pub command_count: usize,
    pub transform_edit: Option<TransformEdit>,
    pub name_edit: Option<String>,
    pub visibility_edit: Option<bool>,
    pub viewport_focused: bool,
    pub create_entity: bool,
    pub delete_entity: Option<Entity>,
    pub duplicate_entity: Option<Entity>,
    pub save_requested: bool,
    pub parent_selected: bool,
    pub unparent_selected: bool,
}

impl TabViewer for DockViewer<'_> {
    type Tab = EditorTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.title().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match *tab {
            EditorTab::Viewport => self.show_viewport(ui),
            EditorTab::Hierarchy => self.show_hierarchy(ui),
            EditorTab::Inspector => self.show_inspector(ui),
            EditorTab::Assets => self.show_assets(ui),
            EditorTab::Console => self.show_console(ui),
            EditorTab::Profiler => self.show_profiler(ui),
            EditorTab::Plugins => self.show_plugins(ui),
            EditorTab::Settings => self.show_settings(ui),
        }
    }

    fn clear_background(&self, tab: &Self::Tab) -> bool {
        !matches!(tab, Self::Tab::Viewport)
    }

    fn scroll_bars(&self, tab: &Self::Tab) -> [bool; 2] {
        match tab {
            Self::Tab::Viewport => [false, false],
            _ => [true, true],
        }
    }

    fn is_closeable(&self, tab: &Self::Tab) -> bool {
        !matches!(tab, Self::Tab::Viewport)
    }
}

impl DockViewer<'_> {
    fn surface(ui: &mut egui::Ui, body: impl FnOnce(&mut egui::Ui)) {
        egui::Frame::group(ui.style())
            .inner_margin(egui::Margin::same(12))
            .show(ui, body);
    }

    fn card(ui: &mut egui::Ui, body: impl FnOnce(&mut egui::Ui)) {
        egui::Frame::new()
            .fill(egui::Color32::from_rgb(30, 30, 37))
            .stroke(egui::Stroke::new(
                1.0,
                egui::Color32::from_rgb(54, 54, 64),
            ))
            .corner_radius(egui::CornerRadius::same(10))
            .inner_margin(egui::Margin::same(10))
            .show(ui, body);
    }

    fn section_header(ui: &mut egui::Ui, title: &str, subtitle: Option<&str>) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(title).size(16.0).strong());
            if let Some(subtitle) = subtitle {
                ui.add_space(6.0);
                ui.label(egui::RichText::new(subtitle).weak());
            }
        });
    }

    fn toolbar_button(ui: &mut egui::Ui, label: &str, selected: bool) -> bool {
        ui.add(
            egui::Button::new(label)
                .selected(selected)
                .min_size(egui::vec2(46.0, 30.0)),
        )
        .clicked()
    }

    fn property_vector(ui: &mut egui::Ui, label: &str, value: &mut Vec3, speed: f32) -> bool {
        let mut changed = false;
        ui.horizontal(|ui| {
            ui.add_sized([68.0, 24.0], egui::Label::new(label));
            ui.label("X");
            changed |= ui
                .add(egui::DragValue::new(&mut value.x).speed(speed))
                .changed();
            ui.label("Y");
            changed |= ui
                .add(egui::DragValue::new(&mut value.y).speed(speed))
                .changed();
            ui.label("Z");
            changed |= ui
                .add(egui::DragValue::new(&mut value.z).speed(speed))
                .changed();
        });
        changed
    }

    fn show_viewport(&mut self, ui: &mut egui::Ui) {
        self.viewport_focused = true;
        egui::Frame::new()
            .fill(egui::Color32::from_rgb(13, 13, 17))
            .show(ui, |ui| {
                self.show_viewport_toolbar(ui);
                ui.separator();
                self.show_viewport_status(ui);
                ui.separator();
                ui.centered_and_justified(|ui| {
                    self.show_viewport_center(ui);
                });
            });
    }

    fn show_viewport_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new("Scene").strong());
            ui.separator();
            if Self::toolbar_button(
                ui,
                "2D",
                self.ui_state.viewport_mode == ViewportMode::TwoD,
            ) {
                self.ui_state.viewport_mode = ViewportMode::TwoD;
            }
            if Self::toolbar_button(
                ui,
                "3D",
                self.ui_state.viewport_mode == ViewportMode::ThreeD,
            ) {
                self.ui_state.viewport_mode = ViewportMode::ThreeD;
            }
            ui.separator();
            if Self::toolbar_button(ui, "Edit", self.project.mode == EditorMode::Edit) {
                self.project.mode = EditorMode::Edit;
            }
            if Self::toolbar_button(ui, "Play", self.project.mode == EditorMode::Play) {
                self.project.mode = EditorMode::Play;
            }
            if Self::toolbar_button(ui, "Pause", self.project.mode == EditorMode::Paused) {
                self.project.mode = EditorMode::Paused;
            }
            ui.separator();
            ui.label("Tools");
            ui.label("W Move");
            ui.label("E Rotate");
            ui.label("R Scale");
            ui.label("X Space");
        });
    }

    fn show_viewport_status(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            let mode = match self.ui_state.viewport_mode {
                ViewportMode::TwoD => "Orthographic 2D",
                ViewportMode::ThreeD => "Perspective 3D",
            };
            ui.label(mode);
            ui.separator();
            ui.label(format!("{} scene entities", self.entities.len()));
            ui.separator();
            ui.label(format!("{} selected", self.selection.entities.len()));
            ui.separator();
            if self.settings.settings.viewport.snap_enabled {
                ui.label("Snap On");
            } else {
                ui.label("Snap Off");
            }
            if self.project.dirty {
                ui.separator();
                ui.label(egui::RichText::new("Unsaved").strong());
            }
        });
    }

    fn show_viewport_center(&mut self, ui: &mut egui::Ui) {
        Self::card(ui, |ui| {
            ui.set_min_width(420.0);
            ui.vertical_centered(|ui| {
                let title = match self.ui_state.viewport_mode {
                    ViewportMode::TwoD => "2D Scene View",
                    ViewportMode::ThreeD => "3D Scene View",
                };
                ui.heading(title);
                ui.add_space(8.0);
                ui.label("Live Bevy rendering occupies the main window behind this editor surface.");
                ui.add_space(8.0);
                match self.ui_state.viewport_mode {
                    ViewportMode::TwoD => {
                        ui.small("Middle mouse: pan   Wheel: zoom   1: 2D   2: 3D");
                        ui.small(format!(
                            "Grid {:.2}   Pan {:.2}   Zoom {:.2}",
                            self.settings.settings.viewport.grid_size,
                            self.settings.settings.viewport.camera_pan_speed,
                            self.settings.settings.viewport.camera_zoom_speed
                        ));
                    }
                    ViewportMode::ThreeD => {
                        ui.small("Right mouse: look   WASD/EQ: fly   W/E/R: transform mode");
                        ui.small(format!(
                            "Grid {:.2}   Move {:.2}   Orbit {:.2}",
                            self.settings.settings.viewport.grid_size,
                            self.settings.settings.viewport.camera_move_speed,
                            self.settings.settings.viewport.camera_orbit_speed
                        ));
                    }
                }
            });
        });
    }

    fn show_hierarchy(&mut self, ui: &mut egui::Ui) {
        Self::section_header(ui, "Scene Hierarchy", Some("Entities"));
        ui.add_space(8.0);
        self.show_hierarchy_toolbar(ui);
        ui.separator();
        Self::surface(ui, |ui| {
            if self.entities.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(30.0);
                    ui.heading("Empty Scene");
                    ui.small("Create an entity to start authoring.");
                    ui.add_space(10.0);
                    if ui.button("Create Entity").clicked() {
                        self.create_entity = true;
                    }
                });
                return;
            }
            for (entity, name) in self.entities {
                self.show_hierarchy_row(ui, *entity, name);
            }
        });
    }

    fn show_hierarchy_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            if ui.button("＋ Entity").clicked() {
                self.create_entity = true;
            }
            if ui.button("Duplicate").clicked() {
                self.duplicate_entity = self.selection.primary();
            }
            if ui.button("Delete").clicked() {
                self.delete_entity = self.selection.primary();
            }
            ui.separator();
            if self.selection.entities.len() >= 2 {
                if ui.button("Parent").clicked() {
                    self.parent_selected = true;
                }
                if ui.button("Unparent").clicked() {
                    self.unparent_selected = true;
                }
            }
        });
    }

    fn show_hierarchy_row(&mut self, ui: &mut egui::Ui, entity: Entity, name: &str) {
        let selected = self.selection.contains(entity);
        let depth = self.entity_depth(entity);
        let indent = (depth as f32 * 16.0).min(160.0);
        ui.horizontal(|ui| {
            ui.add_space(indent);
            let icon = if depth == 0 { "◆" } else { "◇" };
            ui.label(icon);
            let response = ui.selectable_label(selected, name);
            if response.clicked() {
                if ui.input(|input| input.modifiers.shift) {
                    self.selection.toggle(entity);
                } else {
                    self.selection.select(entity);
                }
            }
            if selected {
                ui.label(egui::RichText::new("Selected").small().weak());
            }
        });
    }

    fn entity_depth(&self, entity: Entity) -> usize {
        let mut depth = 0usize;
        let mut current = entity;
        for _ in 0..64 {
            let parent = self
                .parents
                .iter()
                .find(|(candidate, _)| *candidate == current)
                .and_then(|(_, parent)| *parent);
            let Some(parent) = parent else {
                break;
            };
            current = parent;
            depth += 1;
        }
        depth.min(12)
    }

    fn show_inspector(&mut self, ui: &mut egui::Ui) {
        Self::section_header(ui, "Inspector", Some("Entity properties"));
        ui.add_space(8.0);
        let Some(mut edit) = self.selected_transform else {
            Self::surface(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(36.0);
                    ui.heading("Nothing Selected");
                    ui.small("Select an entity in the Scene Hierarchy or Viewport.");
                    ui.add_space(36.0);
                });
            });
            return;
        };
        self.show_identity_section(ui, edit.entity);
        ui.add_space(8.0);
        self.show_visibility_section(ui);
        ui.add_space(8.0);
        self.show_transform_section(ui, &mut edit);
        ui.add_space(8.0);
        self.show_components_section(ui);
    }

    fn show_identity_section(&mut self, ui: &mut egui::Ui, entity: Entity) {
        Self::surface(ui, |ui| {
            Self::section_header(ui, "Identity", Some("Scene object"));
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.add_sized([74.0, 24.0], egui::Label::new("Entity"));
                ui.monospace(format!("{entity:?}"));
            });
            if let Some(name) = self.selected_name.as_mut() {
                ui.horizontal(|ui| {
                    ui.add_sized([74.0, 24.0], egui::Label::new("Name"));
                    let response = ui.text_edit_singleline(name);
                    if response.lost_focus()
                        && ui.input(|input| input.key_pressed(egui::Key::Enter))
                    {
                        self.name_edit = Some(name.clone());
                    }
                });
            }
        });
    }

    fn show_visibility_section(&mut self, ui: &mut egui::Ui) {
        Self::surface(ui, |ui| {
            Self::section_header(ui, "Visibility", Some("Render state"));
            ui.add_space(6.0);
            let mut visible = self.selected_visible.unwrap_or(true);
            if ui.checkbox(&mut visible, "Visible in scene").changed() {
                self.visibility_edit = Some(visible);
            }
        });
    }

    fn show_transform_section(&mut self, ui: &mut egui::Ui, edit: &mut TransformEdit) {
        Self::surface(ui, |ui| {
            Self::section_header(ui, "Transform", Some("Local space"));
            ui.add_space(8.0);
            let mut changed = false;
            changed |= Self::property_vector(ui, "Position", &mut edit.translation, 0.05);
            let mut rotation = edit.rotation;
            changed |= Self::property_vector(ui, "Rotation", &mut rotation, 0.5);
            edit.rotation = rotation;
            changed |= Self::property_vector(ui, "Scale", &mut edit.scale, 0.05);
            if changed {
                self.project.dirty = true;
            }
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                if ui.button("Apply Transform").clicked() {
                    self.transform_edit = Some(*edit);
                }
                if ui.button("Reset Transform").clicked() {
                    self.transform_edit = Some(TransformEdit {
                        entity: edit.entity,
                        translation: Vec3::ZERO,
                        rotation: Vec3::ZERO,
                        scale: Vec3::ONE,
                    });
                }
            });
        });
    }

    fn show_components_section(&mut self, ui: &mut egui::Ui) {
        Self::surface(ui, |ui| {
            Self::section_header(ui, "Components", Some("Editor-visible components"));
            ui.add_space(6.0);
            for (name, detail) in [
                ("Transform", "Translation / Rotation / Scale"),
                ("Name", "Display name"),
                ("Visibility", "Visible / Hidden"),
                ("EditorEntity", "Editor authoring marker"),
            ] {
                ui.horizontal(|ui| {
                    ui.label(name);
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new(detail).weak());
                });
            }
        });
    }

    fn show_assets(&mut self, ui: &mut egui::Ui) {
        Self::section_header(ui, "Asset Browser", Some("Project library"));
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            if ui.button("Refresh").clicked() {
                self.assets.refresh_requested = true;
            }
            ui.label(format!("{} assets", self.assets.entries.len()));
            ui.label(format!("Generation {}", self.assets.generation));
        });
        ui.separator();
        let mut filter = String::new();
        ui.horizontal(|ui| {
            ui.label("Filter");
            ui.text_edit_singleline(&mut filter);
        });
        ui.add_space(6.0);
        Self::surface(ui, |ui| {
            if self.assets.entries.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(30.0);
                    ui.heading("No Assets");
                    ui.small("Place files in the project's assets directory and refresh.");
                });
                return;
            }
            for entry in self.assets.entries.iter().take(1200) {
                let display = entry.path.display().to_string();
                if !filter.is_empty() && !display.to_ascii_lowercase().contains(&filter.to_ascii_lowercase()) {
                    continue;
                }
                ui.horizontal(|ui| {
                    ui.label(asset_icon(entry.kind));
                    ui.label(display);
                    ui.with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            ui.label(format_file_size(entry.bytes));
                        },
                    );
                });
            }
        });
    }

    fn show_console(&mut self, ui: &mut egui::Ui) {
        Self::section_header(ui, "Console", Some("Editor diagnostics"));
        ui.separator();
        Self::surface(ui, |ui| {
            ui.monospace("[info] Bevy-GUI editor kernel online");
            ui.monospace(format!("[info] {} registered commands", self.command_count));
            ui.monospace(format!("[info] {} editor plugins", self.plugin_names.len()));
            ui.monospace(format!("[info] {} selected entities", self.selection.entities.len()));
            ui.monospace(format!("[info] project: {}", self.project.name));
            ui.monospace(format!("[info] viewport: {:?}", self.ui_state.viewport_mode));
            ui.monospace(format!("[info] profiler: {:.1} FPS", self.profiler.fps));
            ui.monospace(format!("[info] frame: {:.2} ms", self.profiler.frame_time_ms));
            if self.project.dirty {
                ui.monospace("[warn] project contains unsaved changes");
            }
            if let Some(error) = &self.settings.last_error {
                ui.monospace(format!("[error] settings: {error}"));
            }
        });
        ui.add_space(8.0);
        if ui.button("Save Scene").clicked() {
            self.save_requested = true;
        }
    }

    fn show_profiler(&mut self, ui: &mut egui::Ui) {
        Self::section_header(ui, "Profiler", Some("Live frame timing"));
        ui.add_space(8.0);
        Self::surface(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new(format!("{:.1} FPS", self.profiler.fps)).size(18.0).strong());
                ui.separator();
                ui.label(format!("{:.2} ms", self.profiler.frame_time_ms));
                ui.separator();
                ui.label(format!("{} samples", self.profiler.samples));
            });
            ui.add_space(8.0);
            let target = 16.6667_f32;
            let fraction = (self.profiler.frame_time_ms / (target * 2.0)).clamp(0.0, 1.0);
            ui.add(egui::ProgressBar::new(fraction).text("Frame budget"));
            ui.add_space(10.0);
            for (label, value) in [
                ("Minimum", self.profiler.min_frame_ms),
                ("Current", self.profiler.frame_time_ms),
                ("Maximum", self.profiler.max_frame_ms),
            ] {
                ui.horizontal(|ui| {
                    ui.label(label);
                    ui.with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            ui.monospace(format!("{value:.2} ms"));
                        },
                    );
                });
            }
        });
    }

    fn show_plugins(&mut self, ui: &mut egui::Ui) {
        Self::section_header(ui, "Plugins", Some("Installed editor extensions"));
        ui.add_space(8.0);
        if self.plugin_names.is_empty() {
            Self::surface(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(30.0);
                    ui.heading("No Plugins");
                });
            });
            return;
        }
        for plugin in self.plugin_names {
            Self::card(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("✦").size(18.0));
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(plugin).strong());
                        ui.label(egui::RichText::new("Registered with the editor kernel").weak());
                    });
                });
            });
            ui.add_space(6.0);
        }
    }

    fn show_settings(&mut self, ui: &mut egui::Ui) {
        Self::section_header(ui, "Settings", Some("Editor configuration"));
        ui.add_space(8.0);
        crate::ui::settings::show_settings(ui, self.settings, self.project);
    }
}

fn asset_icon(kind: crate::assets::AssetKind) -> &'static str {
    match kind {
        crate::assets::AssetKind::Scene => "◫",
        crate::assets::AssetKind::Texture => "▧",
        crate::assets::AssetKind::Mesh => "◇",
        crate::assets::AssetKind::Material => "●",
        crate::assets::AssetKind::Audio => "♫",
        crate::assets::AssetKind::Script => "⌘",
        crate::assets::AssetKind::Data => "▤",
        crate::assets::AssetKind::Other => "•",
    }
}

fn format_file_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    let value = bytes as f64;
    if value >= GB {
        format!("{:.1} GB", value / GB)
    } else if value >= MB {
        format!("{:.1} MB", value / MB)
    } else if value >= KB {
        format!("{:.1} KB", value / KB)
    } else {
        format!("{} B", bytes)
    }
}

pub fn show_dock_area(
    ui: &mut egui::Ui,
    dock: &mut EditorDockState,
    viewer: &mut DockViewer<'_>,
) {
    DockArea::new(&mut dock.state)
        .show_add_buttons(true)
        .show_add_popup(true)
        .show_close_buttons(true)
        .show_inside(ui, viewer);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_icon_is_stable_for_all_kinds() {
        let kinds = [
            crate::assets::AssetKind::Scene,
            crate::assets::AssetKind::Texture,
            crate::assets::AssetKind::Mesh,
            crate::assets::AssetKind::Material,
            crate::assets::AssetKind::Audio,
            crate::assets::AssetKind::Script,
            crate::assets::AssetKind::Data,
            crate::assets::AssetKind::Other,
        ];
        for kind in kinds {
            assert!(!asset_icon(kind).is_empty());
        }
    }

    #[test]
    fn file_size_format_is_human_readable() {
        assert_eq!(format_file_size(0), "0 B");
        assert!(format_file_size(1024).ends_with("KB"));
        assert!(format_file_size(1024 * 1024).ends_with("MB"));
    }
}
