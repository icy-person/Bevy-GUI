use bevy::prelude::*;
use bevy_egui::egui;

use crate::{EditorPlugin, EditorPluginRegistry, PanelRegistry};

pub struct SceneEditorPlugin;

impl Default for SceneEditorPlugin {
    fn default() -> Self {
        Self
    }
}

impl EditorPlugin for SceneEditorPlugin {
    fn name(&self) -> &'static str {
        "scene-editor"
    }

    fn build(&self, app: &mut App) {
        app.world_mut()
            .resource_mut::<EditorPluginRegistry>()
            .register(self.name(), "0.1");
        app.world_mut().resource_mut::<PanelRegistry>().register(
            crate::panel::PanelId("scene"),
            "Scene",
            scene_panel,
        );
    }
}

fn scene_panel(_world: &mut World, ui: &mut egui::Ui) {
    ui.label("Scene graph service");
    ui.small("Hierarchy, scene documents and entity authoring are provided by the scene subsystem.");
}
