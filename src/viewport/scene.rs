use bevy::camera_controller::free_camera::FreeCamera;
use bevy::dev_tools::infinite_grid::InfiniteGrid;
use bevy::prelude::*;
use bevy::picking::prelude::*;

use super::components::{EditorEntity, InitialSelected};

pub fn setup_editor_scene(
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

fn select_clicked_entity(event: On<Pointer<Click>>, mut selection: ResMut<crate::SelectionState>) {
    selection.select(event.entity);
}
