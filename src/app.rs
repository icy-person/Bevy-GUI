use bevy::prelude::*;
use bevy_egui::EguiPlugin;

use crate::{
    command::{EditorCommand, EditorCommandId, EditorCommandRegistry, EditorCommandBus},
    docking::EditorDockState,
    editor::register_builtin_state,
    plugins::install_builtin_editor_plugins,
    project::ProjectState,
    ui::install_editor_ui,
    viewport::install_viewport,
};

pub struct BevyGuiPlugin;

impl Plugin for BevyGuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .init_resource::<ProjectState>()
            .init_resource::<EditorCommandRegistry>()
            .init_resource::<EditorCommandBus>()
            .init_resource::<EditorDockState>();

        register_builtin_state(app);
        install_builtin_editor_plugins(app);
        install_viewport(app);
        install_editor_ui(app);
        app.add_systems(Startup, register_default_commands);
    }
}

fn register_default_commands(mut registry: ResMut<EditorCommandRegistry>) {
    for (id, label, shortcut) in [
        ("project.save", "Save Project", Some("Ctrl+S")),
        ("project.play", "Play", Some("F6")),
        ("project.pause", "Pause", Some("F7")),
        ("project.stop", "Stop", Some("F8")),
        ("edit.undo", "Undo", Some("Ctrl+Z")),
        ("edit.redo", "Redo", Some("Ctrl+Y")),
        ("scene.save", "Save Scene", Some("Ctrl+Shift+S")),
        ("scene.new_entity", "Create Entity", Some("Ctrl+Shift+A")),
        ("scene.duplicate", "Duplicate Entity", Some("Ctrl+D")),
        ("scene.delete", "Delete Entity", Some("Delete")),
    ] {
        registry.register(EditorCommand {
            id: EditorCommandId(id),
            label,
            shortcut,
        });
    }
}
