use bevy::prelude::*;
use bevy_egui::egui;
use crate::{editor::{EditorPlugin, EditorPluginRegistry}, panel::PanelRegistry, scene::{EditorPrimitive, EditorVisual, ScenePrimitive}, scene_model::EditorParent, selection::SelectionState, viewport::EditorEntity};

pub struct InspectorEditorPlugin;
impl Default for InspectorEditorPlugin { fn default() -> Self { Self } }
impl EditorPlugin for InspectorEditorPlugin {
    fn name(&self) -> &'static str { "inspector" }
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<EditorPluginRegistry>().register(self.name(), "2.3");
        app.world_mut().resource_mut::<PanelRegistry>().register(crate::panel::PanelId("inspector"), "Inspector", inspector_panel);
    }
}

fn inspector_panel(world: &mut World, ui: &mut egui::Ui) {
    let Some(entity) = world.get_resource::<SelectionState>().and_then(SelectionState::primary) else { empty_inspector(ui); return; };
    if world.get::<EditorEntity>(entity).is_none() { ui.colored_label(egui::Color32::RED, "Selected entity is not an editor entity."); return; }

    let snapshot = {
        let mut query = world.query_filtered::<(Option<&Name>, Option<&Transform>, Option<&Visibility>, Option<&EditorParent>, Option<&EditorPrimitive>, Option<&EditorVisual>), With<EditorEntity>>();
        let Ok((name, transform, visibility, parent, primitive, visual)) = query.get(world, entity) else { empty_inspector(ui); return; };
        (name.map(Name::as_str).unwrap_or("Entity").to_owned(), transform.copied(), visibility.copied(), parent.map(|p| p.0), primitive.copied(), visual.map(|v| v.0.clone()))
    };

    ui.horizontal(|ui| { ui.strong("Inspector"); ui.weak(format!("Entity {:?}", entity)); });
    ui.separator();
    ui.label(format!("Name: {}", snapshot.0));
    ui.label(format!("Parent: {:?}", snapshot.3.flatten()));

    if let Some(transform) = snapshot.1 {
        egui::CollapsingHeader::new("Transform").default_open(true).show(ui, |ui| {
            ui.label(format!("Position  X {:.3}  Y {:.3}  Z {:.3}", transform.translation.x, transform.translation.y, transform.translation.z));
            let (x,y,z)=transform.rotation.to_euler(EulerRot::XYZ);
            ui.label(format!("Rotation  X {:.1}°  Y {:.1}°  Z {:.1}°", x.to_degrees(), y.to_degrees(), z.to_degrees()));
            ui.label(format!("Scale     X {:.3}  Y {:.3}  Z {:.3}", transform.scale.x, transform.scale.y, transform.scale.z));
        });
    }

    egui::CollapsingHeader::new("Visibility").default_open(true).show(ui, |ui| {
        ui.label(if matches!(snapshot.2, Some(Visibility::Hidden)) { "Hidden" } else { "Visible" });
    });

    egui::CollapsingHeader::new("Rendering").default_open(true).show(ui, |ui| {
        ui.label(format!("Primitive: {:?}", snapshot.4.map(|p| p.0).unwrap_or(ScenePrimitive::None)));
        if let Some(visual) = &snapshot.5 {
            ui.label(format!("Metallic: {:.2}", visual.metallic));
            ui.label(format!("Roughness: {:.2}", visual.roughness));
            ui.label(format!("Collision: {}", visual.collision));
            ui.label(format!("Audio: {}", visual.audio.asset.as_deref().unwrap_or("<none>")));
        }
    });

    egui::CollapsingHeader::new("Components").default_open(true).show(ui, |ui| {
        for (label, present) in [
            ("EditorEntity", true),
            ("Name", world.get::<Name>(entity).is_some()),
            ("Transform", world.get::<Transform>(entity).is_some()),
            ("Visibility", world.get::<Visibility>(entity).is_some()),
            ("EditorParent", world.get::<EditorParent>(entity).is_some()),
            ("EditorPrimitive", world.get::<EditorPrimitive>(entity).is_some()),
            ("EditorVisual", world.get::<EditorVisual>(entity).is_some()),
        ] { component_status(ui, label, present); }
    });
}

fn component_status(ui: &mut egui::Ui, label: &str, present: bool) { ui.horizontal(|ui| { ui.label(if present { "✓" } else { "○" }); ui.label(label); }); }
fn empty_inspector(ui: &mut egui::Ui) { ui.vertical_centered(|ui| { ui.add_space(48.0); ui.label(egui::RichText::new("Inspector").size(18.0).strong()); ui.label("Select an entity from the Hierarchy or Viewport."); }); }
