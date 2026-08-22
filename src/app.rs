use bevy::prelude::*;
use bevy::picking::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use std::fs;
use std::path::{Path, PathBuf};

use crate::{
    command::{EditorCommand, EditorCommandId, EditorCommandRegistry},
    docking::{show_dock_area, DockViewer, EditorDockState, TransformEdit},
    editor::{register_builtin_state, EditorUiState},
    history::{TransformHistory, TransformSnapshot},
    plugins::install_builtin_editor_plugins,
    project::{EditorMode, ProjectState},
    selection::SelectionState,
};

pub struct BevyGuiPlugin;

#[derive(Resource)]
struct InitialSelected(Entity);

#[derive(Resource, Default)]
struct GizmoHistoryTracker {
    active_last_frame: bool,
}

impl Plugin for BevyGuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            EguiPlugin::default(),
            DefaultPickingPlugins,
            TransformGizmoPlugin,
        ))
        .init_resource::<SelectionState>()
        .init_resource::<ProjectState>()
        .init_resource::<EditorCommandRegistry>()
        .init_resource::<crate::PanelRegistry>()
        .init_resource::<EditorDockState>()
        .insert_resource(TransformHistory::with_capacity(256))
        .init_resource::<GizmoHistoryTracker>();

        register_builtin_state(app);
        install_builtin_editor_plugins(app);

        app.add_systems(
            Startup,
            (register_default_commands, setup_editor_scene).chain(),
        )
        .add_systems(
            Update,
            (
                select_initial_entity,
                keyboard_editor_shortcuts,
                sync_transform_gizmo_focus,
                editor_mode_tick,
            )
                .chain(),
        )
        .add_systems(
            PostUpdate,
            record_finished_gizmo_drag.after(TransformGizmoSystems),
        )
        .add_systems(EguiPrimaryContextPass, editor_ui_system);
    }
}

fn register_default_commands(mut registry: ResMut<EditorCommandRegistry>) {
    registry.register(EditorCommand {
        id: EditorCommandId("project.save"),
        label: "Save Project",
        shortcut: Some("Ctrl+S"),
    });
    registry.register(EditorCommand {
        id: EditorCommandId("project.play"),
        label: "Play",
        shortcut: Some("F6"),
    });
    registry.register(EditorCommand {
        id: EditorCommandId("project.pause"),
        label: "Pause",
        shortcut: Some("F7"),
    });
    registry.register(EditorCommand {
        id: EditorCommandId("project.stop"),
        label: "Stop",
        shortcut: Some("F8"),
    });
    registry.register(EditorCommand {
        id: EditorCommandId("edit.undo"),
        label: "Undo",
        shortcut: Some("Ctrl+Z"),
    });
    registry.register(EditorCommand {
        id: EditorCommandId("edit.redo"),
        label: "Redo",
        shortcut: Some("Ctrl+Y"),
    });
}

fn setup_editor_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(7.0, 5.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
        TransformGizmoCamera,
        Name::new("Editor Camera"),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 12_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, -0.5, 0.0)),
        Name::new("Key Light"),
    ));

    let cube = commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::new(2.0, 2.0, 2.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.15, 0.55, 0.95),
                metallic: 0.05,
                perceptual_roughness: 0.32,
                ..default()
            })),
            Transform::default(),
            Pickable::default(),
            Name::new("Player"),
        ))
        .id();

    commands
        .entity(cube)
        .observe(select_clicked_entity);
    commands.insert_resource(InitialSelected(cube));
}

fn select_initial_entity(
    initial: Option<Res<InitialSelected>>,
    mut selection: ResMut<SelectionState>,
) {
    if selection.entity.is_none() {
        if let Some(initial) = initial {
            selection.select(initial.0);
        }
    }
}

fn select_clicked_entity(
    event: On<Pointer<Click>>,
    mut selection: ResMut<SelectionState>,
) {
    selection.select(event.entity);
}

fn sync_transform_gizmo_focus(
    mut commands: Commands,
    selection: Res<SelectionState>,
    query: Query<(Entity, Option<&TransformGizmoFocus>)>,
) {
    for (entity, focus) in &query {
        match (selection.entity == Some(entity), focus.is_some()) {
            (true, false) => {
                commands.entity(entity).insert(TransformGizmoFocus);
            }
            (false, true) => {
                commands.entity(entity).remove::<TransformGizmoFocus>();
            }
            _ => {}
        }
    }
}

fn keyboard_editor_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    mut project: ResMut<ProjectState>,
    mut gizmo: ResMut<TransformGizmoSettings>,
    mut history: ResMut<TransformHistory>,
    mut transforms: Query<&mut Transform>,
) {
    if keys.just_pressed(KeyCode::KeyW) {
        gizmo.mode = TransformGizmoMode::Translate;
    }
    if keys.just_pressed(KeyCode::KeyE) {
        gizmo.mode = TransformGizmoMode::Rotate;
    }
    if keys.just_pressed(KeyCode::KeyR) {
        gizmo.mode = TransformGizmoMode::Scale;
    }
    if keys.just_pressed(KeyCode::KeyX) {
        gizmo.space = match gizmo.space {
            TransformGizmoSpace::World => TransformGizmoSpace::Local,
            TransformGizmoSpace::Local => TransformGizmoSpace::World,
        };
    }

    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    if ctrl && keys.just_pressed(KeyCode::KeyZ) {
        history.undo(&mut transforms);
        project.dirty = true;
    }
    if ctrl && keys.just_pressed(KeyCode::KeyY) {
        history.redo(&mut transforms);
        project.dirty = true;
    }

    if keys.just_pressed(KeyCode::F6) {
        project.mode = EditorMode::Play;
    }
    if keys.just_pressed(KeyCode::F7) {
        project.mode = EditorMode::Paused;
    }
    if keys.just_pressed(KeyCode::F8) {
        project.mode = EditorMode::Edit;
    }
    if ctrl && keys.just_pressed(KeyCode::KeyS) {
        project.dirty = false;
    }
}

