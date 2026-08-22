use std::path::PathBuf;

use bevy::prelude::*;

use crate::{
    editor::EditorUiState,
    project::{save_project, ProjectState},
    scene::{save_scene, SceneDocument},
};

pub fn save_editor_project(
    project: &mut ProjectState,
    state: &mut EditorUiState,
    entities: &[(Entity, String, Transform, Option<Entity>)],
) {
    let document = SceneDocument::from_world(
        entities
            .iter()
            .map(|(entity, name, transform, parent)| (*entity, name.clone(), *transform, *parent)),
    );
    let relative = project
        .main_scene
        .clone()
        .unwrap_or_else(|| PathBuf::from(".bevy-gui/untitled.scene.json"));
    let root = project.root.clone();
    let path = root.join(relative);

    match save_scene(&path, &document) {
        Ok(()) => {
            project.main_scene = path.strip_prefix(&root).ok().map(PathBuf::from);
            match save_project(&root, project) {
                Ok(()) => {
                    project.dirty = false;
                    state.status = format!("Saved {} scene nodes", document.entities.len());
                }
                Err(error) => {
                    state.status = format!("Scene saved; project manifest failed: {error}");
                }
            }
        }
        Err(error) => state.status = format!("Save failed: {error}"),
    }
}
