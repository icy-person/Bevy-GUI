use bevy::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::engine_runtime::EngineRuntimeCorePlugin;
use crate::input::EngineInputPlugin;
use crate::project::{EditorMode,ProjectState};
use crate::runtime::PlaySession;
use crate::scene::{EditorPrimitive,ScenePrimitive};
use crate::scene_model::EditorParent;
use crate::viewport::EditorEntity;

#[derive(Resource,Debug,Clone)]pub struct EngineSettings{pub fixed_timestep_hz:f64,pub max_delta_seconds:f32,pub enable_physics:bool,pub enable_audio:bool,pub enable_hot_reload:bool}
impl Default for EngineSettings{fn default()->Self{Self{fixed_timestep_hz:60.0,max_delta_seconds:0.1,enable_physics:true,enable_audio:true,enable_hot_reload:true}}}
#[derive(Resource,Debug,Default,Clone)]pub struct EnginePaths{pub project_root:PathBuf,pub assets:PathBuf,pub scenes:PathBuf,pub cache:PathBuf}
#[derive(Resource,Debug,Clone)]pub struct EngineRuntimeConfig{pub project_root:PathBuf,pub main_scene:Option<PathBuf>}
impl Default for EngineRuntimeConfig{fn default()->Self{Self{project_root:std::env::current_dir().unwrap_or_else(|_|PathBuf::from(".")),main_scene:None}}}
impl EngineRuntimeConfig{pub fn with_scene(path:impl Into<PathBuf>)->Self{Self{main_scene:Some(path.into()),..Default::default()}}pub fn scene_path(&self)->Option<PathBuf>{self.main_scene.as_ref().map(|scene|if scene.is_absolute(){scene.clone()}else{self.project_root.join(scene)})}}
#[derive(Component,Debug,Clone,Copy)]pub struct RuntimeEntity;

pub struct EngineRuntimePlugin;
impl Plugin for EngineRuntimePlugin{fn build(&self,app:&mut App){app.add_plugins((EngineRuntimeCorePlugin,EngineInputPlugin)).init_resource::<EngineSettings>().init_resource::<EnginePaths>().init_resource::<EngineRuntimeConfig>().add_plugins(avian3d::prelude::PhysicsPlugins::default()).add_systems(Startup,(initialize_runtime_paths,load_configured_scene));}}
fn initialize_runtime_paths(config:Res<EngineRuntimeConfig>,mut paths:ResMut<EnginePaths>){paths.project_root=config.project_root.clone();paths.assets=config.project_root.join("assets");paths.scenes=config.project_root.join("scenes");paths.cache=config.project_root.join(".bevy-gui");}
fn load_configured_scene(mut commands:Commands,config:Res<EngineRuntimeConfig>,mut meshes:ResMut<Assets<Mesh>>,mut materials:ResMut<Assets<StandardMaterial>>,asset_server:Res<AssetServer>){let Some(path)=config.scene_path()else{return};match crate::scene::load_scene(&path){Ok(document)=>{crate::scene::spawn_scene_with_renderables(&mut commands,&mut meshes,&mut materials,&asset_server,&document);}Err(error)=>warn!("Failed to load scene {}: {}",path.display(),error)}}
pub fn load_runtime_scene(commands:&mut Commands,config:&EngineRuntimeConfig)->Result<Vec<Entity>,crate::scene::SceneIoError>{let path=config.scene_path().ok_or_else(||crate::scene::SceneIoError::Read(std::io::Error::other("no main scene configured")))?;let document=crate::scene::load_scene(&path)?;Ok(crate::scene::spawn_scene(commands,&document))}

