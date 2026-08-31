use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{fs, io, path::Path};
use thiserror::Error;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct SceneNode;
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditorPrimitive(pub ScenePrimitive);
#[derive(Component, Debug, Clone)]
pub struct EditorVisual(pub SceneVisual);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ScenePrimitive { Cube, Plane, Sphere, Capsule, None }
impl Default for ScenePrimitive { fn default()->Self{Self::None} }
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SceneNodeKind { Empty, Mesh, Camera3d, DirectionalLight, PointLight, SpotLight }
impl Default for SceneNodeKind { fn default()->Self{Self::Mesh} }
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SceneBody { None, Static, Dynamic, Kinematic }
impl Default for SceneBody { fn default()->Self{Self::None} }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneAudio { #[serde(default)] pub asset:Option<String>, #[serde(default)] pub looping:bool, #[serde(default="default_audio_volume")] pub volume:f32, #[serde(default)] pub spatial:bool }
impl Default for SceneAudio { fn default()->Self{Self{asset:None,looping:false,volume:1.0,spatial:false}} }
fn default_audio_volume()->f32{1.0}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneVisual { #[serde(default)] pub primitive:ScenePrimitive, #[serde(default)] pub mesh_asset:Option<String>, #[serde(default)] pub material_asset:Option<String>, #[serde(default="default_color")] pub base_color:[f32;4], #[serde(default)] pub metallic:f32, #[serde(default="default_roughness")] pub roughness:f32, #[serde(default)] pub body:SceneBody, #[serde(default)] pub collision:bool, #[serde(default)] pub audio:SceneAudio }
impl Default for SceneVisual { fn default()->Self{Self{primitive:ScenePrimitive::None,mesh_asset:None,material_asset:None,base_color:default_color(),metallic:0.0,roughness:0.5,body:SceneBody::None,collision:false,audio:SceneAudio::default()}} }
fn default_color()->[f32;4]{[0.2,0.55,0.95,1.0]}
fn default_roughness()->f32{0.5}
fn default_visible()->bool{true}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneDocument { pub format_version:u32,pub entities:Vec<SceneEntity> }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneEntity { pub id:u64,pub parent:Option<u64>,pub name:String,pub translation:[f32;3],pub rotation:[f32;4],pub scale:[f32;3],#[serde(default="default_visible")]pub visible:bool,#[serde(default)]pub kind:SceneNodeKind,#[serde(default)]pub visual:SceneVisual }

impl SceneDocument { pub fn from_entities<I>(entities:I)->Self where I:IntoIterator<Item=(Entity,String,Transform,Option<Entity>)>{build_document(entities.into_iter().map(|(e,n,t,p)|(e,n,t,p,true,SceneNodeKind::Mesh,SceneVisual::default())))} pub fn from_entities_with_visibility<I>(entities:I)->Self where I:IntoIterator<Item=(Entity,String,Transform,Option<Entity>,bool)>{build_document(entities.into_iter().map(|(e,n,t,p,v)|(e,n,t,p,v,SceneNodeKind::Mesh,SceneVisual::default())))} pub fn from_entities_with_visuals<I>(entities:I)->Self where I:IntoIterator<Item=(Entity,String,Transform,Option<Entity>,bool,SceneVisual)>{build_document(entities.into_iter().map(|(e,n,t,p,v,s)|(e,n,t,p,v,SceneNodeKind::Mesh,s)))} pub fn from_entities_with_kinds<I>(entities:I)->Self where I:IntoIterator<Item=(Entity,String,Transform,Option<Entity>,bool,SceneNodeKind,SceneVisual)>{build_document(entities)} }
fn build_document<I>(entities:I)->SceneDocument where I:IntoIterator<Item=(Entity,String,Transform,Option<Entity>,bool,SceneNodeKind,SceneVisual)>{let snapshot:Vec<_>=entities.into_iter().collect();let ids:std::collections::BTreeMap<_,_>=snapshot.iter().enumerate().map(|(i,(e,_,_,_,_,_,_))|(*e,i as u64+1)).collect();SceneDocument{format_version:7,entities:snapshot.into_iter().enumerate().map(|(i,(_,name,transform,parent,visible,kind,visual))|SceneEntity{id:i as u64+1,parent:parent.and_then(|p|ids.get(&p).copied()),name,translation:transform.translation.to_array(),rotation:[transform.rotation.x,transform.rotation.y,transform.rotation.z,transform.rotation.w],scale:transform.scale.to_array(),visible,kind,visual}).collect()}}
impl SceneEntity{pub fn transform(&self)->Transform{Transform{translation:Vec3::from_array(self.translation),rotation:Quat::from_xyzw(self.rotation[0],self.rotation[1],self.rotation[2],self.rotation[3]),scale:Vec3::from_array(self.scale)}}}
fn spawn_base(commands:&mut Commands,entity:&SceneEntity)->Entity{commands.spawn((Name::new(entity.name.clone()),entity.transform(),if entity.visible{Visibility::Visible}else{Visibility::Hidden},SceneNode,crate::EditorParent(None),EditorPrimitive(entity.visual.primitive),EditorVisual(entity.visual.clone()))).id()}
pub fn spawn_scene(commands:&mut Commands,document:&SceneDocument)->Vec<Entity>{let mut by_id=std::collections::BTreeMap::new();let mut pending=Vec::new();let mut spawned=Vec::with_capacity(document.entities.len());for entity in &document.entities{let e=spawn_base(commands,entity);by_id.insert(entity.id,e);pending.push((e,entity.parent));spawned.push(e)}for(e,p)in pending{if let Some(parent)=p.and_then(|id|by_id.get(&id).copied()){commands.entity(e).insert(crate::EditorParent(Some(parent)));}}spawned}

pub fn spawn_scene_with_renderables(commands:&mut Commands,meshes:&mut Assets<Mesh>,materials:&mut Assets<StandardMaterial>,asset_server:&AssetServer,document:&SceneDocument)->Vec<Entity>{let mut by_id=std::collections::BTreeMap::new();let mut pending=Vec::new();let mut spawned=Vec::with_capacity(document.entities.len());for entity in &document.entities{let visual=&entity.visual;let material=materials.add(StandardMaterial{base_color:Color::srgba(visual.base_color[0],visual.base_color[1],visual.base_color[2],visual.base_color[3]),metallic:visual.metallic.clamp(0.0,1.0),perceptual_roughness:visual.roughness.clamp(0.0,1.0),..default()});let e=spawn_base(commands,entity);match entity.kind{SceneNodeKind::Empty=>{},SceneNodeKind::Mesh=>match visual.primitive{ScenePrimitive::Cube=>{commands.entity(e).insert((Mesh3d(meshes.add(Cuboid::new(1.0,1.0,1.0))),MeshMaterial3d(material.clone())));},ScenePrimitive::Plane=>{commands.entity(e).insert((Mesh3d(meshes.add(Plane3d::default().mesh().size(2.0,2.0))),MeshMaterial3d(material.clone())));},ScenePrimitive::Sphere=>{commands.entity(e).insert((Mesh3d(meshes.add(Sphere::new(0.5).mesh().uv(32,18))),MeshMaterial3d(material.clone())));},ScenePrimitive::Capsule=>{commands.entity(e).insert((Mesh3d(meshes.add(Capsule3d::new(0.35,0.8))),MeshMaterial3d(material.clone())));},ScenePrimitive::None=>{if let Some(asset)=&visual.mesh_asset{commands.entity(e).insert(SceneRoot(asset_server.load(asset.clone())));}}},SceneNodeKind::Camera3d=>{commands.entity(e).insert(Camera3d::default());},SceneNodeKind::DirectionalLight=>{commands.entity(e).insert(DirectionalLight{illuminance:10000.0,shadows_enabled:true,..default()});},SceneNodeKind::PointLight=>{commands.entity(e).insert(PointLight{intensity:1500000.0,shadows_enabled:true,..default()});},SceneNodeKind::SpotLight=>{commands.entity(e).insert(SpotLight{intensity:1000000.0,shadows_enabled:true,..default()});}}if visual.collision{let collider=match visual.primitive{ScenePrimitive::Sphere=>avian3d::prelude::Collider::sphere(0.5),ScenePrimitive::Capsule=>avian3d::prelude::Collider::capsule(0.35,0.8),_=>avian3d::prelude::Collider::cuboid(0.5,0.5,0.5)};match visual.body{SceneBody::Static=>{commands.entity(e).insert((avian3d::prelude::RigidBody::Static,collider));},SceneBody::Dynamic=>{commands.entity(e).insert((avian3d::prelude::RigidBody::Dynamic,collider));},SceneBody::Kinematic=>{commands.entity(e).insert((avian3d::prelude::RigidBody::Kinematic,collider));},SceneBody::None=>{}}}if let Some(audio)=&visual.audio.asset{let settings=if visual.audio.looping{PlaybackSettings::LOOP}else{PlaybackSettings::ONCE}.with_volume(Volume::Linear(visual.audio.volume.clamp(0.0,2.0))).with_spatial(visual.audio.spatial);commands.entity(e).insert((AudioPlayer::new(asset_server.load(audio.clone())),settings));}by_id.insert(entity.id,e);pending.push((e,entity.parent));spawned.push(e)}for(e,p)in pending{if let Some(parent)=p.and_then(|id|by_id.get(&id).copied()){commands.entity(e).insert(ChildOf(parent));}}spawned}

#[derive(Debug,Error)]pub enum SceneIoError{#[error("failed to create scene directory: {0}")]CreateDirectory(#[source]io::Error),#[error("failed to serialize scene: {0}")]Serialize(#[source]serde_json::Error),#[error("failed to write scene: {0}")]Write(#[source]io::Error),#[error("failed to read scene: {0}")]Read(#[source]io::Error),#[error("failed to parse scene: {0}")]Parse(#[source]serde_json::Error)}
pub fn save_scene(path:&Path,document:&SceneDocument)->Result<(),SceneIoError>{if let Some(parent)=path.parent(){fs::create_dir_all(parent).map_err(SceneIoError::CreateDirectory)?}let json=serde_json::to_string_pretty(document).map_err(SceneIoError::Serialize)?;fs::write(path,json).map_err(SceneIoError::Write)}
pub fn load_scene(path:&Path)->Result<SceneDocument,SceneIoError>{let json=fs::read_to_string(path).map_err(SceneIoError::Read)?;serde_json::from_str(&json).map_err(SceneIoError::Parse)}
#[cfg(test)]mod tests{use super::*;#[test]fn scene_round_trip(){let doc=SceneDocument::from_entities_with_kinds([(Entity::from_bits(1),"Cube".into(),Transform::default(),None,true,SceneNodeKind::Mesh,SceneVisual{primitive:ScenePrimitive::Cube,body:SceneBody::Static,collision:true,audio:SceneAudio{asset:Some("music/test.ogg".into()),looping:true,volume:0.5,spatial:false},..default()})]);let restored:SceneDocument=serde_json::from_str(&serde_json::to_string(&doc).unwrap()).unwrap();assert_eq!(restored.format_version,7);assert!(restored.entities[0].visual.collision);assert_eq!(restored.entities[0].visual.audio.asset.as_deref(),Some("music/test.ogg"));assert!(restored.entities[0].visual.audio.looping);}}
