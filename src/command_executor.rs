use bevy::prelude::*;

use crate::{
    asset_pipeline::ImportDatabase,
    assets::AssetDatabase,
    command::{EditorCommandBus, EditorCommandId, HistoryCommandEvent},
    export::{default_profile, export_project},
    prefab::{prefab_path, save_prefab, PrefabDocument},
    project::{save_project, EditorMode, ProjectState},
    scene::{load_scene, EditorPrimitive, EditorVisual, ScenePrimitive, SceneVisual},
    scene_tools::validate_scene,
    scene_model::EditorParent,
    selection::SelectionState,
    viewport::{EditorEntity, ViewportCursor},
};

#[derive(Resource, Default, Debug)]
pub struct CommandExecutionState { pub executed:u64,pub last:Option<EditorCommandId>,pub last_error:Option<String>,pub last_message:Option<String> }

pub fn execute_editor_commands(
    mut bus:ResMut<EditorCommandBus>,
    mut project:ResMut<ProjectState>,
    mut assets:ResMut<AssetDatabase>,
    mut imports:ResMut<ImportDatabase>,
    mut state:ResMut<CommandExecutionState>,
    mut selection:ResMut<SelectionState>,
    cursor:Res<ViewportCursor>,
    mut history_events:EventWriter<HistoryCommandEvent>,
    mut commands:Commands,
    mut meshes:ResMut<Assets<Mesh>>,
    mut materials:ResMut<Assets<StandardMaterial>>,
    asset_server:Res<AssetServer>,
    transforms:Query<(Entity,&Transform,Option<&Name>,Option<&EditorParent>,Option<&Visibility>,Option<&EditorPrimitive>,Option<&EditorVisual>),With<EditorEntity>>,
    names:Query<(Entity,Option<&Name>,Option<&EditorParent>,Option<&Visibility>),With<EditorEntity>>,
) {
    for id in bus.drain() {
        state.executed=state.executed.saturating_add(1);
        state.last=Some(id);
        state.last_error=None;
        state.last_message=None;
        match id.0 {
            "project.save"|"scene.save" => save_current_scene(&project,&transforms,&mut state),
            "scene.open" => open_current_scene(&mut commands,&mut selection,&project,&asset_server,&mut meshes,&mut materials,&transforms,&mut state),
            "project.play"=>project.mode=EditorMode::Play,
            "project.pause"=>project.mode=EditorMode::Paused,
            "project.stop"=>project.mode=EditorMode::Edit,
            "project.export"=>{let profile=default_profile(&project);match export_project(&project,&profile){Ok(report)=>{let executable=report.executable.as_ref().map(|path|format!("; executable {}",path.display())).unwrap_or_default();state.last_message=Some(format!("Built {} files ({} bytes) to {}{}",report.files,report.bytes,report.output.display(),executable));},Err(error)=>state.last_error=Some(error.to_string())}},
            "assets.refresh"=>{assets.refresh_requested=true;state.last_message=Some("Asset scan requested".into())},
            "assets.import"=>{let report=imports.import_all();let _=imports.save();state.last_message=Some(format!("Imported {} assets; {} failed; {} unsupported (generation {})",report.imported,report.failed,report.unsupported,report.generation));if !report.errors.is_empty(){state.last_error=Some(report.errors.join(" | "));}},
            "scene.validate"=>{let Some(main_scene)=project.main_scene.as_ref()else{state.last_error=Some("No main scene is configured".into());continue};match load_scene(&project.root.join(main_scene)){Ok(document)=>{let report=validate_scene(&document);if report.is_valid(){state.last_message=Some(format!("Scene valid: {} entities, {} warnings",document.entities.len(),report.warnings()));}else{state.last_error=Some(format!("Scene validation failed: {} errors, {} warnings",report.errors(),report.warnings()));}},Err(error)=>state.last_error=Some(error.to_string())}},
            "scene.prefab_create"=>create_prefab(&project,&selection,&transforms,&names,&mut state),
            "scene.new_entity"=>spawn_editor_entity(&mut commands,&mut selection,&mut project,&mut state,"Entity",None,Transform::default()),
            "scene.new_cube"=>spawn_editor_entity(&mut commands,&mut selection,&mut project,&mut state,"Cube",Some(ScenePrimitive::Cube),Transform::from_translation(cursor.position_or_zero())),
            "scene.new_plane"=>spawn_editor_entity(&mut commands,&mut selection,&mut project,&mut state,"Plane",Some(ScenePrimitive::Plane),Transform::from_translation(cursor.position_or_zero())),
            "scene.new_sphere"=>spawn_editor_entity_with_assets(&mut commands,&mut selection,&mut project,&mut state,&mut meshes,&mut materials,"Sphere",ScenePrimitive::Sphere,Transform::from_translation(cursor.position_or_zero())),
            "scene.new_capsule"=>spawn_editor_entity_with_assets(&mut commands,&mut selection,&mut project,&mut state,&mut meshes,&mut materials,"Capsule",ScenePrimitive::Capsule,Transform::from_translation(cursor.position_or_zero())),
            "scene.duplicate"=>{if let Some(source)=selection.primary(){if let Ok((_,transform,name,_,visibility,primitive,visual))=transforms.get(source){let copy_name=name.map(|n|format!("{} Copy",n.as_str())).unwrap_or_else(||"Duplicate".into());let entity=commands.spawn((*transform,Name::new(copy_name),visibility.copied().unwrap_or(Visibility::Inherited),EditorEntity,EditorParent(None),Pickable::default(),EditorPrimitive(primitive.map(|p|p.0).unwrap_or(ScenePrimitive::None)),visual.cloned().unwrap_or_default())).id();selection.select(entity);project.dirty=true;state.last_message=Some("Entity duplicated".into())}else{state.last_error=Some("Selected entity is no longer available".into())}}else{state.last_message=Some("Select an entity first".into())}},
            "scene.delete"=>{if let Some(entity)=selection.primary(){commands.entity(entity).despawn();selection.entities.retain(|current|*current!=entity);selection.focused=selection.entities.last().copied();project.dirty=true;state.last_message=Some("Entity deleted".into())}else{state.last_message=Some("Select an entity first".into())}},
            "edit.undo"=>history_events.send(HistoryCommandEvent{undo:true}),
            "edit.redo"=>history_events.send(HistoryCommandEvent{undo:false}),
            _=>{}
        }
    }
}

