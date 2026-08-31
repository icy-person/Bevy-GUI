use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::{
    animation::{AnimationClip, AnimationLibrary, AnimationTrack, KeyValue},
    component_registry::ComponentRegistry,
    editor::EditorUiState,
    engine_features::{EngineDiagnostics, EngineEventMonitor, EngineFeature, EngineFeatureRegistry, EngineGraphRegistry},
    history::TransformHistory,
    project::{EditorMode, ProjectState},
    selection::SelectionState,
    shader_graph::{ShaderGraph, ShaderGraphLibrary, ShaderNodeKind},
    visual_scripting::{VisualNodeKind, VisualScript, VisualScriptAsset, VisualScriptRuntime},
    viewport::EditorEntity,
};

#[derive(Resource,Debug,Clone,Copy)]
pub struct EngineToolsUiState{pub visible:bool,pub tab:EngineToolsTab}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum EngineToolsTab{Overview,Systems,Events,Query,State,Animation,Shaders,VisualScript}
impl Default for EngineToolsUiState{fn default()->Self{Self{visible:true,tab:EngineToolsTab::Overview}}}

pub struct EngineToolsUiPlugin;
impl Plugin for EngineToolsUiPlugin{fn build(&self,app:&mut App){app.init_resource::<EngineToolsUiState>().add_systems(bevy_egui::EguiPrimaryContextPass,draw_engine_tools_ui);}}

