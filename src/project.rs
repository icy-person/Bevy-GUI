use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{fs, io, path::{Path, PathBuf}};
use thiserror::Error;

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct ProjectState {
    pub name: String,
    pub root: PathBuf,
    pub dirty: bool,
    pub mode: EditorMode,
    pub main_scene: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EditorMode {
    #[default]
    Edit,
    Play,
    Paused,
}

impl Default for ProjectState {
    fn default() -> Self {
        Self {
            name: "Untitled Project".into(),
            root: PathBuf::from("."),
            dirty: false,
            mode: EditorMode::Edit,
            main_scene: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectManifest {
    pub format_version: u32,
    pub name: String,
    pub main_scene: Option<String>,
    pub asset_directory: String,
    pub render_backend: String,
}

impl ProjectManifest {
    pub fn from_state(state: &ProjectState) -> Self {
        Self {
            format_version: 1,
            name: state.name.clone(),
            main_scene: state.main_scene.as_ref().map(|path| path.display().to_string()),
            asset_directory: "assets".into(),
            render_backend: "auto".into(),
        }
    }

    pub fn apply_to_state(&self, root: PathBuf) -> ProjectState {
        ProjectState {
            name: self.name.clone(),
            root,
            dirty: false,
            mode: EditorMode::Edit,
            main_scene: self.main_scene.as_ref().map(PathBuf::from),
        }
    }
}

#[derive(Debug, Error)]
pub enum ProjectIoError {
    #[error("failed to create project directory: {0}")]
    CreateDirectory(#[source] io::Error),
    #[error("failed to serialize project: {0}")]
    Serialize(#[source] serde_json::Error),
    #[error("failed to write project: {0}")]
    Write(#[source] io::Error),
    #[error("failed to read project: {0}")]
    Read(#[source] io::Error),
    #[error("failed to parse project: {0}")]
    Parse(#[source] serde_json::Error),
}

pub fn project_file(root: &Path) -> PathBuf {
    root.join("project.godot-rs.json")
}

pub fn save_project(root: &Path, state: &ProjectState) -> Result<(), ProjectIoError> {
    fs::create_dir_all(root).map_err(ProjectIoError::CreateDirectory)?;
    let document = ProjectManifest::from_state(state);
    let json = serde_json::to_string_pretty(&document).map_err(ProjectIoError::Serialize)?;
    fs::write(project_file(root), json).map_err(ProjectIoError::Write)
}

pub fn load_project(root: &Path) -> Result<ProjectState, ProjectIoError> {
    let json = fs::read_to_string(project_file(root)).map_err(ProjectIoError::Read)?;
    let manifest: ProjectManifest = serde_json::from_str(&json).map_err(ProjectIoError::Parse)?;
    Ok(manifest.apply_to_state(root.to_path_buf()))
}
