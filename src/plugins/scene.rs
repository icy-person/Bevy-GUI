use bevy::prelude::*;
use bevy_egui::egui;

use crate::{
    editor::{EditorPlugin, EditorPluginRegistry},
    panel::PanelRegistry,
    project::ProjectState,
    scene_model::SceneEditorState,
};

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
            .register(self.name(), "1.0");
        app.world_mut().resource_mut::<PanelRegistry>().register(
            crate::panel::PanelId("scene"),
            "Scene",
            scene_panel,
        );
    }
}

fn scene_panel(world: &mut World, ui: &mut egui::Ui) {
    let project = world.get_resource::<ProjectState>();
    let scene_state = world.get_resource::<SceneEditorState>();
    ui.strong("Scene Authoring");
    if let Some(project) = project {
        ui.label(format!("Project: {}", project.name));
        ui.label(format!(
            "Main scene: {}",
            project
                .main_scene
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "<none>".into())
        ));
    }
    if let Some(state) = scene_state {
        ui.label(format!("Tracked scene nodes: {}", state.nodes.len()));
    }
}
