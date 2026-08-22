//! Built-in editor capabilities. Every capability is a regular Bevy `Plugin` so
//! projects can replace, disable or extend it without changing the editor core.

use bevy::prelude::*;
use bevy_egui::egui;

use crate::{EditorPlugin, EditorPluginRegistry, PanelRegistry};

pub struct SceneEditorPlugin;
pub struct ViewportPlugin;
pub struct InspectorPlugin;
pub struct AssetBrowserPlugin;
pub struct ConsolePlugin;

impl Default for SceneEditorPlugin {
    fn default() -> Self {
        Self
    }
}

impl Default for ViewportPlugin {
    fn default() -> Self {
        Self
    }
}

impl Default for InspectorPlugin {
    fn default() -> Self {
        Self
    }
}

impl Default for AssetBrowserPlugin {
    fn default() -> Self {
        Self
    }
}

impl Default for ConsolePlugin {
    fn default() -> Self {
        Self
    }
}

impl EditorPlugin for SceneEditorPlugin {
    fn name(&self) -> &'static str {
        "scene-editor"
    }

    fn build(&self, app: &mut App) {
        app.add_plugins(SceneRuntimePlugin);
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

impl EditorPlugin for ViewportPlugin {
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

impl EditorPlugin for InspectorPlugin {
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

impl EditorPlugin for ConsolePlugin {
    fn name(&self) -> &'static str {
        "console"
    }

    fn build(&self, app: &mut App) {
        app.world_mut()
            .resource_mut::<EditorPluginRegistry>()
            .register(self.name(), "0.1");
        app.world_mut().resource_mut::<PanelRegistry>().register(
            crate::panel::PanelId("console"),
            "Console",
            console_panel,
        );
    }
}

struct SceneRuntimePlugin;

impl Plugin for SceneRuntimePlugin {
    fn build(&self, _app: &mut App) {}
}

fn scene_panel(_world: &mut World, ui: &mut egui::Ui) {
    ui.label("Scene graph service");
}

fn viewport_panel(_world: &mut World, ui: &mut egui::Ui) {
    ui.label("Viewport service");
}

fn inspector_panel(_world: &mut World, ui: &mut egui::Ui) {
    ui.label("Inspector service");
}

fn asset_panel(_world: &mut World, ui: &mut egui::Ui) {
    ui.label("Asset database service");
}

fn console_panel(_world: &mut World, ui: &mut egui::Ui) {
    ui.label("Console service");
}

pub fn install_builtin_editor_plugins(app: &mut App) {
    let plugins: [Box<dyn EditorPlugin>; 5] = [
        Box::new(SceneEditorPlugin),
        Box::new(ViewportPlugin),
        Box::new(InspectorPlugin),
        Box::new(AssetBrowserPlugin),
        Box::new(ConsolePlugin),
    ];

    for plugin in plugins {
        plugin.build(app);
    }
}
