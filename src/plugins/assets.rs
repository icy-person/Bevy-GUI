use bevy::prelude::*;
use bevy_egui::egui;

use crate::{
    assets::AssetDatabase,
    editor::{EditorPlugin, EditorPluginRegistry},
    panel::PanelRegistry,
};

pub struct AssetBrowserPlugin;

impl Default for AssetBrowserPlugin {
    fn default() -> Self {
        Self
    }
}

impl EditorPlugin for AssetBrowserPlugin {
    fn name(&self) -> &'static str {
        "asset-browser"
    }

    fn build(&self, app: &mut App) {
        app.world_mut()
            .resource_mut::<EditorPluginRegistry>()
            .register(self.name(), "1.0");
        app.world_mut().resource_mut::<PanelRegistry>().register(
            crate::panel::PanelId("assets"),
            "Assets",
            asset_panel,
        );
    }
}

fn asset_panel(world: &mut World, ui: &mut egui::Ui) {
    let Some(database) = world.get_resource::<AssetDatabase>() else {
        ui.label("Asset database is not initialized.");
        return;
    };
    ui.horizontal(|ui| {
        ui.strong("Asset Database");
        ui.label(format!("generation {}", database.generation));
    });
    ui.label(format!("{} indexed files", database.entries.len()));
    ui.small(format!("root: {}", database.root.display()));
    if let Some(selected) = &database.selected {
        ui.separator();
        ui.monospace(selected.display().to_string());
    }
}
