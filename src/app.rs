use bevy::camera_controller::free_camera::{FreeCamera, FreeCameraPlugin};
use bevy::dev_tools::infinite_grid::{InfiniteGrid, InfiniteGridPlugin};
use bevy::picking::prelude::*;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use std::{fs, path::{Path, PathBuf}};

use crate::{
    command::{EditorCommand, EditorCommandBus, EditorCommandId, EditorCommandRegistry},
    docking::{show_dock_area, DockViewer, EditorDockState, TransformEdit},
    editor::{register_builtin_state, EditorUiState},
    history::{TransformHistory, TransformSnapshot},
    plugins::install_builtin_editor_plugins,
    project::{save_project, EditorMode, ProjectState},
    runtime::PlaySession,
    scene::{save_scene, SceneDocument},
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
            FreeCameraPlugin,
            InfiniteGridPlugin,
        ))
        .init_resource::<SelectionState>()
        .init_resource::<ProjectState>()
        .init_resource::<EditorCommandRegistry>()
        .init_resource::<EditorCommandBus>()
        .init_resource::<crate::PanelRegistry>()
        .init_resource::<EditorDockState>()
        .init_resource::<PlaySession>()
        .insert_resource(TransformHistory::with_capacity(256))
        .init_resource::<GizmoHistoryTracker>();

        register_builtin_state(app);
        install_builtin_editor_plugins(app);

        app.add_systems(Startup, (register_default_commands, setup_editor_scene).chain())
            .add_systems(
                Update,
                (
                    select_initial_entity,
                    keyboard_editor_shortcuts,
                    apply_runtime_mode,
                    sync_transform_gizmo_focus,
                )
                    .chain(),
            )
            .add_systems(PostUpdate, record_finished_gizmo_drag.after(TransformGizmoSystems))
            .add_systems(EguiPrimaryContextPass, editor_ui_system);
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

fn setup_editor_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(7.0, 5.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
        FreeCamera::default(),
        TransformGizmoCamera,
        Name::new("Editor Camera"),
    ));
    commands.spawn((InfiniteGrid, Name::new("Editor Grid")));
    commands.spawn((
        DirectionalLight {
            illuminance: 12_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, -0.5, 0.0)),
        Name::new("Key Light"),
    ));

    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.15, 0.55, 0.95),
        metallic: 0.05,
        perceptual_roughness: 0.32,
        ..default()
    });

    let cube = commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::new(2.0, 2.0, 2.0))),
            MeshMaterial3d(material),
            Transform::default(),
            Pickable::default(),
            Name::new("Player"),
            EditorEntity,
        ))
        .id();
    commands.entity(cube).observe(select_clicked_entity);
    commands.insert_resource(InitialSelected(cube));
}

#[derive(Component)]
struct EditorEntity;

fn select_initial_entity(
    initial: Option<Res<InitialSelected>>,
    mut selection: ResMut<SelectionState>,
) {
    if selection.primary().is_none() {
        if let Some(initial) = initial {
            selection.select(initial.0);
        }
    }
}

fn select_clicked_entity(event: On<Pointer<Click>>, mut selection: ResMut<SelectionState>) {
    selection.select(event.entity);
}

fn keyboard_editor_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    mut project: ResMut<ProjectState>,
    mut gizmo: ResMut<TransformGizmoSettings>,
    mut history: ResMut<TransformHistory>,
    mut transforms: Query<&mut Transform>,
    mut bus: ResMut<EditorCommandBus>,
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
        bus.emit(EditorCommandId("edit.undo"));
    }
    if ctrl && keys.just_pressed(KeyCode::KeyY) {
        history.redo(&mut transforms);
        project.dirty = true;
        bus.emit(EditorCommandId("edit.redo"));
    }
    if ctrl && keys.just_pressed(KeyCode::KeyS) {
        bus.emit(EditorCommandId("project.save"));
    }
    if ctrl && keys.just_pressed(KeyCode::KeyD) {
        bus.emit(EditorCommandId("scene.duplicate"));
    }
    if ctrl && keys.just_pressed(KeyCode::KeyA) {
        bus.emit(EditorCommandId("scene.new_entity"));
    }
    if keys.just_pressed(KeyCode::Delete) {
        bus.emit(EditorCommandId("scene.delete"));
    }
    if keys.just_pressed(KeyCode::F6) {
        project.mode = EditorMode::Play;
        bus.emit(EditorCommandId("project.play"));
    }
    if keys.just_pressed(KeyCode::F7) {
        project.mode = EditorMode::Paused;
        bus.emit(EditorCommandId("project.pause"));
    }
    if keys.just_pressed(KeyCode::F8) {
        project.mode = EditorMode::Edit;
        bus.emit(EditorCommandId("project.stop"));
    }
}

