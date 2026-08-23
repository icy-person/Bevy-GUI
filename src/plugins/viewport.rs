use bevy::prelude::*;
use bevy_egui::egui;

use crate::{
    editor::{EditorPlugin, EditorPluginRegistry, EditorUiState, ViewportMode},
    panel::PanelRegistry,
    settings::EditorSettingsState,
    viewport::{PlacementAxis, PlacementSettings, ViewportCursor},
};

pub struct ViewportEditorPlugin;

impl Default for ViewportEditorPlugin {
    fn default() -> Self {
        Self
    }
}

impl EditorPlugin for ViewportEditorPlugin {
    fn name(&self) -> &'static str {
        "viewport"
    }

    fn build(&self, app: &mut App) {
        app.world_mut()
            .resource_mut::<EditorPluginRegistry>()
            .register(self.name(), "1.1");
        app.world_mut().resource_mut::<PanelRegistry>().register(
            crate::panel::PanelId("viewport"),
            "Viewport",
            viewport_panel,
        );
    }
}

fn viewport_panel(world: &mut World, ui: &mut egui::Ui) {
    let mode = world
        .get_resource::<EditorUiState>()
        .map(|state| state.viewport_mode)
        .unwrap_or(ViewportMode::ThreeD);
    let cursor = world.get_resource::<ViewportCursor>().cloned().unwrap_or_default();
    let settings = world
        .get_resource::<EditorSettingsState>()
        .map(|state| state.settings.viewport.clone());

    ui.horizontal(|ui| {
        ui.strong("Viewport");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(match mode {
                ViewportMode::TwoD => "2D",
                ViewportMode::ThreeD => "3D",
            });
        });
    });
    ui.separator();

    ui.horizontal(|ui| {
        if ui.selectable_label(mode == ViewportMode::TwoD, "2D").clicked() {
            if let Some(mut state) = world.get_resource_mut::<EditorUiState>() {
                state.viewport_mode = ViewportMode::TwoD;
            }
        }
        if ui.selectable_label(mode == ViewportMode::ThreeD, "3D").clicked() {
            if let Some(mut state) = world.get_resource_mut::<EditorUiState>() {
                state.viewport_mode = ViewportMode::ThreeD;
            }
        }
    });

    egui::CollapsingHeader::new("Placement")
        .default_open(true)
        .show(ui, |ui| {
            let mut placement = world
                .get_resource::<PlacementSettings>()
                .cloned()
                .unwrap_or_default();
            let mut changed = false;
            changed |= ui.checkbox(&mut placement.enabled, "Enabled").changed();
            changed |= ui.checkbox(&mut placement.snap_to_grid, "Snap to grid").changed();
            changed |= ui
                .add(
                    egui::DragValue::new(&mut placement.grid_size)
                        .prefix("Grid ")
                        .speed(0.05)
                        .range(0.001..=1000.0),
                )
                .changed();
            egui::ComboBox::from_label("Axis")
                .selected_text(match placement.axis {
                    PlacementAxis::X => "X",
                    PlacementAxis::Y => "Y",
                    PlacementAxis::Z => "Z",
                })
                .show_ui(ui, |ui| {
                    changed |= ui.selectable_value(&mut placement.axis, PlacementAxis::X, "X").changed();
                    changed |= ui.selectable_value(&mut placement.axis, PlacementAxis::Y, "Y").changed();
                    changed |= ui.selectable_value(&mut placement.axis, PlacementAxis::Z, "Z").changed();
                });
            if changed {
                if let Some(mut resource) = world.get_resource_mut::<PlacementSettings>() {
                    *resource = placement;
                }
            }
        });

    egui::CollapsingHeader::new("Camera")
        .default_open(false)
        .show(ui, |ui| {
            if let Some(settings) = settings {
                ui.label(format!("Move speed: {:.2}", settings.camera_move_speed));
                ui.label(format!("Orbit speed: {:.2}", settings.camera_orbit_speed));
                ui.label(format!("Zoom speed: {:.2}", settings.camera_zoom_speed));
            }
            ui.small("W/A/S/D/Q/E fly camera; hold right mouse to look.");
            ui.small("F-frame and axis shortcuts are handled by the viewport input system.");
        });

    egui::CollapsingHeader::new("Cursor")
        .default_open(true)
        .show(ui, |ui| {
            if let Some(position) = cursor.grid_position {
                ui.monospace(format!("Snapped  X {:.2}  Y {:.2}  Z {:.2}", position.x, position.y, position.z));
            } else {
                ui.label("Move the cursor over the 3D viewport to calculate a world position.");
            }
            if let Some(origin) = cursor.ray_origin {
                ui.small(format!("Ray origin: {:.2}, {:.2}, {:.2}", origin.x, origin.y, origin.z));
            }
            if let Some(direction) = cursor.ray_direction {
                ui.small(format!("Ray direction: {:.3}, {:.3}, {:.3}", direction.x, direction.y, direction.z));
            }
        });
}
