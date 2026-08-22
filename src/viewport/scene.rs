use bevy::camera_controller::free_camera::FreeCamera;
use bevy::dev_tools::infinite_grid::InfiniteGrid;
use bevy::prelude::*;
use bevy::picking::prelude::*;

use super::components::{EditorEntity, InitialSelected};
use crate::{load_scene, ProjectState};

pub fn setup_editor_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    project: Res<ProjectState>,
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

    if let Some(main_scene) = &project.main_scene {
        let path = project.root.join(main_scene);
        if let Ok(document) = load_scene(&path) {
            let mut first = None;
            for entity in document.entities {
                let id = commands
                    .spawn((
                        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
                        MeshMaterial3d(materials.add(StandardMaterial::default())),
                        Transform {
                            translation: Vec3::from_array(entity.translation),
                            rotation: Quat::from_xyzw(
                                entity.rotation[0],
                                entity.rotation[1],
                                entity.rotation[2],
                                entity.rotation[3],
                            ),
                            scale: Vec3::from_array(entity.scale),
                        },
                        Pickable::default(),
                        Name::new(entity.name),
                        EditorEntity,
                    ))
                    .id();
                commands.entity(id).observe(select_clicked_entity);
                first.get_or_insert(id);
            }
            if let Some(entity) = first {
                commands.insert_resource(InitialSelected(entity));
                return;
            }
        }
    }

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
