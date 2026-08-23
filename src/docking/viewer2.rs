use bevy::prelude::*;
use bevy_egui::egui;
use egui_dock::{DockArea, TabViewer};

use crate::{
    assets::{AssetDatabase, AssetKind},
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
        !matches!(tab, EditorTab::Viewport)
    }

    fn scroll_bars(&self, tab: &Self::Tab) -> [bool; 2] {
        if matches!(tab, EditorTab::Viewport) { [false, false] } else { [true, true] }
    }

    fn is_closeable(&self, tab: &Self::Tab) -> bool {
        !matches!(tab, EditorTab::Viewport)
    }
}

impl DockViewer<'_> {
    fn panel(ui: &mut egui::Ui, body: impl FnOnce(&mut egui::Ui)) {
        egui::Frame::new()
            .fill(ui.visuals().panel_fill)
            .inner_margin(egui::Margin::same(12))
            .show(ui, body);
    }

    fn surface(ui: &mut egui::Ui, body: impl FnOnce(&mut egui::Ui)) {
        egui::Frame::new()
            .fill(ui.visuals().widgets.noninteractive.bg_fill)
            .stroke(egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.fg_stroke.color.gamma_multiply(0.28)))
            .corner_radius(egui::CornerRadius::same(12))
            .inner_margin(egui::Margin::same(12))
            .show(ui, body);
    }

    fn heading(ui: &mut egui::Ui, title: &str, subtitle: &str) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(title).size(16.0).strong());
            ui.label(egui::RichText::new(subtitle).weak().size(11.0));
        });
    }

    fn tool(ui: &mut egui::Ui, label: &str, active: bool) -> egui::Response {
        ui.add_sized(
            [48.0, 32.0],
            egui::Button::new(label)
                .selected(active)
                .corner_radius(egui::CornerRadius::same(8)),
        )
    }

    fn show_viewport(&mut self, ui: &mut egui::Ui) {
        self.viewport_focused = true;
        let available = ui.available_size();
        Self::panel(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new("Scene").strong());
                if Self::tool(ui, "2D", self.ui_state.viewport_mode == ViewportMode::TwoD).clicked() {
                    self.ui_state.viewport_mode = ViewportMode::TwoD;
                }
                if Self::tool(ui, "3D", self.ui_state.viewport_mode == ViewportMode::ThreeD).clicked() {
                    self.ui_state.viewport_mode = ViewportMode::ThreeD;
                }
                ui.separator();
                if Self::tool(ui, "W", false).on_hover_text("Move").clicked() {}
                if Self::tool(ui, "E", false).on_hover_text("Rotate").clicked() {}
                if Self::tool(ui, "R", false).on_hover_text("Scale").clicked() {}
                ui.separator();
                ui.label(if self.settings.settings.viewport.snap_enabled { "Snap" } else { "Free" });
                ui.label(if self.project.dirty { "Unsaved" } else { "Saved" });
            });
            ui.separator();
            ui.horizontal_wrapped(|ui| {
                let mode = match self.ui_state.viewport_mode { ViewportMode::TwoD => "2D Orthographic", ViewportMode::ThreeD => "3D Perspective" };
                ui.label(mode);
                ui.separator();
                ui.label(format!("{} entities", self.entities.len()));
                ui.separator();
                ui.label(format!("{} selected", self.selection.entities.len()));
                ui.separator();
                ui.label(format!("{:.1} FPS", self.profiler.fps));
            });
            ui.separator();
            ui.allocate_space(available - egui::vec2(0.0, 90.0).min(available));
        });
    }

    fn show_hierarchy(&mut self, ui: &mut egui::Ui) {
        Self::panel(ui, |ui| {
            Self::heading(ui, "Scene Hierarchy", "Entities");
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                if ui.button("＋ Entity").clicked() { self.create_entity = true; }
                if ui.button("Duplicate").clicked() { self.duplicate_entity = self.selection.primary(); }
                if ui.button("Delete").clicked() { self.delete_entity = self.selection.primary(); }
                if self.selection.entities.len() >= 2 && ui.button("Parent").clicked() { self.parent_selected = true; }
                if self.selection.entities.len() >= 2 && ui.button("Unparent").clicked() { self.unparent_selected = true; }
            });
            ui.separator();
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                if self.entities.is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.label(egui::RichText::new("Empty Scene").size(18.0).strong());
                    });
                    return;
                }
                for (entity, name) in self.entities {
                    let selected = self.selection.contains(*entity);
                    let depth = self.entity_depth(*entity);
                    ui.horizontal(|ui| {
                        ui.add_space((depth as f32 * 14.0).min(140.0));
                        ui.label(if depth == 0 { "◆" } else { "◇" });
                        let response = ui.selectable_label(selected, name);
                        if response.clicked() {
                            if ui.input(|input| input.modifiers.shift) { self.selection.toggle(*entity); } else { self.selection.select(*entity); }
                        }
                    });
                }
            });
        });
    }

    fn entity_depth(&self, entity: Entity) -> usize {
        let mut depth = 0usize;
        let mut current = entity;
        for _ in 0..64 {
            let parent = self.parents.iter().find(|(candidate, _)| *candidate == current).and_then(|(_, parent)| *parent);
            let Some(parent) = parent else { break; };
            current = parent;
            depth += 1;
        }
        depth.min(12)
    }

    fn show_inspector(&mut self, ui: &mut egui::Ui) {
        Self::panel(ui, |ui| {
            Self::heading(ui, "Inspector", "Entity properties");
            ui.separator();
            let Some(mut edit) = self.selected_transform else {
                ui.centered_and_justified(|ui| ui.label("Select an entity to inspect it."));
                return;
            };
            Self::surface(ui, |ui| {
                ui.horizontal(|ui| { ui.label("Entity"); ui.monospace(format!("{:?}", edit.entity)); });
                if let Some(name) = self.selected_name.as_mut() {
                    ui.horizontal(|ui| { ui.label("Name"); if ui.text_edit_singleline(name).lost_focus() { self.name_edit = Some(name.clone()); } });
                }
            });
            ui.add_space(8.0);
            Self::surface(ui, |ui| {
                let mut changed = false;
                ui.label(egui::RichText::new("Transform").strong());
                changed |= drag_vec3(ui, "Position", &mut edit.translation, 0.05);
                changed |= drag_vec3(ui, "Rotation", &mut edit.rotation, 0.5);
                changed |= drag_vec3(ui, "Scale", &mut edit.scale, 0.02);
                if changed { self.project.dirty = true; self.transform_edit = Some(edit); }
                ui.horizontal(|ui| {
                    if ui.button("Apply").clicked() { self.transform_edit = Some(edit); }
                    if ui.button("Reset").clicked() { self.transform_edit = Some(TransformEdit { entity: edit.entity, translation: Vec3::ZERO, rotation: Vec3::ZERO, scale: Vec3::ONE }); }
                });
            });
            ui.add_space(8.0);
            Self::surface(ui, |ui| {
                let mut visible = self.selected_visible.unwrap_or(true);
                if ui.checkbox(&mut visible, "Visible").changed() { self.visibility_edit = Some(visible); }
                ui.separator();
                ui.label(egui::RichText::new("Components").strong());
                for label in ["EditorEntity", "Name", "Transform", "Visibility", "EditorParent"] { ui.label(format!("• {label}")); }
            });
        });
    }

    fn show_assets(&mut self, ui: &mut egui::Ui) {
        Self::panel(ui, |ui| {
            Self::heading(ui, "Asset Browser", "Project library");
            ui.horizontal(|ui| {
                if ui.button("Refresh").clicked() { self.assets.refresh_requested = true; }
                ui.label(format!("{} assets", self.assets.entries.len()));
            });
            if ui.add(egui::TextEdit::singleline(&mut self.assets.search).hint_text("Search assets…")).changed() { ui.ctx().request_repaint(); }
            ui.separator();
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                let entries = self.assets.filtered().take(1000).map(|e| (e.path.clone(), e.kind, e.bytes)).collect::<Vec<_>>();
                if entries.is_empty() { ui.centered_and_justified(|ui| ui.label("No matching assets.")); return; }
                for (path, kind, bytes) in entries {
                    let selected = self.assets.selected.as_ref() == Some(&path);
                    let text = format!("{}  {}  · {}", asset_icon(kind), path.display(), format_size(bytes));
                    if ui.selectable_label(selected, text).clicked() { self.assets.selected = Some(path); }
                }
            });
        });
    }

    fn show_console(&mut self, ui: &mut egui::Ui) {
        Self::panel(ui, |ui| {
            Self::heading(ui, "Console", "Editor diagnostics");
            ui.separator();
            ui.monospace(format!("Project: {}", self.project.name));
            ui.monospace(format!("Mode: {:?}", self.project.mode));
            ui.monospace(format!("Commands: {}", self.command_count));
            ui.monospace(format!("Plugins: {}", self.plugin_names.len()));
            ui.monospace(format!("Selection: {}", self.selection.entities.len()));
            ui.monospace(format!("FPS: {:.1}", self.profiler.fps));
            ui.monospace(format!("Frame: {:.2} ms", self.profiler.frame_time_ms));
            ui.monospace(format!("Average: {:.2} ms", self.profiler.average_frame_ms));
            ui.monospace(format!("1% low: {:.1} FPS", self.profiler.one_percent_low_fps));
            if self.project.dirty { ui.colored_label(egui::Color32::from_rgb(255, 190, 96), "Unsaved changes"); }
            if ui.button("Save Scene").clicked() { self.save_requested = true; }
        });
    }

    fn show_profiler(&mut self, ui: &mut egui::Ui) {
        Self::panel(ui, |ui| {
            Self::heading(ui, "Profiler", "Frame timing");
            ui.separator();
            ui.horizontal_wrapped(|ui| {
                metric(ui, "FPS", format!("{:.1}", self.profiler.fps));
                metric(ui, "Frame", format!("{:.2} ms", self.profiler.frame_time_ms));
                metric(ui, "1% Low", format!("{:.1}", self.profiler.one_percent_low_fps));
            });
            ui.add_space(8.0);
            let budget = self.profiler.frame_budget_ms(60.0);
            ui.add(egui::ProgressBar::new((self.profiler.frame_time_ms / budget).clamp(0.0, 1.0)).text(format!("60 FPS budget · {:.2} / {:.2} ms", self.profiler.frame_time_ms, budget)));
            ui.add_space(8.0);
            ui.label(format!("Min {:.2} ms · Avg {:.2} ms · Max {:.2} ms", self.profiler.min_frame_ms, self.profiler.average_frame_ms, self.profiler.max_frame_ms));
            ui.label(format!("Dropped frames: {} · Samples: {}", self.profiler.dropped_frames, self.profiler.samples));
        });
    }

    fn show_plugins(&mut self, ui: &mut egui::Ui) {
        Self::panel(ui, |ui| {
            Self::heading(ui, "Plugins", "Editor extensions");
            ui.separator();
            for plugin in self.plugin_names {
                Self::surface(ui, |ui| {
                    ui.horizontal(|ui| { ui.label("✦"); ui.label(plugin); });
                });
                ui.add_space(6.0);
            }
        });
    }

    fn show_settings(&mut self, ui: &mut egui::Ui) {
        Self::panel(ui, |ui| { crate::ui::settings::show_settings(ui, self.settings, self.project); });
    }
}

