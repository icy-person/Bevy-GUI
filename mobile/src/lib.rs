use bevy::prelude::*;
use bevy_gui::{EngineRuntimeConfig, EngineRuntimePlugin};

#[bevy_main]
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(EngineRuntimeConfig::with_scene("scenes/main.scene.json"))
        .add_plugins(EngineRuntimePlugin)
        .add_systems(Startup, setup_mobile)
        .run();
}

fn setup_mobile(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(7.0, 5.0, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight { illuminance: 10_000.0, shadows_enabled: true, ..default() },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, -0.5, 0.0)),
    ));
}
