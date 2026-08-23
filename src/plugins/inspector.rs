use bevy::prelude::*;
use bevy_egui::egui;

use crate::{
    editor::{EditorPlugin, EditorPluginRegistry},
    panel::PanelRegistry,
    scene_model::EditorParent,
    selection::SelectionState,
    viewport::EditorEntity,
};

pub struct InspectorEditorPlugin;

impl Default for InspectorEditorPlugin {
    fn default() -> Self {
        Self
    }
}

impl EditorPlugin for InspectorEditorPlugin {
    fn name(&self) -> &'static str {
        "inspector"
    }

    fn build(&self, app: &mut App) {
        app.world_mut()
            .resource_mut::<EditorPluginRegistry>()
            .register(self.name(), "1.1");
        app.world_mut().resource_mut::<PanelRegistry>().register(
            crate::panel::PanelId("inspector"),
            "Inspector",
            inspector_panel,
        );
    }
}

fn inspector_panel(world: &mut World, ui: &mut egui::Ui) {
    let entity = world
        .get_resource::<SelectionState>()
        .and_then(|selection| selection.primary());
    let Some(entity) = entity else {
        empty_inspector(ui);
        return;
    };

    let snapshot = {
        let mut query = world.query_filtered::<
            (
                Option<&Name>,
                Option<&Transform>,
                Option<&Visibility>,
                Option<&EditorParent>,
            ),
            With<EditorEntity>,
        >();
        let Ok((name, transform, visibility, parent)) = query.get(world, entity) else {
            ui.colored_label(egui::Color32::from_rgb(255, 130, 130), "Selected entity is no longer alive.");
            return;
        };
        (
            name.map(Name::as_str).unwrap_or("Entity").to_owned(),
            transform.copied(),
            visibility
                .copied()
                .map(|value| !matches!(value, Visibility::Hidden))
                .unwrap_or(true),
            parent.and_then(|value| value.0),
            transform.is_some(),
            name.is_some(),
            visibility.is_some(),
            parent.is_some(),
        )
    };

    let (mut name_text, original_transform, mut visible, parent_entity, has_transform, has_name, has_visibility, has_parent) = snapshot;
    let mut translation = original_transform.map(|value| value.translation).unwrap_or(Vec3::ZERO);
    let mut rotation = original_transform
        .map(|value| {
            let (x, y, z) = value.rotation.to_euler(EulerRot::XYZ);
            Vec3::new(x.to_degrees(), y.to_degrees(), z.to_degrees())
        })
        .unwrap_or(Vec3::ZERO);
    let mut scale = original_transform.map(|value| value.scale).unwrap_or(Vec3::ONE);

    ui.horizontal(|ui| {
        ui.strong("Inspector");
        ui.weak(format!("Entity {:?}", entity));
    });
    ui.separator();

    egui::CollapsingHeader::new("Identity")
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Name");
                if ui.text_edit_singleline(&mut name_text).changed() {
                    world.entity_mut(entity).insert(Name::new(name_text.clone()));
                    mark_dirty(world);
                }
            });
            ui.horizontal(|ui| {
                ui.label("Parent");
                ui.monospace(
                    parent_entity
                        .map(|value| format!("{:?}", value))
                        .unwrap_or_else(|| "<root>".into()),
                );
            });
        });

    egui::CollapsingHeader::new("Visibility")
        .default_open(true)
        .show(ui, |ui| {
            if ui.checkbox(&mut visible, "Visible in scene").changed() {
                let state = if visible { Visibility::Inherited } else { Visibility::Hidden };
                world.entity_mut(entity).insert(state);
                mark_dirty(world);
            }
        });

    if has_transform {
        let mut changed = false;
        egui::CollapsingHeader::new("Transform")
            .default_open(true)
            .show(ui, |ui| {
                ui.label("Position");
                changed |= ui.add(egui::DragValue::new(&mut translation.x).prefix("X ").speed(0.05)).changed();
                changed |= ui.add(egui::DragValue::new(&mut translation.y).prefix("Y ").speed(0.05)).changed();
                changed |= ui.add(egui::DragValue::new(&mut translation.z).prefix("Z ").speed(0.05)).changed();
                ui.label("Rotation (degrees)");
                changed |= ui.add(egui::DragValue::new(&mut rotation.x).prefix("X ").speed(0.5)).changed();
                changed |= ui.add(egui::DragValue::new(&mut rotation.y).prefix("Y ").speed(0.5)).changed();
                changed |= ui.add(egui::DragValue::new(&mut rotation.z).prefix("Z ").speed(0.5)).changed();
                ui.label("Scale");
                changed |= ui.add(egui::DragValue::new(&mut scale.x).prefix("X ").speed(0.02)).changed();
                changed |= ui.add(egui::DragValue::new(&mut scale.y).prefix("Y ").speed(0.02)).changed();
                changed |= ui.add(egui::DragValue::new(&mut scale.z).prefix("Z ").speed(0.02)).changed();
                ui.horizontal(|ui| {
                    if ui.button("Reset Position").clicked() {
                        translation = Vec3::ZERO;
                        changed = true;
                    }
                    if ui.button("Reset Rotation").clicked() {
                        rotation = Vec3::ZERO;
                        changed = true;
                    }
                    if ui.button("Reset Scale").clicked() {
                        scale = Vec3::ONE;
                        changed = true;
                    }
                });
            });

        if changed {
            if let Some(mut transform) = world.get_mut::<Transform>(entity) {
                transform.translation = translation;
                transform.rotation = Quat::from_euler(
                    EulerRot::XYZ,
                    rotation.x.to_radians(),
                    rotation.y.to_radians(),
                    rotation.z.to_radians(),
                );
                transform.scale = scale;
            }
            mark_dirty(world);
        }
    } else if ui.button("Add Transform").clicked() {
        world.entity_mut(entity).insert(Transform::default());
        mark_dirty(world);
    }

    egui::CollapsingHeader::new("Components")
        .default_open(true)
        .show(ui, |ui| {
            component_status(ui, "EditorEntity", true);
            component_status(ui, "Name", has_name);
            component_status(ui, "Transform", has_transform);
            component_status(ui, "Visibility", has_visibility);
            component_status(ui, "EditorParent", has_parent);
        });
}

fn component_status(ui: &mut egui::Ui, label: &str, present: bool) {
    ui.horizontal(|ui| {
        ui.label(if present { "✓" } else { "○" });
        ui.label(label);
    });
}

fn empty_inspector(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(48.0);
        ui.label(egui::RichText::new("Inspector").size(18.0).strong());
        ui.label("Select an entity from the Hierarchy or Viewport.");
    });
}

fn mark_dirty(world: &mut World) {
    if let Some(mut project) = world.get_resource_mut::<crate::ProjectState>() {
        project.dirty = true;
    }
    if let Some(mut scene) = world.get_resource_mut::<crate::scene_model::SceneEditorState>() {
        scene.mark_dirty();
    }
}
