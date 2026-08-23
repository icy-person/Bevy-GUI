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
    let Some(selection) = world.get_resource::<SelectionState>() else {
        ui.label("Selection state is not initialized.");
        return;
    };
    let Some(entity) = selection.primary() else {
        empty_inspector(ui);
        return;
    };

    ui.horizontal(|ui| {
        ui.strong("Inspector");
        ui.weak(format!("Entity {:?}", entity));
    });
    ui.separator();

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

    let mut name_text = name.map(Name::as_str).unwrap_or("Entity").to_owned();
    let mut visible = !matches!(visibility, Some(Visibility::Hidden));
    let mut translation = transform.map(|value| value.translation).unwrap_or(Vec3::ZERO);
    let mut rotation = transform
        .map(|value| {
            let (x, y, z) = value.rotation.to_euler(EulerRot::XYZ);
            Vec3::new(x.to_degrees(), y.to_degrees(), z.to_degrees())
        })
        .unwrap_or(Vec3::ZERO);
    let mut scale = transform.map(|value| value.scale).unwrap_or(Vec3::ONE);
    let parent_entity = parent.and_then(|value| value.0);

    egui::CollapsingHeader::new("Identity")
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Name");
                if ui.text_edit_singleline(&mut name_text).changed() {
                    if let Some(mut name) = world.get_mut::<Name>(entity) {
                        *name = Name::new(name_text.clone());
                    } else {
                        world.entity_mut(entity).insert(Name::new(name_text.clone()));
                    }
                    mark_dirty(world);
                }
            });
            if ui.checkbox(&mut visible, "Visible").changed() {
                let value = if visible { Visibility::Inherited } else { Visibility::Hidden };
                world.entity_mut(entity).insert(value);
                mark_dirty(world);
            }
            ui.small(format!("Parent: {}", parent_entity.map(|value| format!("{:?}", value)).unwrap_or_else(|| "<root>".into())));
        });

    if let Some(mut transform) = world.get_mut::<Transform>(entity) {
        let original = *transform;
        egui::CollapsingHeader::new("Transform")
            .default_open(true)
            .show(ui, |ui| {
                ui.label("Position");
                let mut changed = false;
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

                if changed {
                    transform.translation = translation;
                    transform.rotation = Quat::from_euler(
                        EulerRot::XYZ,
                        rotation.x.to_radians(),
                        rotation.y.to_radians(),
                        rotation.z.to_radians(),
                    );
                    transform.scale = scale;
                }
            });
        if *transform != original {
            drop(transform);
            mark_dirty(world);
        }
    } else {
        ui.horizontal(|ui| {
            ui.label("Transform");
            if ui.button("Add Transform").clicked() {
                world.entity_mut(entity).insert(Transform::default());
                mark_dirty(world);
            }
        });
    }

    egui::CollapsingHeader::new("Components")
        .default_open(true)
        .show(ui, |ui| {
            component_status(ui, "EditorEntity", true);
            component_status(ui, "Name", name.is_some());
            component_status(ui, "Transform", transform.is_some());
            component_status(ui, "Visibility", visibility.is_some());
            component_status(ui, "EditorParent", parent.is_some());
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