fn apply_runtime_mode(
    mut commands: Commands,
    mut project: ResMut<ProjectState>,
    mut session: ResMut<PlaySession>,
    names: Query<(Entity, Option<&Name>, &Transform), With<EditorEntity>>,
    mut transforms: Query<&mut Transform, With<EditorEntity>>,
) {
    match project.mode {
        EditorMode::Play if session.snapshot.is_none() => {
            let snapshot = SceneDocument::from_entities(names.iter().map(|(_, name, transform)| {
                (
                    name.map(|value| value.as_str().to_owned())
                        .unwrap_or_else(|| "Entity".into()),
                    *transform,
                )
            }));
            session.start(snapshot);
        }
        EditorMode::Paused => session.pause(),
        EditorMode::Edit if session.snapshot.is_some() => {
            if let Some(snapshot) = session.stop() {
                let mut by_name = std::collections::BTreeMap::new();
                for entity in snapshot.entities {
                    by_name.insert(entity.name, entity);
                }
                for (_, name, mut transform) in &mut names.iter().map(|(e, n, _)| (e, n, e)) {
                    let _ = (name, &mut transform);
                }
                for (entity, name, _) in names {
                    if let Some(saved) = name.and_then(|value| by_name.get(value.as_str())) {
                        if let Ok(mut current) = transforms.get_mut(entity) {
                            current.translation = Vec3::from_array(saved.translation);
                            current.rotation = Quat::from_xyzw(
                                saved.rotation[0],
                                saved.rotation[1],
                                saved.rotation[2],
                                saved.rotation[3],
                            );
                            current.scale = Vec3::from_array(saved.scale);
                        }
                    }
                }
                commands.queue(|_world: &mut World| {});
            }
        }
        _ => {}
    }
}

fn sync_transform_gizmo_focus(
    mut commands: Commands,
    selection: Res<SelectionState>,
    query: Query<(Entity, Option<&TransformGizmoFocus>)>,
) {
    for (entity, focus) in &query {
        match (selection.contains(entity), focus.is_some()) {
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
    registry: Res<EditorCommandRegistry>,
    plugins: Res<crate::editor::EditorPluginRegistry>,
    mut history: ResMut<TransformHistory>,
    transforms: Query<&Transform, With<EditorEntity>>,
    names: Query<(Entity, Option<&Name>), With<EditorEntity>>,
    mut commands: Commands,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    let mut save_requested = false;

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
                save_requested = true;
            }
            ui.label(format!("Mode: {:?}", project.mode));
            ui.label(format!("{} selected", selection.entities.len()));
            ui.label(format!("Undo {} / Redo {}", history.undo_len(), history.redo_len()));
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

    let selected_transform = selection.primary().and_then(|entity| {
        transforms.get(entity).ok().map(|transform| {
            let (x, y, z) = transform.rotation.to_euler(EulerRot::XYZ);
            TransformEdit {
                entity,
                translation: transform.translation,
                rotation: Vec3::new(x.to_degrees(), y.to_degrees(), z.to_degrees()),
                scale: transform.scale,
            }
        })
    });

    let assets = scan_assets(&project.root, 5, 1000);
    let plugin_names: Vec<String> = plugins
        .iter()
        .map(|(name, version)| format!("{name} v{version}"))
        .collect();

    let mut viewer = DockViewer {
        project: &mut project,
        selection: &mut selection,
        ui_state: &mut state,
        entities: &entities,
        selected_transform,
        assets: &assets,
        plugin_names: &plugin_names,
        command_count: registry.iter().count(),
        transform_edit: None,
        viewport_focused: false,
        create_entity: false,
        delete_entity: None,
        duplicate_entity: None,
        save_requested: false,
    };

    egui::CentralPanel::default().show(ctx, |ui| {
        show_dock_area(ui, &mut dock, &mut viewer);
    });

    let create_entity = viewer.create_entity;
    let delete_entity = viewer.delete_entity;
    let duplicate_entity = viewer.duplicate_entity;
    save_requested |= viewer.save_requested;
    let transform_edit = viewer.transform_edit;
    drop(viewer);

    if create_entity {
        let entity = commands
            .spawn((Transform::default(), Name::new("Entity"), Pickable::default(), EditorEntity))
            .id();
        commands.entity(entity).observe(select_clicked_entity);
        selection.select(entity);
        project.dirty = true;
    }

    if let Some(entity) = duplicate_entity {
        if let Ok(current) = transforms.get(entity) {
            let new_entity = commands
                .spawn((
                    *current,
                    Name::new("Duplicate"),
                    Pickable::default(),
                    EditorEntity,
                ))
                .id();
            commands.entity(new_entity).observe(select_clicked_entity);
            selection.select(new_entity);
            project.dirty = true;
        }
    }

    if let Some(entity) = delete_entity {
        if selection.contains(entity) {
            commands.entity(entity).despawn();
            selection.entities.retain(|current| *current != entity);
            selection.focused = selection.entities.last().copied();
            project.dirty = true;
        }
    }

    if let Some(edit) = transform_edit {
        if let Ok(current) = transforms.get(edit.entity) {
            apply_transform_edit(&mut history, &mut commands, &mut project, edit, *current);
        }
    }

    if save_requested {
        let items = entities.iter().filter_map(|(entity, name)| {
            transforms.get(*entity).ok().map(|transform| (name.clone(), *transform))
        });
        let document = SceneDocument::from_entities(items);
        let scene_path = project
            .main_scene
            .clone()
            .unwrap_or_else(|| PathBuf::from(".bevy-gui/untitled.scene.json"));
        let scene_path = project.root.join(scene_path);
        match save_scene(&scene_path, &document) {
            Ok(()) => {
                project.main_scene = scene_path
                    .strip_prefix(&project.root)
                    .ok()
                    .map(PathBuf::from);
                match save_project(&project.root, &project) {
                    Ok(()) => {
                        project.dirty = false;
                        state.status = format!("Saved {} entities", document.entities.len());
                    }
                    Err(error) => {
                        state.status = format!("Scene saved; project manifest failed: {error}");
                    }
                }
            }
            Err(error) => {
                state.status = format!("Save failed: {error}");
            }
        }
    }

    Ok(())
}

fn apply_transform_edit(
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
