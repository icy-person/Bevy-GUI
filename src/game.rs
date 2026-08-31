use bevy::prelude::*;
use crate::{EngineRuntimeConfig, EngineRuntimePlugin};

#[derive(Debug, Clone)]
pub struct GameConfig {
    pub title: String,
    pub width: f32,
    pub height: f32,
    pub scene: Option<std::path::PathBuf>,
}

impl Default for GameConfig {
    fn default() -> Self { Self { title: "Bevy Game".into(), width: 1280.0, height: 720.0, scene: None } }
}

impl GameConfig {
    pub fn with_scene(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.scene = Some(path.into());
        self
    }
}

pub fn build_game_app(config: GameConfig) -> App {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: config.title,
            resolution: WindowResolution::new(config.width, config.height),
            ..default()
        }),
        ..default()
    }));
    let runtime_config = match config.scene {
        Some(scene) => EngineRuntimeConfig::with_scene(scene),
        None => EngineRuntimeConfig::default(),
    };
    app.insert_resource(runtime_config).add_plugins(EngineRuntimePlugin);
    app
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn game_config_has_safe_defaults() {
        let config = GameConfig::default();
        assert_eq!(config.width, 1280.0);
        assert_eq!(config.height, 720.0);
        assert!(config.scene.is_none());
    }
}
