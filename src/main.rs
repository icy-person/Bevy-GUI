use bevy::prelude::*;
use bevy_gui::BevyGuiPlugin;

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
        .run();
}
