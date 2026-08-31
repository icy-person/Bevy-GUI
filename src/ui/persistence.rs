use std::path::PathBuf;
use bevy::prelude::*;
use crate::{editor::EditorUiState, project::{save_project,ProjectState},scene::{save_scene,EditorPrimitive,SceneDocument,ScenePrimitive,SceneVisual},scene_model::EditorParent,viewport::EditorEntity};

pub fn save_editor_project(project:&mut ProjectState,state:&mut EditorUiState,entities:&[(Entity,String,Transform,Option<Entity>,bool,SceneVisual)]){
    let document=SceneDocument::from_entities_with_visuals(entities.iter().map(|(entity,name,transform,parent,visible,visual)|(*entity,name.clone(),*transform,*parent,*visible,visual.clone())));
    let relative=project.main_scene.clone().unwrap_or_else(||PathBuf::from(".bevy-gui/untitled.scene.json"));
    let root=project.root.clone();
    let path=root.join(relative);
    match save_scene(&path,&document){
        Ok(())=>{project.main_scene=path.strip_prefix(&root).ok().map(PathBuf::from);match save_project(&root,project){Ok(())=>{project.dirty=false;state.status=format!("Saved {} scene nodes",document.entities.len());},Err(error)=>state.status=format!("Scene saved; project manifest failed: {error}")}},
        Err(error)=>state.status=format!("Save failed: {error}"),
    }
}

pub fn visual_for_entity(primitive:Option<&EditorPrimitive>)->SceneVisual{SceneVisual{primitive:primitive.map(|p|p.0).unwrap_or(ScenePrimitive::None),..default()}}

#[cfg(test)]
mod tests{use super::*;#[test]fn visual_defaults_to_none(){assert_eq!(visual_for_entity(None).primitive,ScenePrimitive::None);}}
