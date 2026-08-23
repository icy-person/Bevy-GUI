use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use crate::{asset_pipeline::ImportDatabase,assets::install_asset_database,command::{EditorCommand,EditorCommandBus,EditorCommandId,EditorCommandRegistry},command_executor::{execute_editor_commands,CommandExecutionState},component_registry::install_component_registry,docking::EditorDockState,editor::register_builtin_state,panel::PanelRegistry,plugins::install_builtin_editor_plugins,profiler::install_profiler,project::ProjectState,scene_model::SceneEditorState,scene_tools::SceneSelectionSet,settings::install_settings,ui::{command_palette::install_command_palette,install_editor_ui},viewport::install_viewport,viewport2d::install_2d_viewport};

pub struct BevyGuiPlugin;

impl Plugin for BevyGuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .init_resource::<ProjectState>()
            .init_resource::<EditorCommandRegistry>()
            .init_resource::<EditorCommandBus>()
            .init_resource::<CommandExecutionState>()
            .init_resource::<EditorDockState>()
            .init_resource::<SceneEditorState>()
            .init_resource::<PanelRegistry>()
            .init_resource::<ImportDatabase>()
            .init_resource::<SceneSelectionSet>();
        register_builtin_state(app);
        install_component_registry(app);
        install_settings(app);
        install_asset_database(app);
        install_profiler(app);
        install_builtin_editor_plugins(app);
        install_viewport(app);
        install_2d_viewport(app);
        install_editor_ui(app);
        install_command_palette(app);
        app.add_systems(PostStartup, load_import_database)
            .add_systems(Startup, register_default_commands)
            .add_systems(Update, execute_editor_commands);
    }
}

fn load_import_database(project: Res<ProjectState>, mut database: ResMut<ImportDatabase>) {
    match ImportDatabase::load(project.root.clone()) {
        Ok(loaded) => *database = loaded,
        Err(_) => *database = ImportDatabase::new(project.root.clone()),
    }
}

fn register_default_commands(mut registry: ResMut<EditorCommandRegistry>) {
    for (id, label, shortcut) in [
        ("project.save", "Save Project", Some("Ctrl+S")),
        ("project.play", "Play", Some("F6")),
        ("project.pause", "Pause", Some("F7")),
        ("project.stop", "Stop", Some("F8")),
        ("project.export", "Export Project", Some("Ctrl+Shift+B")),
        ("edit.undo", "Undo", Some("Ctrl+Z")),
        ("edit.redo", "Redo", Some("Ctrl+Y")),
        ("scene.save", "Save Scene", Some("Ctrl+Shift+S")),
        ("scene.open", "Open Scene", Some("Ctrl+O")),
        ("scene.new_entity", "Create Entity", Some("Ctrl+Shift+A")),
        ("scene.duplicate", "Duplicate Entity", Some("Ctrl+D")),
        ("scene.delete", "Delete Entity", Some("Delete")),
        ("scene.validate", "Validate Scene", Some("Ctrl+Shift+V")),
        ("scene.prefab_create", "Create Prefab", Some("Ctrl+P")),
        ("assets.refresh", "Refresh Assets", Some("F5")),
        ("assets.import", "Import Assets", Some("Ctrl+Shift+I")),
        ("editor.command_palette", "Command Palette", Some("Ctrl+Shift+P")),
    ] {
        registry.register(EditorCommand {
            id: EditorCommandId(id),
            label,
            shortcut,
        });
    }
}