fn save_current_scene(project:&ProjectState,transforms:&Query<(Entity,&Transform,Option<&Name>,Option<&EditorParent>,Option<&Visibility>,Option<&EditorPrimitive>,Option<&EditorVisual>),With<EditorEntity>>,state:&mut CommandExecutionState){
    let Some(scene)=project.main_scene.as_ref() else {state.last_error=Some("No main scene is configured".into());return};
    let document=crate::scene::SceneDocument::from_entities_with_visuals(transforms.iter().map(|(entity,transform,name,parent,visibility,primitive,visual)|{let mut authored=visual.map(|value|value.0.clone()).unwrap_or_default();authored.primitive=primitive.map(|value|value.0).unwrap_or(authored.primitive);(entity,name.map(|v|v.as_str().to_owned()).unwrap_or_else(||"Entity".into()),*transform,parent.and_then(|v|v.0),visibility.map(|v|!matches!(v,Visibility::Hidden)).unwrap_or(true),authored)}));
    match crate::scene::save_scene(&project.root.join(scene),&document){Ok(())=>state.last_message=Some(format!("Saved {} scene entities",document.entities.len())),Err(error)=>state.last_error=Some(error.to_string())}
}

fn open_current_scene(commands:&mut Commands,selection:&mut SelectionState,project:&ProjectState,asset_server:&AssetServer,meshes:&mut Assets<Mesh>,materials:&mut Assets<StandardMaterial>,current:&Query<(Entity,&Transform,Option<&Name>,Option<&EditorParent>,Option<&Visibility>,Option<&EditorPrimitive>,Option<&EditorVisual>),With<EditorEntity>>,state:&mut CommandExecutionState){
    let Some(scene)=project.main_scene.as_ref() else {state.last_error=Some("No main scene is configured".into());return};
    match load_scene(&project.root.join(scene)) {
        Ok(document)=>{
            for (entity,_,_,_,_,_,_) in current.iter(){commands.entity(entity).despawn();}
            selection.entities.clear();selection.focused=None;
            let spawned=crate::scene::spawn_scene_with_renderables(commands,meshes,materials,asset_server,&document);
            for entity in spawned.iter().copied(){commands.entity(entity).insert(EditorEntity);}
            state.last_message=Some(format!("Opened scene with {} entities",spawned.len()));
        }
        Err(error)=>state.last_error=Some(error.to_string()),
    }
}

