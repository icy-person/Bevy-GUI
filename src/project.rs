use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct ProjectState {
    pub name: String,
    pub root: PathBuf,
    pub dirty: bool,
    pub mode: EditorMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorMode {
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
        }
    }
}
