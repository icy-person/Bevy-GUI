use bevy::prelude::*;
use bevy_egui::egui;

use crate::{EditorPlugin, EditorPluginRegistry, PanelRegistry};

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
            .register(self.name(), "0.1");
        app.world_mut().resource_mut::<PanelRegistry>().register(
            crate::panel::PanelId("assets"),
            "Assets",
            asset_panel,
        );
    }
}

fn asset_panel(_world: &mut World, ui: &mut egui::Ui) {
    ui.label("Asset database service");
    ui.small("Import, indexing, previews and drag/drop are provided by the asset subsystem.");
}