fn editor_mode_tick(time: Res<Time>, project: Res<ProjectState>) {
    if project.mode == EditorMode::Play {
        let _elapsed = time.delta_secs();
        // Runtime systems can consume EditorMode::Play without changing the UI kernel.
    }
}

fn record_finished_gizmo_drag(
    mut tracker: ResMut<GizmoHistoryTracker>,
    gizmo_state: Res<TransformGizmoState>,
    mut history: ResMut<TransformHistory>,
    mut project: ResMut<ProjectState>,
) {
    if tracker.active_last_frame && !gizmo_state.active {
        if let Some(entity) = gizmo_state.entity {
            history.push(TransformSnapshot {
                entity,
                transform: gizmo_state.start_transform,
            });
            project.dirty = true;
        }
    }
    tracker.active_last_frame = gizmo_state.active;
}

fn editor_ui_system(
    mut contexts: EguiContexts,
    mut dock: ResMut<EditorDockState>,
    mut state: ResMut<EditorUiState>,
    mut project: ResMut<ProjectState>,
    mut selection: ResMut<SelectionState>,
    commands_registry: Res<EditorCommandRegistry>,
    plugins: Res<crate::editor::EditorPluginRegistry>,
    mut history: ResMut<TransformHistory>,
    transforms: Query<&Transform>,
    names: Query<(Entity, Option<&Name>)>,
    mut commands: Commands,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    egui::Panel::top("editor_menu").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("Bevy-GUI");
            ui.separator();
            for label in ["File", "Edit", "View", "Assets", "Scene", "Entity", "Build"] {
                ui.menu_button(label, |ui| {
                    ui.label("Plugin-provided command surface");
                });
            }
            ui.separator();
            if ui.button("Save").clicked() {
                project.dirty = false;
            }
            if ui.button("Undo").clicked() {
                state.status = format!("Undo stack: {}", history.undo_len());
            }
            if ui.button("Redo").clicked() {
                state.status = format!("Redo stack: {}", history.redo_len());
            }
            ui.separator();
            ui.label(format!("Mode: {:?}", project.mode));
            if project.dirty {
                ui.label("● Dirty");
            }
        });
    });

    let entities: Vec<(Entity, String)> = names
        .iter()
        .map(|(entity, name)| {
            (
                entity,
                name.map(|value| value.as_str().to_owned())
                    .unwrap_or_else(|| format!("Entity {:?}", entity)),
            )
        })
        .collect();

    let selected_transform = selection
        .entity
        .and_then(|entity| transforms.get(entity).ok().map(|transform| TransformEdit {
            entity,
            translation: transform.translation,
            rotation: {
                let (x, y, z) = transform.rotation.to_euler(EulerRot::XYZ);
                Vec3::new(x.to_degrees(), y.to_degrees(), z.to_degrees())
            },
            scale: transform.scale,
        }));

    let assets = scan_assets(&project.root, 4, 500);
    let plugin_names: Vec<String> = plugins
        .iter()
        .map(|(name, version)| format!("{name} v{version}"))
        .collect();

    let command_count = commands_registry.iter().count();
    let mut viewer = DockViewer {
        project: &mut project,
        selection: &mut selection,
        ui_state: &mut state,
        entities: &entities,
        selected_transform,
        assets: &assets,
        plugin_names: &plugin_names,
        command_count,
        transform_edit: None,
        viewport_focused: false,
    };

    egui::CentralPanel::default().show(ctx, |ui| {
        show_dock_area(ui, &mut dock, &mut viewer);
    });

    if let Some(edit) = viewer.transform_edit {
        if let Ok(current) = transforms.get(edit.entity) {
            history_push_and_apply(&mut history, &mut commands, &mut project, edit, *current);
        }
    }

    Ok(())
}

fn history_push_and_apply(
    history: &mut TransformHistory,
    commands: &mut Commands,
    project: &mut ProjectState,
    edit: TransformEdit,
    current: Transform,
) {
    let rotation = Quat::from_euler(
        EulerRot::XYZ,
        edit.rotation.x.to_radians(),
        edit.rotation.y.to_radians(),
        edit.rotation.z.to_radians(),
    );
    let next = Transform {
        translation: edit.translation,
        rotation,
        scale: edit.scale,
    };

    if current != next {
        history.push(TransformSnapshot {
            entity: edit.entity,
            transform: current,
        });
        commands.entity(edit.entity).insert(next);
        project.dirty = true;
    }
}

fn scan_assets(root: &Path, max_depth: usize, max_files: usize) -> Vec<String> {
    let mut output = Vec::new();
    visit_assets(root, root, 0, max_depth, max_files, &mut output);
    output.sort();
    output
}

fn visit_assets(
    root: &Path,
    current: &Path,
    depth: usize,
    max_depth: usize,
    max_files: usize,
    output: &mut Vec<String>,
) {
    if depth > max_depth || output.len() >= max_files {
        return;
    }
    let entries = match fs::read_dir(current) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        if output.len() >= max_files {
            break;
        }
        let path: PathBuf = entry.path();
        if path
            .file_name()
            .is_some_and(|name| name == "target" || name == ".git")
        {
            continue;
        }
        if path.is_dir() {
            visit_assets(root, &path, depth + 1, max_depth, max_files, output);
        } else if path.is_file() {
            if let Ok(relative) = path.strip_prefix(root) {
                output.push(relative.display().to_string());
            }
        }
    }
}
