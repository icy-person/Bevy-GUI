use bevy::camera_controller::free_camera::{FreeCamera, FreeCameraPlugin};
use bevy::dev_tools::infinite_grid::{InfiniteGrid, InfiniteGridPlugin};
use bevy::picking::prelude::*;
use bevy::prelude::*;
use std::collections::BTreeMap;

use crate::{
    command::{EditorCommandBus, EditorCommandId},
    history::{TransformHistory, TransformSnapshot},
    project::{EditorMode, ProjectState},
    runtime::PlaySession,
    scene::SceneDocument,
    selection::SelectionState,
};

#[derive(Resource)]
pub struct InitialSelected(pub Entity);

#[derive(Resource, Default)]
pub struct GizmoHistoryTracker {
    pub active_last_frame: bool,
}

#[derive(Component)]
pub struct EditorEntity;

pub fn install_viewport(app: &mut App) {
    app.add_plugins((
        DefaultPickingPlugins,
        TransformGizmoPlugin,
        FreeCameraPlugin,
        InfiniteGridPlugin,
    ))
    .init_resource::<SelectionState>()
    .init_resource::<PlaySession>()
    .insert_resource(TransformHistory::with_capacity(256))
    .init_resource::<GizmoHistoryTracker>()
    .add_systems(Startup, setup_editor_scene)
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
    .add_systems(
        PostUpdate,
        record_finished_gizmo_drag.after(TransformGizmoSystems),
    );
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

fn select_initial_entity(
    initial: Option<Res<InitialSelected>>,
    mut selection: ResMut<SelectionState>,
) {
    if selection.primary().is_none() && let Some(initial) = initial {
        selection.select(initial.0);
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
    project: Res<ProjectState>,
    mut session: ResMut<PlaySession>,
    mut query: Query<(Entity, Option<&Name>, &mut Transform), With<EditorEntity>>,
) {
    match project.mode {
        EditorMode::Play if session.snapshot.is_none() => {
            let snapshot = SceneDocument::from_entities(query.iter_mut().map(
                |(_, name, transform)| {
                    (
                        name.map(|value| value.as_str().to_owned())
                            .unwrap_or_else(|| "Entity".into()),
                        *transform,
                    )
                },
            ));
            session.start(snapshot);
        }
        EditorMode::Paused => session.pause(),
        EditorMode::Play if session.snapshot.is_some() => session.resume(),
        EditorMode::Edit if session.snapshot.is_some() => {
            if let Some(snapshot) = session.stop() {
                let saved: BTreeMap<_, _> = snapshot
                    .entities
                    .into_iter()
                    .map(|entity| (entity.name.clone(), entity))
                    .collect();
                for (_, name, mut transform) in &mut query {
                    if let Some(saved) = name.and_then(|value| saved.get(value.as_str())) {
                        transform.translation = Vec3::from_array(saved.translation);
                        transform.rotation = Quat::from_xyzw(
                            saved.rotation[0],
                            saved.rotation[1],
                            saved.rotation[2],
                            saved.rotation[3],
                        );
                        transform.scale = Vec3::from_array(saved.scale);
                    }
                }
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
    if tracker.active_last_frame
        && !gizmo_state.active
        && let Some(entity) = gizmo_state.entity
    {
        history.push(TransformSnapshot {
            entity,
            transform: gizmo_state.start_transform,
        });
        project.dirty = true;
    }
    tracker.active_last_frame = gizmo_state.active;
}
