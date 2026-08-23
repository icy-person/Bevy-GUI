use bevy::prelude::*;
use bevy_egui::egui;
use egui_dock::{DockArea, TabViewer};

use crate::{
    editor::{EditorUiState, ViewportMode},
    profiler::EditorProfiler,
    project::{EditorMode, ProjectState},
    selection::SelectionState,
    settings::EditorSettingsState,
};

use super::state::{EditorDockState, EditorTab, TransformEdit};

pub struct DockViewer<'a> {
    pub project: &'a mut ProjectState,
    pub selection: &'a mut SelectionState,
    pub ui_state: &'a mut EditorUiState,
    pub settings: &'a mut EditorSettingsState,
    pub profiler: &'a EditorProfiler,
    pub entities: &'a [(Entity, String)],
    pub parents: &'a [(Entity, Option<Entity>)],
    pub selected_transform: Option<TransformEdit>,
    pub selected_name: Option<String>,
    pub assets: &'a [String],
    pub plugin_names: &'a [String],
    pub command_count: usize,
    pub transform_edit: Option<TransformEdit>,
    pub name_edit: Option<String>,
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
            EditorTab::Settings => crate::ui::settings::show_settings(ui, self.settings, self.project),
        }
    }

    fn clear_background(&self, tab: &Self::Tab) -> bool {
        !matches!(tab, Self::Tab::Viewport)
    }

    fn scroll_bars(&self, tab: &Self::Tab) -> [bool; 2] {
        if matches!(tab, Self::Tab::Viewport) { [false, false] } else { [true, true] }
    }

    fn is_closeable(&self, tab: &Self::Tab) -> bool {
        !matches!(tab, Self::Tab::Viewport)
    }
}

