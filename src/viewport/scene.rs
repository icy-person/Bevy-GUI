use bevy::camera_controller::free_camera::FreeCamera;
use bevy::dev_tools::infinite_grid::InfiniteGrid;
use bevy::prelude::*;
use std::collections::BTreeMap;

use super::components::{Editor3dCamera, Editor3dGrid, EditorEntity, InitialSelected};
use crate::{load_scene, EditorParent, EditorPrimitive, ProjectState, ScenePrimitive};

pub fn setup_editor_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    project: Res<ProjectState>,
) {
    commands.spawn((
        Camera3d::default(),
        Editor3dCamera,
        Transform::from_xyz(7.0, 5.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
        FreeCamera::default(),
        TransformGizmoCamera,
        Name::new("Editor Camera 3D"),
    ));
    commands.spawn((InfiniteGrid, Editor3dGrid, Name::new("Editor Grid 3D")));
    commands.spawn((
        DirectionalLight { illuminance: 12_000.0, shadow_maps_enabled: true, ..default() },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, -0.5, 0.0)),
        Name::new("Key Light"),
    ));

    if let Some(main_scene) = &project.main_scene {
        let path = project.root.join(main_scene);
        if let Ok(document) = load_scene(&path) {
            let mut first = None;
            let mut by_id = BTreeMap::new();
            let mut parents = Vec::new();
            for entity in document.entities {
                let spawned = spawn_editor_node(&mut commands, &mut meshes, &mut materials, &asset_server, &entity);
                commands.entity(spawned).observe(select_clicked_entity);
                by_id.insert(entity.id, spawned);
                parents.push((spawned, entity.parent));
                first.get_or_insert(spawned);
            }
            for (entity, parent_id) in parents {
                let parent = parent_id.and_then(|id| by_id.get(&id).copied());
                commands.entity(entity).insert(EditorParent(parent));
            }
            if let Some(entity) = first {
                commands.insert_resource(InitialSelected(entity));
                return;
            }
        }
    }

    let material = materials.add(StandardMaterial { base_color: Color::srgb(0.15, 0.55, 0.95), metallic: 0.05, perceptual_roughness: 0.32, ..default() });
    let cube = commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::new(2.0, 2.0, 2.0))),
            MeshMaterial3d(material),
            EditorPrimitive(ScenePrimitive::Cube),
            Transform::default(),
            Visibility::Visible,
            Pickable::default(),
            Name::new("Player"),
            EditorEntity,
            EditorParent(None),
        ))
        .id();
    commands.entity(cube).observe(select_clicked_entity);
    commands.insert_resource(InitialSelected(cube));
}

fn spawn_editor_node(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    asset_server: &AssetServer,
    entity: &crate::scene::SceneEntity,
) -> Entity {
    let material = materials.add(StandardMaterial {
        base_color: Color::srgba(entity.visual.base_color[0], entity.visual.base_color[1], entity.visual.base_color[2], entity.visual.base_color[3]),
        metallic: entity.visual.metallic.clamp(0.0, 1.0),
        perceptual_roughness: entity.visual.roughness.clamp(0.0, 1.0),
        ..default()
    });
    let mut builder = commands.spawn((
        entity.transform(),
        if entity.visible { Visibility::Visible } else { Visibility::Hidden },
        Pickable::default(),
        Name::new(entity.name.clone()),
        EditorEntity,
        EditorParent(None),
        EditorPrimitive(entity.visual.primitive),
    ));
    match entity.visual.primitive {
        ScenePrimitive::Cube => { builder.insert((Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))), MeshMaterial3d(material))); }
        ScenePrimitive::Plane => { builder.insert((Mesh3d(meshes.add(Plane3d::default().mesh().size(2.0, 2.0))), MeshMaterial3d(material))); }
        ScenePrimitive::Sphere => { builder.insert((Mesh3d(meshes.add(Sphere::new(0.5).mesh().uv(24, 16))), MeshMaterial3d(material))); }
        ScenePrimitive::Capsule => { builder.insert((Mesh3d(meshes.add(Capsule3d::new(0.35, 0.8).mesh().resolution(16))), MeshMaterial3d(material))); }
        ScenePrimitive::None => {
            if let Some(asset) = &entity.visual.mesh_asset {
                builder.insert(SceneRoot(asset_server.load(asset.clone())));
            }
        }
    }
    builder.id()
}

fn select_clicked_entity(event: On<Pointer<Click>>, mut selection: ResMut<crate::SelectionState>) {
    selection.select(event.entity);
}
