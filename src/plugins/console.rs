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
            .register(self.name(), "1.1");
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

    let executed = state.executed;
    let last = state.last.map(|id| id.0.to_owned());
    let message = state.last_message.clone();
    let error = state.last_error.clone();

    ui.horizontal(|ui| {
        ui.strong("Console");
        ui.weak(format!("{executed} commands"));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Clear").clicked() {
                if let Some(mut current) = world.get_resource_mut::<CommandExecutionState>() {
                    current.last = None;
                    current.last_message = None;
                    current.last_error = None;
                }
            }
        });
    });

    ui.separator();
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            if let Some(last) = last {
                log_line(ui, "CMD", egui::Color32::from_rgb(140, 190, 255), &last);
            }
            if let Some(message) = message {
                log_line(ui, "OK", egui::Color32::from_rgb(125, 220, 160), &message);
            }
            if let Some(error) = error {
                log_line(ui, "ERR", egui::Color32::from_rgb(255, 125, 125), &error);
            }
            if executed == 0 {
                ui.centered_and_justified(|ui| {
                    ui.label("No commands have been executed yet.");
                });
            }
        });
}

fn log_line(ui: &mut egui::Ui, prefix: &str, color: egui::Color32, message: &str) {
    ui.horizontal_wrapped(|ui| {
        ui.monospace(egui::RichText::new(prefix).color(color));
        ui.monospace(message);
    });
}
