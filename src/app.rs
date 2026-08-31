use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use crate::{
    animation::AnimationRuntimePlugin,
    asset_pipeline::ImportDatabase,
    assets::install_asset_database,
    command::{EditorCommand, EditorCommandBus, EditorCommandId, EditorCommandRegistry, HistoryCommandEvent},
    command_executor::{execute_editor_commands, execute_history_commands, CommandExecutionState},
    component_registry::install_component_registry,
    docking::EditorDockState,
    editor::register_builtin_state,
    engine::EnginePlugin,
    engine_features::EngineFeaturesPlugin,
    engine_tools_ui::EngineToolsUiPlugin,
    jackdaw_ui::JackdawUiPlugin,
    panel::PanelRegistry,
    plugins::install_builtin_editor_plugins,
    profiler::install_profiler,
    project::ProjectState,
    scene_model::SceneEditorState,
    scene_tools::SceneSelectionSet,
    settings::install_settings,
    shader_graph::ShaderGraphPlugin,
    ui::{command_palette::install_command_palette, install_editor_ui},
    viewport::install_viewport,
    viewport2d::install_2d_viewport,
    visual_scripting::VisualScriptingPlugin,
};

pub struct BevyGuiPlugin;

impl Plugin for BevyGuiPlugin {
    fn build(&self, app:&mut App) {
        app.add_plugins(EguiPlugin::default())
            .add_plugins(JackdawUiPlugin)
            .add_plugins(EnginePlugin)
            .add_plugins(EngineFeaturesPlugin)
            .add_plugins(EngineToolsUiPlugin)
            .add_plugins(AnimationRuntimePlugin)
            .add_plugins(ShaderGraphPlugin)
            .add_plugins(VisualScriptingPlugin)
            .init_resource::<ProjectState>()
            .init_resource::<EditorCommandRegistry>()
            .init_resource::<EditorCommandBus>()
            .init_resource::<CommandExecutionState>()
            .init_resource::<crate::TransformHistory>()
            .add_event::<HistoryCommandEvent>()
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
            .add_systems(Startup, (register_default_commands, load_initial_editor_scene).chain())
            .add_systems(Update, (execute_editor_commands, execute_history_commands));
    }
}

fn load_import_database(project:Res<ProjectState>,mut database:ResMut<ImportDatabase>) { match ImportDatabase::load(project.root.clone()){Ok(loaded)=>*database=loaded,Err(_)=>*database=ImportDatabase::new(project.root.clone())} }

fn load_initial_editor_scene(mut commands:Commands,project:Res<ProjectState>,mut scene_state:ResMut<SceneEditorState>,mut meshes:ResMut<Assets<Mesh>>,mut materials:ResMut<Assets<StandardMaterial>>,asset_server:Res<AssetServer>) {
    let Some(relative)=project.main_scene.as_ref() else{return};
    let path=project.root.join(relative);
    let Ok(document)=crate::scene::load_scene(&path) else{return};
    let spawned=crate::scene::spawn_scene_with_renderables(&mut commands,&mut meshes,&mut materials,&asset_server,&document);
    for entity in spawned.iter().copied(){commands.entity(entity).insert((crate::viewport::EditorEntity,Pickable::default()));}
    scene_state.path=Some(relative.clone());
    scene_state.saved_revision=scene_state.revision;
}

fn register_default_commands(mut registry:ResMut<EditorCommandRegistry>) {
    for (id,label,shortcut) in [
        ("project.save","Save Project",Some("Ctrl+S")),("project.play","Play",Some("F6")),("project.pause","Pause",Some("F7")),("project.stop","Stop",Some("F8")),("project.export","Export Project",Some("Ctrl+Shift+B")),
        ("edit.undo","Undo",Some("Ctrl+Z")),("edit.redo","Redo",Some("Ctrl+Y")),("scene.save","Save Scene",Some("Ctrl+Shift+S")),("scene.open","Open Scene",Some("Ctrl+O")),
        ("scene.new_entity","Create Entity",Some("Ctrl+Shift+A")),("scene.new_cube","Create Cube at Cursor",Some("Shift+A")),("scene.new_plane","Create Plane at Cursor",Some("Shift+P")),("scene.new_sphere","Create Sphere at Cursor",Some("Shift+S")),("scene.new_capsule","Create Capsule at Cursor",Some("Shift+C")),
        ("scene.duplicate","Duplicate Entity",Some("Ctrl+D")),("scene.delete","Delete Entity",Some("Delete")),("scene.validate","Validate Scene",Some("Ctrl+Shift+V")),("scene.prefab_create","Create Prefab",Some("Ctrl+P")),
        ("assets.refresh","Refresh Assets",Some("F5")),("assets.import","Import Assets",Some("Ctrl+Shift+I")),("editor.command_palette","Command Palette",Some("Ctrl+Shift+P")),
    ] { registry.register(EditorCommand{id:EditorCommandId(id),label,shortcut}); }
}
