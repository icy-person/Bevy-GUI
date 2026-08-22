use bevy::prelude::*;
use bevy_egui::egui;

use crate::{EditorPlugin, EditorPluginRegistry, PanelRegistry};

pub struct ConsolePlugin;

impl Default for ConsolePlugin {
    fn default() -> Self {
        Self
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

fn console_panel(_world: &mut World, ui: &mut egui::Ui) {
    ui.label("Console service");
    ui.small("Logging, command output and diagnostics are provided by the console subsystem.");
}