#[derive(SystemParam)]
struct ToolEntities<'w,'s>{
    all:Query<'w,'s,(Entity,Option<&'static Transform>,Option<&'static Name>,Option<&'static Visibility>,Option<&'static Camera3d>,Option<&'static Mesh3d>,Option<&'static MeshMaterial3d<StandardMaterial>>,Option<&'static avian3d::prelude::RigidBody>,Option<&'static avian3d::prelude::Collider>,Option<&'static VisualScriptAsset>,Option<&'static crate::EditorPrimitive>,Option<&'static crate::EditorVisual>),With<EditorEntity>>,
    scripts:Query<'w,'s,(Entity,&'static VisualScriptAsset),With<EditorEntity>>,
}

fn draw_engine_tools_ui(mut contexts:EguiContexts,mut ui_state:ResMut<EngineToolsUiState>,editor_state:Res<EditorUiState>,mut project:ResMut<ProjectState>,selection:Res<SelectionState>,features:Res<EngineFeatureRegistry>,graph:Res<EngineGraphRegistry>,mut events:ResMut<EngineEventMonitor>,diagnostics:Res<EngineDiagnostics>,registry:Res<ComponentRegistry>,mut animations:Query<(Entity,&mut AnimationLibrary),With<EditorEntity>>,mut shaders:ResMut<ShaderGraphLibrary>,mut commands:Commands,entities:ToolEntities,mut history:ResMut<TransformHistory>)->Result{
    let ctx=contexts.ctx_mut()?;
    if !ui_state.visible{return Ok(())}
    egui::Window::new("Bevy-GUI Engine").default_width(480.0).default_height(560.0).resizable(true).show(ctx,|ui|{
        ui.horizontal(|ui|{ui.heading("Engine Tools");ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{if ui.small_button("×").clicked(){ui_state.visible=false;}});});
        ui.horizontal_wrapped(|ui|{for(tab,label)in[(EngineToolsTab::Overview,"Overview"),(EngineToolsTab::Systems,"Systems"),(EngineToolsTab::Events,"Events"),(EngineToolsTab::Query,"Query"),(EngineToolsTab::State,"State"),(EngineToolsTab::Animation,"Animation"),(EngineToolsTab::Shaders,"Shaders"),(EngineToolsTab::VisualScript,"Visual Script")]{if ui.selectable_label(ui_state.tab==tab,label).clicked(){ui_state.tab=tab;}}});
        ui.separator();
        match ui_state.tab{
            EngineToolsTab::Overview=>overview(ui,&project,&selection,&features,&diagnostics,&events,&editor_state,&history),
            EngineToolsTab::Systems=>systems(ui,&graph),
            EngineToolsTab::Events=>event_monitor(ui,&mut events),
            EngineToolsTab::Query=>query_visualizer(ui,&selection,&registry,&entities.all),
            EngineToolsTab::State=>state_editor(ui,&project,&editor_state),
            EngineToolsTab::Animation=>animation_panel(ui,&selection,&mut animations,&mut commands,&mut project),
            EngineToolsTab::Shaders=>shader_panel(ui,&mut shaders),
            EngineToolsTab::VisualScript=>visual_script_panel(ui,&selection,&mut commands,&entities.scripts,&mut project),
        }
    });
    Ok(())
}

fn overview(ui:&mut egui::Ui,project:&ProjectState,selection:&SelectionState,features:&EngineFeatureRegistry,diagnostics:&EngineDiagnostics,events:&EngineEventMonitor,editor_state:&EditorUiState,history:&TransformHistory){ui.label(format!("Project: {}",project.name));ui.label(format!("Mode: {:?}",project.mode));ui.label(format!("Selected: {}",selection.entities.len()));ui.separator();ui.label(format!("Frames: {}",diagnostics.frames));ui.label(format!("Entities: {}",diagnostics.entities));ui.label(format!("Drawables: {}",diagnostics.drawables));ui.label(format!("Commands: {}",diagnostics.commands_executed));ui.label(format!("Events: {}",events.len()));ui.label(format!("Undo: {} · Redo: {}",history.undo_len(),history.redo_len()));ui.separator();ui.label(format!("Viewport: {:?}",editor_state.viewport_mode));ui.label(format!("Gizmo: {:?}",editor_state.gizmo_mode));ui.separator();ui.strong("Capabilities");for feature in features.iter(){ui.label(format!("✓ {feature:?}"));}}
fn systems(ui:&mut egui::Ui,graph:&EngineGraphRegistry){ui.label(format!("Registered systems: {}",graph.iter().count()));egui::ScrollArea::vertical().show(ui,|ui|{for system in graph.iter(){ui.collapsing(&system.name,|ui|{ui.label(format!("Schedule: {}",system.schedule));if !system.reads.is_empty(){ui.label(format!("Reads: {}",system.reads.join(", ")));}if !system.writes.is_empty(){ui.label(format!("Writes: {}",system.writes.join(", ")));}if !system.after.is_empty(){ui.label(format!("After: {}",system.after.join(", ")));}})}})}
fn event_monitor(ui:&mut egui::Ui,events:&mut EngineEventMonitor){ui.horizontal(|ui|{ui.label(format!("{} buffered events",events.len()));if ui.small_button("Clear").clicked(){events.clear();}});egui::ScrollArea::vertical().stick_to_bottom(true).show(ui,|ui|{for event in events.iter().rev(){ui.monospace(format!("#{:04} frame {:>7} {:<20} {}",event.sequence,event.frame,event.kind,event.payload));}})}

fn query_visualizer(ui:&mut egui::Ui,selection:&SelectionState,registry:&ComponentRegistry,entities:&Query<(Entity,Option<&Transform>,Option<&Name>,Option<&Visibility>,Option<&Camera3d>,Option<&Mesh3d>,Option<&MeshMaterial3d<StandardMaterial>>,Option<&avian3d::prelude::RigidBody>,Option<&avian3d::prelude::Collider>,Option<&VisualScriptAsset>,Option<&crate::EditorPrimitive>,Option<&crate::EditorVisual>),With<EditorEntity>>){let Some(entity)=selection.primary()else{ui.label("Select an entity to inspect its ECS composition.");return};let Ok((_,transform,name,visibility,camera,mesh,material,body,collider,script,primitive,visual))=entities.get(entity)else{ui.colored_label(egui::Color32::RED,"Selected entity is no longer alive.");return};ui.monospace(format!("Entity {:?}",entity));ui.separator();let actual=[("Transform",transform.is_some()),("Name",name.is_some()),("Visibility",visibility.is_some()),("Camera3d",camera.is_some()),("Mesh3d",mesh.is_some()),("Material",material.is_some()),("RigidBody",body.is_some()),("Collider",collider.is_some()),("Visual Script",script.is_some()),("Editor Primitive",primitive.is_some()),("Editor Visual",visual.is_some())];ui.strong("Actual components");for(label,present)in actual{ui.label(format!("{} {}",if present{"✓"}else{"○"},label));}ui.separator();ui.strong(format!("Registry: {} components / {} properties",registry.component_count(),registry.property_count()));}
fn state_editor(ui:&mut egui::Ui,project:&ProjectState,editor_state:&EditorUiState){ui.heading("Runtime / Editor State");ui.label(format!("Project: {:?}",project.mode));ui.label(format!("Viewport: {:?}",editor_state.viewport_mode));ui.label(format!("Gizmo: {:?}",editor_state.gizmo_mode));ui.label(format!("Transform space: {:?}",editor_state.transform_space));if project.mode==EditorMode::Play{ui.colored_label(egui::Color32::from_rgb(125,220,160),"Play-In-Editor is running");}}

fn animation_panel(ui:&mut egui::Ui,selection:&SelectionState,animations:&mut Query<(Entity,&mut AnimationLibrary),With<EditorEntity>>,commands:&mut Commands,project:&mut ProjectState){ui.label(format!("Animation libraries: {}",animations.iter().count()));let Some(selected)=selection.primary()else{ui.label("Select an entity to add or inspect animation.");return};let Ok((entity,mut library))=animations.get_mut(selected)else{if ui.button("Add Animation Library").clicked(){commands.entity(selected).insert(AnimationLibrary::default());project.dirty=true;}ui.label("Selected entity has no AnimationLibrary component yet.");return};if ui.button("Add Demo Move Clip").clicked(){let clip=AnimationClip{name:"DemoMove".into(),duration:2.0,looping:true,tracks:vec![AnimationTrack{property:"translation".into(),keys:vec![crate::animation::Keyframe{time:0.0,value:KeyValue::Vec3([0.0,0.0,0.0])},crate::animation::Keyframe{time:1.0,value:KeyValue::Vec3([2.0,0.0,0.0])},crate::animation::Keyframe{time:2.0,value:KeyValue::Vec3([0.0,0.0,0.0])}]}]};library.0.insert(clip.name.clone(),clip);project.dirty=true;}for(name,clip)in &library.0{ui.collapsing(format!("{} · Entity {:?}",name,entity),|ui|{ui.label(format!("{:.2}s · {} tracks · {}",clip.duration,clip.tracks.len(),if clip.looping{"loop"}else{"once"}));})}}
fn shader_panel(ui:&mut egui::Ui,shaders:&mut ShaderGraphLibrary){ui.horizontal(|ui|{ui.label(format!("Shader graphs: {}",shaders.graphs.len()));if ui.button("New Basic Graph").clicked(){let mut graph=ShaderGraph::new();let value=graph.add_node("Color",ShaderNodeKind::Constant([1.0,0.2,0.05,1.0]),[-160.0,0.0]);let output=graph.add_node("Output",ShaderNodeKind::Output,[80.0,0.0]);let _=graph.connect(value,"value",output,"color");let mut index=1usize;let mut name="Graph".to_owned();while shaders.graphs.contains_key(&name){index+=1;name=format!("Graph{index}");}shaders.insert(name,graph);}});egui::ScrollArea::vertical().show(ui,|ui|{for(name,graph)in &shaders.graphs{ui.collapsing(name,|ui|{ui.label(format!("{} nodes · {} links",graph.nodes.len(),graph.links.len()));for issue in graph.validate(){ui.colored_label(egui::Color32::YELLOW,issue);}})}})}
fn visual_script_panel(ui:&mut egui::Ui,selection:&SelectionState,commands:&mut Commands,scripts:&Query<(Entity,&VisualScriptAsset),With<EditorEntity>>,project:&mut ProjectState){ui.horizontal(|ui|{ui.label(format!("Visual scripts: {}",scripts.iter().count()));if ui.button("Attach Demo Script").clicked(){if let Some(entity)=selection.primary(){let mut script=VisualScript::new();let start=script.add_node("Start",VisualNodeKind::Start,[0.0,0.0]);let move_node=script.add_node("Move",VisualNodeKind::SetPosition([0.0,1.0,0.0]),[160.0,0.0]);let end=script.add_node("End",VisualNodeKind::End,[320.0,0.0]);let _=script.connect(start,"exec",move_node,"exec");let _=script.connect(move_node,"exec",end,"exec");commands.entity(entity).insert((VisualScriptAsset(script),VisualScriptRuntime::default()));project.dirty=true;}}});for(entity,script)in scripts.iter(){ui.collapsing(format!("Entity {:?}",entity),|ui|{ui.label(format!("{} nodes · {} links",script.0.nodes.len(),script.0.links.len()));for issue in script.0.validate(){ui.colored_label(egui::Color32::YELLOW,issue);}})}}

#[allow(dead_code)]fn feature_enabled(features:&EngineFeatureRegistry,feature:EngineFeature)->bool{features.is_enabled(feature)}
