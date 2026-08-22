use bevy::prelude::*;
use bevy_egui::egui;
use egui_dock::{DockArea, TabViewer};

use crate::{
    editor::EditorUiState,
    project::{EditorMode, ProjectState},
    selection::SelectionState,
};

use super::state::{EditorDockState, EditorTab, TransformEdit};

pub struct DockViewer<'a> {
    pub project: &'a mut ProjectState,
    pub selection: &'a mut SelectionState,
    pub ui_state: &'a mut EditorUiState,
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
            EditorTab::Plugins => self.show_plugins(ui),
        }
    }

    fn clear_background(&self, tab: &Self::Tab) -> bool {
        !matches!(tab, Self::Tab::Viewport)
    }

    fn scroll_bars(&self, tab: &Self::Tab) -> [bool; 2] {
        if matches!(tab, Self::Tab::Viewport) {
            [false, false]
        } else {
            [true, true]
        }
    }

    fn is_closeable(&self, tab: &Self::Tab) -> bool {
        !matches!(tab, Self::Tab::Viewport)
    }
}

impl DockViewer<'_> {
    fn show_viewport(&mut self, ui: &mut egui::Ui) {
        self.viewport_focused = true;
        ui.horizontal(|ui| {
            ui.strong("Scene");
            ui.separator();
            for (mode, label) in [
                (EditorMode::Edit, "Edit"),
                (EditorMode::Play, "Play"),
                (EditorMode::Paused, "Pause"),
            ] {
                if ui
                    .selectable_label(self.project.mode == mode, label)
                    .clicked()
                {
                    self.project.mode = mode;
                }
            }
            ui.separator();
            ui.label("W Translate   E Rotate   R Scale   X World/Local   Ctrl+Z/Y");
            ui.separator();
            ui.label(format!("{} selected", self.selection.entities.len()));
        });
        ui.separator();
        ui.centered_and_justified(|ui| {
            ui.vertical_centered(|ui| {
                ui.heading("3D Viewport");
                ui.label("Live Bevy world, FreeCamera and InfiniteGrid are active.");
                ui.label("Click geometry to select; use the gizmo to author transforms.");
            });
        });
    }

    fn show_hierarchy(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Scene Hierarchy");
            if ui.small_button("+").clicked() {
                self.create_entity = true;
            }
            if self.selection.entities.len() >= 2 {
                if ui.small_button("Parent").clicked() {
                    self.parent_selected = true;
                }
                if ui.small_button("Unparent").clicked() {
                    self.unparent_selected = true;
                }
            }
            if let Some(primary) = self.selection.primary() {
                if ui.small_button("Duplicate").clicked() {
                    self.duplicate_entity = Some(primary);
                }
                if ui.small_button("Delete").clicked() {
                    self.delete_entity = Some(primary);
                }
            }
        });
        ui.separator();
        for (entity, name) in self.entities {
            let selected = self.selection.contains(*entity);
            let depth = self.depth(*entity, 0);
            let label = format!("{}{}", "  ".repeat(depth), name);
            let response = ui.selectable_label(selected, label);
            if response.clicked() {
                let ctrl = ui.input(|input| input.modifiers.ctrl);
                if ctrl {
                    self.selection.toggle(*entity);
                } else {
                    self.selection.select(*entity);
                }
            }
        }
    }

    fn depth(&self, entity: Entity, mut depth: usize) -> usize {
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
            depth += 1;
            current = parent;
        }
        depth.min(16)
    }

    fn show_inspector(&mut self, ui: &mut egui::Ui) {
        ui.heading("Inspector");
        ui.separator();
        if let Some(mut edit) = self.selected_transform {
            ui.label(format!("Primary Entity {:?}", edit.entity));
            if let Some(mut name) = self.selected_name.clone() {
                ui.horizontal(|ui| {
                    ui.label("Name");
                    ui.text_edit_singleline(&mut name);
                    if ui.button("Apply").clicked() {
                        self.name_edit = Some(name.clone());
                    }
                });
            }
            ui.separator();
            ui.collapsing("Transform", |ui| {
                for (label, value) in [
                    ("Translation", &mut edit.translation),
                    ("Rotation", &mut edit.rotation),
                    ("Scale", &mut edit.scale),
                ] {
                    ui.label(label);
                    ui.horizontal(|ui| {
                        ui.add(egui::DragValue::new(&mut value.x).speed(0.05));
                        ui.add(egui::DragValue::new(&mut value.y).speed(0.05));
                        ui.add(egui::DragValue::new(&mut value.z).speed(0.05));
                    });
                }
            });
            if ui.button("Apply Transform").clicked() {
                self.transform_edit = Some(edit);
            } else {
                self.selected_transform = Some(edit);
            }
            ui.separator();
            ui.collapsing("Components", |ui| {
                ui.label("Transform");
                ui.label("Name");
                ui.label("Mesh3d / Material references are editor-owned components.");
            });
        } else {
            ui.weak("Nothing selected");
        }
    }

    fn show_assets(&mut self, ui: &mut egui::Ui) {
        ui.heading("Asset Browser");
        ui.separator();
        ui.label(format!("{} discovered files", self.assets.len()));
        egui::ScrollArea::vertical().show(ui, |ui| {
            for path in self.assets.iter().take(1000) {
                let _ = ui.selectable_label(false, path);
            }
        });
    }

    fn show_console(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Console");
            if ui.small_button("Save Scene").clicked() {
                self.save_requested = true;
            }
        });
        ui.separator();
        ui.monospace("[info] plugin-first editor kernel online");
        ui.monospace(format!("[info] {} commands registered", self.command_count));
        ui.monospace(format!("[info] {} plugins installed", self.plugin_names.len()));
        ui.monospace(format!("[info] {} selected", self.selection.entities.len()));
        if self.project.dirty {
            ui.monospace("[warn] current scene has unsaved changes");
        }
        ui.monospace(format!("[info] status: {}", self.ui_state.status));
    }

    fn show_plugins(&mut self, ui: &mut egui::Ui) {
        ui.heading("Plugins");
        ui.separator();
        for plugin in self.plugin_names {
            ui.label(plugin);
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