fn spawn_editor_entity(commands:&mut Commands,selection:&mut SelectionState,project:&mut ProjectState,state:&mut CommandExecutionState,name:&str,primitive:Option<ScenePrimitive>,transform:Transform){let entity=commands.spawn((transform,Name::new(name),Visibility::Inherited,EditorEntity,EditorParent(None),Pickable::default(),EditorPrimitive(primitive.unwrap_or(ScenePrimitive::None)),EditorVisual(SceneVisual{primitive:primitive.unwrap_or(ScenePrimitive::None),..default()}))).id();selection.select(entity);project.dirty=true;state.last_message=Some(format!("{name} created"));}
fn spawn_editor_entity_with_assets(commands:&mut Commands,selection:&mut SelectionState,project:&mut ProjectState,state:&mut CommandExecutionState,meshes:&mut Assets<Mesh>,materials:&mut Assets<StandardMaterial>,name:&str,primitive:ScenePrimitive,transform:Transform){let mesh=match primitive{ScenePrimitive::Sphere=>meshes.add(Sphere::new(0.5).mesh().uv(24,16)),ScenePrimitive::Capsule=>meshes.add(Capsule3d::new(0.35,0.8)),_=>meshes.add(Cuboid::new(1.0,1.0,1.0))};let material=materials.add(StandardMaterial{base_color:Color::srgb(0.8,0.4,0.2),perceptual_roughness:0.5,..default()});let visual=SceneVisual{primitive,..default()};let entity=commands.spawn((Mesh3d(mesh),MeshMaterial3d(material),transform,Name::new(name),Visibility::Inherited,EditorEntity,EditorParent(None),Pickable::default(),EditorPrimitive(primitive),EditorVisual(visual))).id();selection.select(entity);project.dirty=true;state.last_message=Some(format!("{name} created"));}

fn create_prefab(project:&ProjectState,selection:&SelectionState,transforms:&Query<(Entity,&Transform,Option<&Name>,Option<&EditorParent>,Option<&Visibility>,Option<&EditorPrimitive>,Option<&EditorVisual>),With<EditorEntity>>,names:&Query<(Entity,Option<&Name>,Option<&EditorParent>,Option<&Visibility>),With<EditorEntity>>,state:&mut CommandExecutionState){if selection.entities.is_empty(){state.last_error=Some("Select one or more entities before creating a prefab".into());return;}let selected:std::collections::BTreeSet<_>=selection.entities.iter().copied().collect();let mut snapshot=Vec::<(Entity,String,Transform,Option<Entity>,bool)>::new();for entity in &selection.entities{let Ok((_,name,parent,visibility))=names.get(*entity)else{continue};let Ok((_,transform,_,_,_,_,_))=transforms.get(*entity)else{continue};let parent=parent.and_then(|value|value.0).filter(|parent|selected.contains(parent));let visible=visibility.map(|value|!matches!(value,Visibility::Hidden)).unwrap_or(true);snapshot.push((*entity,name.map(|value|value.as_str().to_owned()).unwrap_or_else(||"Entity".into()),*transform,parent,visible));}if snapshot.is_empty(){state.last_error=Some("Selected entities are no longer available".into());return;}let base=project.name.trim().replace(' ','_');let mut path=prefab_path(&project.root,&format!("{base}_Prefab"));let mut suffix=1u32;while path.exists(){path=prefab_path(&project.root,&format!("{base}_Prefab_{suffix}"));suffix+=1;}let prefab_name=path.file_stem().and_then(|value|value.to_str()).unwrap_or("Prefab");let prefab=PrefabDocument::from_scene_entities(prefab_name,snapshot);match save_prefab(&path,&prefab){Ok(())=>state.last_message=Some(format!("Created prefab {}",path.display())),Err(error)=>state.last_error=Some(error.to_string())}}

pub fn execute_history_commands(mut events:EventReader<HistoryCommandEvent>,mut history:ResMut<crate::TransformHistory>,mut query:Query<&mut Transform,With<EditorEntity>>,mut state:ResMut<CommandExecutionState>,mut project:ResMut<ProjectState>){for event in events.read(){if event.undo{if history.can_undo(){history.undo(&mut query);state.last_message=Some("Undo applied".into());project.dirty=true}else{state.last_message=Some("Nothing to undo".into())}}else if history.can_redo(){history.redo(&mut query);state.last_message=Some("Redo applied".into());project.dirty=true}else{state.last_message=Some("Nothing to redo".into())}}}