fn drag_vec3(ui: &mut egui::Ui, label: &str, value: &mut Vec3, speed: f32) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(label);
        changed |= ui.add(egui::DragValue::new(&mut value.x).prefix("X ").speed(speed)).changed();
        changed |= ui.add(egui::DragValue::new(&mut value.y).prefix("Y ").speed(speed)).changed();
        changed |= ui.add(egui::DragValue::new(&mut value.z).prefix("Z ").speed(speed)).changed();
    });
    changed
}

fn metric(ui: &mut egui::Ui, label: &str, value: String) {
    egui::Frame::new().fill(ui.visuals().faint_bg_color).corner_radius(egui::CornerRadius::same(10)).inner_margin(egui::Margin::same(10)).show(ui, |ui| {
        ui.label(egui::RichText::new(value).size(18.0).strong());
        ui.label(egui::RichText::new(label).weak().size(11.0));
    });
}

fn asset_icon(kind: AssetKind) -> &'static str {
    match kind { AssetKind::Scene => "◫", AssetKind::Texture => "▧", AssetKind::Mesh => "◇", AssetKind::Material => "●", AssetKind::Audio => "♫", AssetKind::Script => "⌘", AssetKind::Data => "▤", AssetKind::Other => "•" }
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 { format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0)) }
    else if bytes >= 1024 * 1024 { format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0)) }
    else if bytes >= 1024 { format!("{:.1} KB", bytes as f64 / 1024.0) }
    else { format!("{} B", bytes) }
}

pub fn show_dock_area(ui: &mut egui::Ui, dock: &mut EditorDockState, viewer: &mut DockViewer<'_>) {
    DockArea::new(&mut dock.state)
        .show_add_buttons(true)
        .show_add_popup(true)
        .show_close_buttons(true)
        .show_inside(ui, viewer);
}
