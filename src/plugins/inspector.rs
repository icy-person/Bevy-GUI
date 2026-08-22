use bevy::prelude::*;
use bevy_egui::egui;

use crate::{EditorPlugin, EditorPluginRegistry, PanelRegistry};

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
            .register(self.name(), "0.1");
        app.world_mut().resource_mut::<PanelRegistry>().register(
            crate::panel::PanelId("inspector"),
            "Inspector",
            inspector_panel,
        );
    }
}

fn inspector_panel(_world: &mut World, ui: &mut egui::Ui) {
    ui.label("Inspector service");
    ui.small("Reflection-backed component editing belongs to the inspector subsystem.");
}
