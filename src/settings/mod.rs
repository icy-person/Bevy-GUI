//! Persistent editor settings.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{fs, io, path::{Path, PathBuf}};
use thiserror::Error;

const SETTINGS_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorSettings {
    pub version: u32,
    pub appearance: AppearanceSettings,
    pub editor: EditorBehaviorSettings,
    pub viewport: ViewportSettings,
    pub input: InputSettings,
    pub graphics: GraphicsSettings,
    pub project: ProjectSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceSettings {
    pub theme: String,
    pub accent: [u8; 3],
    pub ui_scale: f32,
    pub compact_controls: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorBehaviorSettings {
    pub autosave: bool,
    pub autosave_seconds: f32,
    pub confirm_delete: bool,
    pub restore_layout: bool,
    pub show_fps: bool,
    pub start_in_2d: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportSettings {
    pub grid_2d: bool,
    pub grid_3d: bool,
    pub grid_size: f32,
    pub snap_enabled: bool,
    pub snap_translation: f32,
    pub snap_rotation_degrees: f32,
    pub snap_scale: f32,
    pub camera_move_speed: f32,
    pub camera_pan_speed: f32,
    pub camera_zoom_speed: f32,
    pub camera_orbit_speed: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputSettings {
    pub forward: String,
    pub backward: String,
    pub left: String,
    pub right: String,
    pub up: String,
    pub down: String,
    pub focus: String,
    pub duplicate: String,
    pub delete: String,
    pub save: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsSettings {
    pub backend: String,
    pub msaa_samples: u32,
    pub vsync: bool,
    pub hdr: bool,
    pub render_scale: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    pub main_scene: String,
    pub assets_directory: String,
    pub build_directory: String,
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            version: SETTINGS_VERSION,
            appearance: AppearanceSettings {
                theme: "Material Dark".into(),
                accent: [103, 80, 164],
                ui_scale: 1.0,
                compact_controls: false,
            },
            editor: EditorBehaviorSettings {
                autosave: false,
                autosave_seconds: 300.0,
                confirm_delete: true,
                restore_layout: true,
                show_fps: false,
                start_in_2d: false,
            },
            viewport: ViewportSettings {
                grid_2d: true,
                grid_3d: true,
                grid_size: 1.0,
                snap_enabled: true,
                snap_translation: 0.5,
                snap_rotation_degrees: 15.0,
                snap_scale: 0.25,
                camera_move_speed: 8.0,
                camera_pan_speed: 8.0,
                camera_zoom_speed: 1.0,
                camera_orbit_speed: 1.0,
            },
            input: InputSettings {
                forward: "W".into(),
                backward: "S".into(),
                left: "A".into(),
                right: "D".into(),
                up: "E".into(),
                down: "Q".into(),
                focus: "F".into(),
                duplicate: "Ctrl+D".into(),
                delete: "Delete".into(),
                save: "Ctrl+S".into(),
            },
            graphics: GraphicsSettings {
                backend: "Vulkan/Auto".into(),
                msaa_samples: 4,
                vsync: true,
                hdr: false,
                render_scale: 1.0,
            },
            project: ProjectSettings {
                main_scene: "".into(),
                assets_directory: "assets".into(),
                build_directory: "build".into(),
            },
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct EditorSettingsState {
    pub settings: EditorSettings,
    pub dirty: bool,
    pub path: Option<PathBuf>,
    pub last_error: Option<String>,
}

impl Default for EditorSettingsState {
    fn default() -> Self {
        Self {
            settings: EditorSettings::default(),
            dirty: false,
            path: None,
            last_error: None,
        }
    }
}

#[derive(Debug, Error)]
pub enum SettingsIoError {
    #[error("failed to create settings directory: {0}")]
    CreateDirectory(#[source] io::Error),
    #[error("failed to read settings: {0}")]
    Read(#[source] io::Error),
    #[error("failed to parse settings: {0}")]
    Parse(#[source] serde_json::Error),
    #[error("failed to serialize settings: {0}")]
    Serialize(#[source] serde_json::Error),
    #[error("failed to write settings: {0}")]
    Write(#[source] io::Error),
}

pub fn settings_path(root: &Path) -> PathBuf {
    root.join(".bevy-gui").join("editor-settings.json")
}

pub fn load_settings(root: &Path) -> Result<EditorSettings, SettingsIoError> {
    let path = settings_path(root);
    if !path.exists() {
        return Ok(EditorSettings::default());
    }
    let json = fs::read_to_string(path).map_err(SettingsIoError::Read)?;
    let mut settings: EditorSettings = serde_json::from_str(&json).map_err(SettingsIoError::Parse)?;
    if settings.version == 0 {
        settings.version = SETTINGS_VERSION;
    }
    Ok(settings)
}

pub fn save_settings(root: &Path, settings: &EditorSettings) -> Result<(), SettingsIoError> {
    let path = settings_path(root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(SettingsIoError::CreateDirectory)?;
    }
    let json = serde_json::to_string_pretty(settings).map_err(SettingsIoError::Serialize)?;
    fs::write(path, json).map_err(SettingsIoError::Write)
}

pub fn install_settings(app: &mut App) {
    app.init_resource::<EditorSettingsState>()
        .add_systems(PostStartup, load_settings_on_startup);
}

fn load_settings_on_startup(
    project: Res<ProjectState>,
    mut state: ResMut<EditorSettingsState>,
) {
    match load_settings(&project.root) {
        Ok(settings) => {
            state.settings = settings;
            state.path = Some(settings_path(&project.root));
            state.last_error = None;
        }
        Err(error) => state.last_error = Some(error.to_string()),
    }
}
