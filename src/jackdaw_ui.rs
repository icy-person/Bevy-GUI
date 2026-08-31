use bevy::prelude::*;

/// Optional integration point for Jackdaw Feathers.
/// The editor shell currently uses egui for its inspector, while keeping this
/// plugin available for future Bevy-native widget integration.
pub struct JackdawUiPlugin;

impl Plugin for JackdawUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(jackdaw_feathers::EditorFeathersPlugin);
    }
}
