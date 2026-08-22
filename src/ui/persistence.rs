use std::path::PathBuf;

use bevy::prelude::*;

use crate::{
    editor::EditorUiState,
    project::{save_project, ProjectState},
    scene::{save_scene, SceneDocument},
    viewport::EditorEntity,
};

pub fn save_editor_project(
    project: &mut ProjectState,
    state: &mut EditorUiState,
    entities: &[(Entity, String)],
    transforms: &Query<&Transform, With<EditorEntity>>,
) {
    let items = entities.iter().filter_map(|(entity, name)| {
        transforms
            .get(*entity)
            .ok()
            .map(|transform| (name.clone(), *transform))
    });
    let document = SceneDocument::from_entities(items);
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
                    state.status = format!("Saved {} entities", document.entities.len());
                }
                Err(error) => {
                    state.status = format!("Scene saved; project manifest failed: {error}");
                }
            }
        }
        Err(error) => state.status = format!("Save failed: {error}"),
    }
}
