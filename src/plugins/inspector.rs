use bevy::prelude::*;
use bevy_egui::egui;

use crate::{
    editor::{EditorPlugin, EditorPluginRegistry},
    panel::PanelRegistry,
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
            .register(self.name(), "1.0");
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
        ui.label("Select an entity to inspect it.");
        return;
    };

    ui.strong(format!("Entity {:?}", entity));
    ui.separator();

    let mut query = world.query_filtered::<
        (Option<&Name>, Option<&Transform>, Option<&Visibility>, Option<&crate::EditorParent>),
        With<EditorEntity>,
    >();
    if let Ok((name, transform, visibility, parent)) = query.get(world, entity) {
        ui.label(format!("Name: {}", name.map(Name::as_str).unwrap_or("<unnamed>")));
        if let Some(transform) = transform {
            ui.label(format!("Position: {:.2}, {:.2}, {:.2}", transform.translation.x, transform.translation.y, transform.translation.z));
            ui.label(format!("Scale: {:.2}, {:.2}, {:.2}", transform.scale.x, transform.scale.y, transform.scale.z));
        }
        if let Some(visibility) = visibility {
            ui.label(format!("Visibility: {:?}", visibility));
        }
        ui.label(format!("Parent: {:?}", parent.and_then(|value| value.0)));
    } else {
        ui.label("Selected entity is not part of the editor scene.");
    }
}
