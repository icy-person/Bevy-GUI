use bevy::prelude::*;
use bevy_egui::egui;
use egui_dock::dock_state::tree::NodeIndex;
use egui_dock::{DockArea, DockState, TabViewer};

use crate::{
    editor::EditorUiState,
    project::{EditorMode, ProjectState},
    selection::SelectionState,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EditorTab {
    Viewport,
    Hierarchy,
    Inspector,
    Assets,
    Console,
    Plugins,
}

impl EditorTab {
    pub fn title(self) -> &'static str {
        match self {
            Self::Viewport => "Viewport",
            Self::Hierarchy => "Hierarchy",
            Self::Inspector => "Inspector",
            Self::Assets => "Asset Browser",
            Self::Console => "Console",
            Self::Plugins => "Plugins",
        }
    }
}

#[derive(Resource)]
pub struct EditorDockState {
    pub state: DockState<EditorTab>,
}

impl Default for EditorDockState {
    fn default() -> Self {
        let mut state = DockState::new(vec![EditorTab::Viewport]);
        let tree = state.main_surface_mut();
        let root = NodeIndex::root();
        let [_old, left] = tree.split_left(root, 0.20, vec![EditorTab::Hierarchy]);
        let [_old, right] = tree.split_right(root, 0.20, vec![EditorTab::Inspector]);
        tree.split_below(root, 0.74, vec![EditorTab::Console]);
        tree.split_below(left, 0.65, vec![EditorTab::Assets]);
        tree.split_below(right, 0.65, vec![EditorTab::Plugins]);
        Self { state }
    }
}

#[derive(Clone, Copy)]
pub struct TransformEdit {
    pub entity: Entity,
    pub translation: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
}

pub struct DockViewer<'a> {
    pub project: &'a mut ProjectState,
    pub selection: &'a mut SelectionState,
    pub ui_state: &'a mut EditorUiState,
    pub entities: &'a [(Entity, String)],
    pub selected_transform: Option<TransformEdit>,
    pub assets: &'a [String],
    pub plugin_names: &'a [String],
    pub command_count: usize,
    pub transform_edit: Option<TransformEdit>,
    pub viewport_focused: bool,
    pub create_entity: bool,
    pub delete_entity: Option<Entity>,
    pub duplicate_entity: Option<Entity>,
    pub save_requested: bool,
}

impl TabViewer for DockViewer<'_> {
    type Tab = EditorTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.title().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match *tab {
            EditorTab::Viewport => {
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
                        ui.label(
                            "Viewport tools are independent of panels through the plugin API.",
                        );
                    });
                });
            }
            EditorTab::Hierarchy => {
                ui.horizontal(|ui| {
                    ui.heading("Scene Hierarchy");
                    if ui
                        .small_button("+")
                        .on_hover_text("Create empty entity")
                        .clicked()
                    {
                        self.create_entity = true;
                    }
                    if self.selection.primary().is_some() {
                        if ui.small_button("Duplicate").clicked() {
                            self.duplicate_entity = self.selection.primary();
                        }
                        if ui.small_button("Delete").clicked() {
                            self.delete_entity = self.selection.primary();
                        }
                    }
                });
                ui.separator();
                for (entity, name) in self.entities {
                    let selected = self.selection.contains(*entity);
                    let response = ui.selectable_label(selected, name);
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
            EditorTab::Inspector => {
                ui.heading("Inspector");
                ui.separator();
                if let Some(mut edit) = self.selected_transform {
                    ui.label(format!("Primary Entity {:?}", edit.entity));
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
                        ui.label("Mesh / Material are visible through asset references.");
                    });
                } else {
                    ui.weak("Nothing selected");
                }
            }
            EditorTab::Assets => {
                ui.heading("Asset Browser");
                ui.separator();
                ui.label(format!("{} discovered files", self.assets.len()));
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for path in self.assets.iter().take(1000) {
                        let _ = ui.selectable_label(false, path);
                    }
                });
            }
            EditorTab::Console => {
                ui.horizontal(|ui| {
                    ui.heading("Console");
                    if ui.small_button("Save Scene").clicked() {
                        self.save_requested = true;
                    }
                });
                ui.separator();
                ui.monospace("[info] plugin-first editor kernel online");
                ui.monospace(format!("[info] {} commands registered", self.command_count));
                ui.monospace(format!(
                    "[info] {} plugins installed",
                    self.plugin_names.len()
                ));
                ui.monospace(format!("[info] {} selected", self.selection.entities.len()));
                if self.project.dirty {
                    ui.monospace("[warn] current scene has unsaved changes");
                }
                ui.monospace(format!("[info] status: {}", self.ui_state.status));
            }
            EditorTab::Plugins => {
                ui.heading("Plugins");
                ui.separator();
                for plugin in self.plugin_names {
                    ui.label(plugin);
                }
            }
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

pub fn show_dock_area(ui: &mut egui::Ui, dock: &mut EditorDockState, viewer: &mut DockViewer<'_>) {
    DockArea::new(&mut dock.state)
        .show_add_buttons(true)
        .show_add_popup(true)
        .show_close_buttons(true)
        .show_inside(ui, viewer);
}