pub struct EnginePlugin;
impl Plugin for EnginePlugin{fn build(&self,app:&mut App){app.init_resource::<EngineSettings>().init_resource::<EnginePaths>().init_resource::<PlaySession>().add_plugins(avian3d::prelude::PhysicsPlugins::default()).add_systems(Startup,initialize_engine_paths).add_systems(Update,sync_editor_runtime_mode);}}
fn initialize_engine_paths(project:Res<ProjectState>,mut paths:ResMut<EnginePaths>){paths.project_root=project.root.clone();paths.assets=project.root.join("assets");paths.scenes=project.root.join("scenes");paths.cache=project.root.join(".bevy-gui");}
fn sync_editor_runtime_mode(mut commands:Commands,project:Res<ProjectState>,mut play_session:ResMut<PlaySession>,editor_entities:Query<(Entity,&Transform,Option<&Name>,Option<&Visibility>,Option<&Mesh3d>,Option<&MeshMaterial3d<StandardMaterial>>,Option<&EditorParent>,Option<&EditorPrimitive>),With<EditorEntity>>,runtime_entities:Query<Entity,With<RuntimeEntity>>){if !project.is_changed(){return}match project.mode{EditorMode::Play if play_session.is_editing()=>{let snapshot=snapshot_editor_scene(&editor_entities);play_session.start(snapshot);spawn_runtime_preview(&mut commands,&editor_entities)},EditorMode::Paused if play_session.is_running()=>play_session.pause(),EditorMode::Play if play_session.is_paused()=>play_session.resume(),EditorMode::Edit if !play_session.is_editing()=>{for entity in &runtime_entities{commands.entity(entity).despawn();}let _=play_session.stop()},_=>{}}}
fn snapshot_editor_scene(entities:&Query<(Entity,&Transform,Option<&Name>,Option<&Visibility>,Option<&Mesh3d>,Option<&MeshMaterial3d<StandardMaterial>>,Option<&EditorParent>,Option<&EditorPrimitive>),With<EditorEntity>>)->crate::scene::SceneDocument{crate::scene::SceneDocument::from_entities_with_visuals(entities.iter().map(|(entity,transform,name,visibility,_mesh,_material,parent,primitive)|{let visual=crate::scene::SceneVisual{primitive:primitive.map(|v|v.0).unwrap_or(ScenePrimitive::None),..Default::default()};(entity,name.map(|n|n.as_str().to_owned()).unwrap_or_else(||"Entity".into()),*transform,parent.and_then(|p|p.0),visibility.map(|v|!matches!(v,Visibility::Hidden)).unwrap_or(true),visual)}))}
fn spawn_runtime_preview(commands:&mut Commands,entities:&Query<(Entity,&Transform,Option<&Name>,Option<&Visibility>,Option<&Mesh3d>,Option<&MeshMaterial3d<StandardMaterial>>,Option<&EditorParent>,Option<&EditorPrimitive>),With<EditorEntity>>){let mut map=HashMap::<Entity,Entity>::new();let mut pending_parents=Vec::new();for(source,transform,name,visibility,mesh,material,parent,_primitive)in entities.iter(){let mut spawned=commands.spawn((RuntimeEntity,*transform,if visibility.map(|v|!matches!(v,Visibility::Hidden)).unwrap_or(true){Visibility::Inherited}else{Visibility::Hidden},Name::new(format!("Runtime: {}",name.map(|n|n.as_str()).unwrap_or("Entity")))));if let Some(mesh)=mesh{spawned.insert(mesh.clone());}if let Some(material)=material{spawned.insert(material.clone());}let runtime_entity=spawned.id();map.insert(source,runtime_entity);pending_parents.push((runtime_entity,parent.and_then(|p|p.0)));}for(runtime_entity,parent)in pending_parents{if let Some(parent_entity)=parent.and_then(|p|map.get(&p).copied()){commands.entity(runtime_entity).insert(ChildOf(parent_entity));}}}
pub fn project_engine_paths(root:&std::path::Path)->EnginePaths{EnginePaths{project_root:root.to_path_buf(),assets:root.join("assets"),scenes:root.join("scenes"),cache:root.join(".bevy-gui")}}