impl DockViewer<'_> {
    fn heading(ui: &mut egui::Ui, title: &str, subtitle: &str) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(title).size(17.0).strong());
            ui.add_space(6.0);
            ui.label(egui::RichText::new(subtitle).weak());
        });
    }

    fn section(ui: &mut egui::Ui, title: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
        egui::Frame::group(ui.style())
            .fill(egui::Color32::from_rgb(29, 29, 36))
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(49, 49, 58)))
            .inner_margin(egui::Margin::same(10))
            .show(ui, |ui| {
                ui.label(egui::RichText::new(title).strong());
                ui.add_space(5.0);
                add_contents(ui);
            });
    }

    fn show_viewport(&mut self, ui: &mut egui::Ui) {
        self.viewport_focused = true;
        egui::Frame::new()
            .fill(egui::Color32::from_rgb(13, 13, 17))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Scene View").strong());
                    ui.separator();
                    for (mode, label) in [(ViewportMode::TwoD, "2D"), (ViewportMode::ThreeD, "3D")] {
                        if ui.selectable_label(self.ui_state.viewport_mode == mode, label).clicked() {
                            self.ui_state.viewport_mode = mode;
                            self.project.dirty = true;
                        }
                    }
                    ui.separator();
                    for (mode, label) in [
                        (EditorMode::Edit, "Edit"),
                        (EditorMode::Play, "Play"),
                        (EditorMode::Paused, "Pause"),
                    ] {
                        if ui.selectable_label(self.project.mode == mode, label).clicked() {
                            self.project.mode = mode;
                        }
                    }
                    ui.separator();
                    if self.ui_state.viewport_mode == ViewportMode::TwoD {
                        ui.weak("Middle drag = pan   Wheel = zoom   Grid + snap configurable in Settings");
                    } else {
                        ui.weak("W Move   E Rotate   R Scale   X World/Local   Ctrl+Z/Y");
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(format!("{} selected", self.selection.entities.len()));
                    });
                });
                ui.separator();
                ui.centered_and_justified(|ui| {
                    egui::Frame::group(ui.style())
                        .fill(egui::Color32::from_rgb(24, 24, 30))
                        .inner_margin(egui::Margin::same(20))
                        .show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                if self.ui_state.viewport_mode == ViewportMode::TwoD {
                                    ui.heading("2D Viewport");
                                    ui.label("Orthographic camera • grid • pan • zoom • 2D authoring");
                                    ui.small("Middle mouse pans. Mouse wheel zooms. Scene objects remain shared with the editor hierarchy.");
                                } else {
                                    ui.heading("3D Viewport");
                                    ui.label("FreeCamera • InfiniteGrid • Picking • Transform Gizmo");
                                    ui.small("Select scene entities and author transforms directly in the Bevy world.");
                                }
                            });
                        });
                });
            });
    }

    fn show_hierarchy(&mut self, ui: &mut egui::Ui) {
        Self::heading(ui, "Scene Hierarchy", "Entities and parent relationships");
        ui.horizontal(|ui| {
            if ui.button("＋ Entity").clicked() { self.create_entity = true; }
            if self.selection.entities.len() >= 2 {
                if ui.small_button("Parent").clicked() { self.parent_selected = true; }
                if ui.small_button("Unparent").clicked() { self.unparent_selected = true; }
            }
            if let Some(primary) = self.selection.primary() {
                if ui.small_button("Duplicate").clicked() { self.duplicate_entity = Some(primary); }
                if ui.small_button("Delete").clicked() { self.delete_entity = Some(primary); }
            }
        });
        ui.separator();
        Self::section(ui, "Entities", |ui| {
            for (entity, name) in self.entities {
                let selected = self.selection.contains(*entity);
                let depth = self.depth(*entity, 0);
                let response = ui.selectable_label(selected, format!("{}{}", "  ".repeat(depth), name));
                if response.clicked() {
                    if ui.input(|input| input.modifiers.ctrl) { self.selection.toggle(*entity); }
                    else { self.selection.select(*entity); }
                }
            }
        });
    }

    fn depth(&self, entity: Entity, mut depth: usize) -> usize {
        let mut current = entity;
        for _ in 0..64 {
            let parent = self.parents.iter().find(|(candidate, _)| *candidate == current).and_then(|(_, parent)| *parent);
            let Some(parent) = parent else { break };
            depth += 1;
            current = parent;
        }
        depth.min(16)
    }

    fn show_inspector(&mut self, ui: &mut egui::Ui) {
        Self::heading(ui, "Inspector", "Selected entity properties");
        ui.separator();
        if let Some(mut edit) = self.selected_transform {
            Self::section(ui, "Identity", |ui| {
                ui.label(format!("Entity {:?}", edit.entity));
                if let Some(mut name) = self.selected_name.clone() {
                    ui.horizontal(|ui| {
                        ui.label("Name");
                        ui.text_edit_singleline(&mut name);
                        if ui.small_button("Apply").clicked() { self.name_edit = Some(name.clone()); }
                    });
                }
            });
            ui.add_space(8.0);
            Self::section(ui, "Transform", |ui| {
                for (label, value) in [("Position", &mut edit.translation), ("Rotation", &mut edit.rotation), ("Scale", &mut edit.scale)] {
                    ui.label(egui::RichText::new(label).strong());
                    ui.horizontal(|ui| {
                        ui.add(egui::DragValue::new(&mut value.x).speed(0.05));
                        ui.add(egui::DragValue::new(&mut value.y).speed(0.05));
                        ui.add(egui::DragValue::new(&mut value.z).speed(0.05));
                    });
                }
                if ui.button("Apply Transform").clicked() { self.transform_edit = Some(edit); }
            });
            ui.add_space(8.0);
            Self::section(ui, "Components", |ui| {
                ui.label("Transform");
                ui.label("Name");
                ui.label("Mesh3d / Material");
                ui.label("2D Sprite / Mesh2d");
            });
        } else {
            ui.centered_and_justified(|ui| ui.weak("Select an entity to inspect it"));
        }
    }

    fn show_assets(&mut self, ui: &mut egui::Ui) {
        Self::heading(ui, "Asset Browser", "Project files and imported resources");
        ui.separator();
        Self::section(ui, "Library", |ui| {
            ui.label(format!("{} discovered files", self.assets.len()));
            for path in self.assets.iter().take(1000) { let _ = ui.selectable_label(false, path); }
        });
    }

    fn show_console(&mut self, ui: &mut egui::Ui) {
        Self::heading(ui, "Console", "Editor and runtime messages");
        ui.separator();
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.monospace("[info] plugin-first editor kernel online");
            ui.monospace(format!("[info] {} commands registered", self.command_count));
            ui.monospace(format!("[info] {} plugins installed", self.plugin_names.len()));
            ui.monospace(format!("[info] {} selected", self.selection.entities.len()));
            if self.project.dirty { ui.monospace("[warn] current scene has unsaved changes"); }
            ui.monospace(format!("[info] status: {}", self.ui_state.status));
        });
        if ui.button("Save Scene").clicked() { self.save_requested = true; }
    }

    fn show_profiler(&mut self, ui: &mut egui::Ui) {
        Self::heading(ui, "Profiler", "Live editor frame timing");
        ui.separator();
        Self::section(ui, "Frame", |ui| {
            ui.horizontal(|ui| {
                ui.label("FPS");
                ui.strong(format!("{:.1}", self.profiler.fps));
                ui.separator();
                ui.label("Frame time");
                ui.strong(format!("{:.2} ms", self.profiler.frame_time_ms));
            });
            ui.add(
                egui::ProgressBar::new((self.profiler.frame_time_ms / 33.3).clamp(0.0, 1.0))
                    .text("16.7ms = 60 FPS target"),
            );
        });
        Self::section(ui, "Window statistics", |ui| {
            ui.label(format!("Minimum: {:.2} ms", self.profiler.min_frame_ms.min(9999.0)));
            ui.label(format!("Maximum: {:.2} ms", self.profiler.max_frame_ms));
            ui.label(format!("Samples: {}", self.profiler.samples));
        });
    }

    fn show_plugins(&mut self, ui: &mut egui::Ui) {
        Self::heading(ui, "Plugins", "Editor extensions and services");
        ui.separator();
        for plugin in self.plugin_names {
            egui::Frame::group(ui.style()).show(ui, |ui| ui.horizontal(|ui| { ui.label("✦"); ui.label(plugin); }));
            ui.add_space(5.0);
        }
    }
}

pub fn show_dock_area(ui: &mut egui::Ui, dock: &mut EditorDockState, viewer: &mut DockViewer<'_>) {
    DockArea::new(&mut dock.state)
        .show_add_buttons(true)
        .show_add_popup(true)
        .show_close_buttons(true)
        .show_inside(ui, viewer);
}
