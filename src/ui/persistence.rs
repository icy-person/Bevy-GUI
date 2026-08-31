use std::path::PathBuf;
use bevy::prelude::*;
use crate::{editor::EditorUiState,project::{save_project,ProjectState},scene::{save_scene,EditorPrimitive,SceneDocument,ScenePrimitive,SceneVisual}};

pub trait SceneSaveItem{fn into_saved(self)->(Entity,String,Transform,Option<Entity>,bool,SceneVisual);}
impl SceneSaveItem for (Entity,String,Transform,Option<Entity>,bool){fn into_saved(self)->(Entity,String,Transform,Option<Entity>,bool,SceneVisual){let(entity,name,transform,parent,visible)=self;let primitive=match name.to_ascii_lowercase().as_str(){"cube"|"duplicate cube"=>ScenePrimitive::Cube,"plane"=>ScenePrimitive::Plane,"sphere"=>ScenePrimitive::Sphere,"capsule"=>ScenePrimitive::Capsule,_=>ScenePrimitive::None};(entity,name,transform,parent,visible,SceneVisual{primitive,..default()})}}
impl SceneSaveItem for (Entity,String,Transform,Option<Entity>,bool,SceneVisual){fn into_saved(self)->(Entity,String,Transform,Option<Entity>,bool,SceneVisual){self}}

pub fn save_editor_project<I,T>(project:&mut ProjectState,state:&mut EditorUiState,entities:I)
where I:IntoIterator<Item=T>, T:SceneSaveItem{
    let document=SceneDocument::from_entities_with_visuals(entities.into_iter().map(SceneSaveItem::into_saved));
    let relative=project.main_scene.clone().unwrap_or_else(||PathBuf::from(".bevy-gui/untitled.scene.json"));let root=project.root.clone();let path=root.join(relative);
    match save_scene(&path,&document){Ok(())=>{project.main_scene=path.strip_prefix(&root).ok().map(PathBuf::from);match save_project(&root,project){Ok(())=>{project.dirty=false;state.status=format!("Saved {} scene nodes",document.entities.len());},Err(error)=>state.status=format!("Scene saved; project manifest failed: {error}")}},Err(error)=>state.status=format!("Save failed: {error}")}
}

pub fn visual_for_entity(primitive:Option<&EditorPrimitive>)->SceneVisual{SceneVisual{primitive:primitive.map(|p|p.0).unwrap_or(ScenePrimitive::None),..default()}}
#[cfg(test)]mod tests{use super::*;#[test]fn legacy_items_preserve_common_primitives_by_name(){let value=(Entity::from_bits(1),"Cube".to_owned(),Transform::default(),None,true).into_saved();assert_eq!(value.5.primitive,ScenePrimitive::Cube);}}
