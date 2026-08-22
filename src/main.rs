use bevy::prelude::*;
use bevy_gui::{load_project, BevyGuiPlugin, ProjectState};
use std::env;

fn project_state_from_disk() -> ProjectState {
    let root = env::current_dir().unwrap_or_else(|_| ".".into());
    load_project(&root).unwrap_or_else(|_| ProjectState {
        root,
        ..Default::default()
    })
}

fn main() {
    App::new()
        .insert_resource(project_state_from_disk())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy-GUI Editor".into(),
                resolution: (1440, 900).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(BevyGuiPlugin)
        .run();
}
