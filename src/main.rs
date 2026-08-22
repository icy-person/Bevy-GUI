use bevy::prelude::*;
use bevy_gui::{load_project, BevyGuiPlugin, ProjectState};
use std::env;

fn load_project_on_startup(mut project: ResMut<ProjectState>) {
    let root = env::current_dir().unwrap_or_else(|_| ".".into());
    if let Ok(loaded) = load_project(&root) {
        *project = loaded;
    } else {
        project.root = root;
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy-GUI Editor".into(),
                resolution: (1440, 900).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(BevyGuiPlugin)
        .add_systems(Startup, load_project_on_startup.after(bevy_gui::app::setup_editor_scene))
        .run();
}
