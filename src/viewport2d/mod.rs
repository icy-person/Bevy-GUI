//! 2D editor viewport: orthographic camera, grid, pan/zoom and 2D authoring helpers.

use bevy::{
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseButton},
    prelude::*,
};

use crate::{
    editor::{EditorUiState, ViewportMode},
    settings::EditorSettingsState,
};

#[derive(Component)]
pub struct Editor2dCamera;

#[derive(Component)]
pub struct Editor2dEntity;

#[derive(Resource, Default)]
pub struct Editor2dState {
    pub camera: Option<Entity>,
    pub initialized: bool,
}

pub fn install_2d_viewport(app: &mut App) {
    app.init_resource::<Editor2dState>()
        .add_systems(Startup, setup_2d_world)
        .add_systems(
            Update,
            (sync_2d_visibility, control_2d_camera, draw_2d_grid).chain(),
        );
}

fn setup_2d_world(
    mut commands: Commands,
    mut state: ResMut<Editor2dState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let camera = commands.spawn((Camera2d, Editor2dCamera, Visibility::Hidden)).id();
    state.camera = Some(camera);

    let positions = [
        (Vec2::new(-3.0, 1.5), Vec2::new(2.0, 1.5)),
        (Vec2::new(0.5, -1.0), Vec2::new(1.5, 2.0)),
        (Vec2::new(3.0, 1.0), Vec2::new(1.0, 1.0)),
    ];
    for (index, (position, size)) in positions.into_iter().enumerate() {
        commands.spawn((
            Mesh2d(meshes.add(Rectangle::from_size(size))),
            MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgb(
                0.18 + index as f32 * 0.12,
                0.42,
                0.82,
            )))),
            Transform::from_xyz(position.x, position.y, 0.0),
            Name::new(format!("2D Sprite {}", index + 1)),
            Editor2dEntity,
        ));
    }
    state.initialized = true;
}

fn sync_2d_visibility(
    editor: Res<EditorUiState>,
    mut cameras: Query<
        &mut Visibility,
        (With<Editor2dCamera>, Without<Editor2dEntity>),
    >,
    mut entities: Query<
        &mut Visibility,
        (With<Editor2dEntity>, Without<Editor2dCamera>),
    >,
) {
    let visible = editor.viewport_mode == ViewportMode::TwoD;
    for mut visibility in &mut cameras {
        *visibility = if visible { Visibility::Inherited } else { Visibility::Hidden };
    }
    for mut visibility in &mut entities {
        *visibility = if visible { Visibility::Inherited } else { Visibility::Hidden };
    }
}

fn control_2d_camera(
    editor: Res<EditorUiState>,
    settings: Res<EditorSettingsState>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    mut cameras: Query<(&mut Transform, &mut Projection), With<Editor2dCamera>>,
) {
    if editor.viewport_mode != ViewportMode::TwoD {
        return;
    }
    let Ok((mut transform, mut projection)) = cameras.single_mut() else { return };

    if mouse_buttons.pressed(MouseButton::Middle) && motion.delta != Vec2::ZERO {
        let speed = settings.settings.viewport.camera_pan_speed.max(0.01);
        transform.translation.x -= motion.delta.x * 0.01 * speed;
        transform.translation.y += motion.delta.y * 0.01 * speed;
    }

    if scroll.delta.y.abs() > f32::EPSILON {
        let zoom_speed = settings.settings.viewport.camera_zoom_speed.max(0.01);
        if let Projection::Orthographic(ref mut ortho) = *projection {
            ortho.scale =
                (ortho.scale * (-scroll.delta.y * 0.1 * zoom_speed).exp()).clamp(0.05, 100.0);
        }
    }
}

fn draw_2d_grid(
    editor: Res<EditorUiState>,
    settings: Res<EditorSettingsState>,
    mut gizmos: Gizmos,
) {
    if editor.viewport_mode != ViewportMode::TwoD || !settings.settings.viewport.grid_2d {
        return;
    }
    let spacing = settings.settings.viewport.grid_size.max(0.05);
    gizmos.grid_2d(
        Isometry2d::IDENTITY,
        UVec2::new(60, 60),
        Vec2::splat(spacing),
        Color::srgba(0.35, 0.35, 0.42, 0.22),
    );
}
