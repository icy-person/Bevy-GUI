use bevy::prelude::*;
use bevy_egui::egui;

use crate::{
    command_executor::CommandExecutionState,
    editor::{EditorPlugin, EditorPluginRegistry},
    panel::PanelRegistry,
};

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
            .register(self.name(), "1.0");
        app.world_mut().resource_mut::<PanelRegistry>().register(
            crate::panel::PanelId("console"),
            "Console",
            console_panel,
        );
    }
}

fn console_panel(world: &mut World, ui: &mut egui::Ui) {
    let Some(state) = world.get_resource::<CommandExecutionState>() else {
        ui.label("Command execution state is not initialized.");
        return;
    };
    ui.strong("Command Console");
    ui.label(format!("Executed: {}", state.executed));
    if let Some(last) = &state.last {
        ui.monospace(format!("Last: {}", last.0));
    }
    if let Some(message) = &state.last_message {
        ui.colored_label(egui::Color32::from_rgb(130, 210, 155), message);
    }
    if let Some(error) = &state.last_error {
        ui.colored_label(egui::Color32::from_rgb(255, 125, 125), error);
    }
}
