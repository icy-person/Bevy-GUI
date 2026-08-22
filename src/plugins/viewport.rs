use bevy::prelude::*;
use bevy_egui::egui;

use crate::{
    editor::{EditorPlugin, EditorPluginRegistry},
    panel::PanelRegistry,
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
            .register(self.name(), "0.1");
        app.world_mut().resource_mut::<PanelRegistry>().register(
            crate::panel::PanelId("viewport"),
            "Viewport",
            viewport_panel,
        );
    }
}

fn viewport_panel(_world: &mut World, ui: &mut egui::Ui) {
    ui.label("3D Viewport service");
    ui.small("Camera, picking, grid and transform tools are provided by the viewport subsystem.");
}
