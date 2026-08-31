use bevy::prelude::*;
use std::path::PathBuf;

use crate::project::ProjectState;
use crate::runtime::{PlayMode, PlaySession};

/// Shared runtime configuration for the editor and standalone game.
#[derive(Resource, Debug, Clone)]
pub struct EngineSettings {
    pub fixed_timestep_hz: f64,
    pub max_delta_seconds: f32,
    pub enable_physics: bool,
    pub enable_audio: bool,
    pub enable_hot_reload: bool,
}

impl Default for EngineSettings {
    fn default() -> Self {
        Self {
            fixed_timestep_hz: 60.0,
            max_delta_seconds: 0.1,
            enable_physics: true,
            enable_audio: true,
            enable_hot_reload: true,
        }
    }
}

#[derive(Resource, Debug, Default, Clone)]
pub struct EnginePaths {
    pub project_root: PathBuf,
    pub assets: PathBuf,
    pub scenes: PathBuf,
    pub cache: PathBuf,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct EngineResetEvent;

#[derive(Event, Debug, Clone, Copy)]
pub struct EnginePlayEvent;

#[derive(Event, Debug, Clone, Copy)]
pub struct EnginePauseEvent;

#[derive(Event, Debug, Clone, Copy)]
pub struct EngineStopEvent;

pub struct EnginePlugin;

impl Plugin for EnginePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EngineSettings>()
            .init_resource::<EnginePaths>()
            .add_event::<EngineResetEvent>()
            .add_event::<EnginePlayEvent>()
            .add_event::<EnginePauseEvent>()
            .add_event::<EngineStopEvent>()
            .add_systems(Startup, initialize_engine_paths)
            .add_systems(
                Update,
                (
                    consume_play_events,
                    consume_pause_events,
                    consume_stop_events,
                    clamp_frame_time,
                ),
            );
    }
}

fn initialize_engine_paths(project: Res<ProjectState>, mut paths: ResMut<EnginePaths>) {
    paths.project_root = project.root.clone();
    paths.assets = project.root.join("assets");
    paths.scenes = project.root.join("scenes");
    paths.cache = project.root.join(".bevy-gui");
}

fn consume_play_events(mut events: EventReader<EnginePlayEvent>, mut session: ResMut<PlaySession>) {
    for _ in events.read() {
        session.mode = PlayMode::Playing;
    }
}

fn consume_pause_events(mut events: EventReader<EnginePauseEvent>, mut session: ResMut<PlaySession>) {
    for _ in events.read() {
        session.mode = PlayMode::Paused;
    }
}

fn consume_stop_events(mut events: EventReader<EngineStopEvent>, mut session: ResMut<PlaySession>) {
    for _ in events.read() {
        session.mode = PlayMode::Stopped;
    }
}

fn clamp_frame_time(time: Res<Time>, settings: Res<EngineSettings>) {
    let _ = (time.delta_seconds().min(settings.max_delta_seconds), settings.fixed_timestep_hz);
}

pub fn project_engine_paths(root: &std::path::Path) -> EnginePaths {
    EnginePaths {
        project_root: root.to_path_buf(),
        assets: root.join("assets"),
        scenes: root.join("scenes"),
        cache: root.join(".bevy-gui"),
    }
}
