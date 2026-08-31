use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::{
    animation::AnimationLibrary,
    component_registry::ComponentRegistry,
    editor::EditorUiState,
    engine_features::{EngineDiagnostics, EngineEventMonitor, EngineFeature, EngineFeatureRegistry, EngineGraphRegistry},
    project::ProjectState,
    selection::SelectionState,
    shader_graph::ShaderGraphLibrary,
    visual_scripting::VisualScriptAsset,
};

#[derive(Resource, Debug, Clone, Copy)]
pub struct EngineToolsUiState { pub visible: bool, pub tab: EngineToolsTab }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineToolsTab { Overview, Systems, Events, Query, State, Animation, Shaders, VisualScript }
impl Default for EngineToolsUiState { fn default()->Self{Self{visible:true,tab:EngineToolsTab::Overview}} }

pub struct EngineToolsUiPlugin;
impl Plugin for EngineToolsUiPlugin{fn build(&self,app:&mut App){app.init_resource::<EngineToolsUiState>().add_systems(bevy_egui::EguiPrimaryContextPass,draw_engine_tools_ui);}}

fn draw_engine_tools_ui(mut contexts:EguiContexts,mut ui_state:ResMut<EngineToolsUiState>,editor_state:Res<EditorUiState>,project:Res<ProjectState>,selection:Res<SelectionState>,features:Res<EngineFeatureRegistry>,graph:Res<EngineGraphRegistry>,events:Res<EngineEventMonitor>,diagnostics:Res<EngineDiagnostics>,registry:Res<ComponentRegistry>,animations:Query<(Entity,&AnimationLibrary)>,shaders:Res<ShaderGraphLibrary>,scripts:Query<(Entity,&VisualScriptAsset)>)->Result{
    let ctx=contexts.ctx_mut()?;
    if !ui_state.visible{return Ok(())}
    egui::Window::new("Bevy-GUI Engine").default_width(420.0).default_height(520.0).resizable(true).show(ctx,|ui|{
        ui.horizontal(|ui|{ui.heading("Engine Tools");ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{if ui.small_button("×").clicked(){ui_state.visible=false;}});});
        ui.horizontal_wrapped(|ui|{for(tab,label)in[(EngineToolsTab::Overview,"Overview"),(EngineToolsTab::Systems,"Systems"),(EngineToolsTab::Events,"Events"),(EngineToolsTab::Query,"Query"),(EngineToolsTab::State,"State"),(EngineToolsTab::Animation,"Animation"),(EngineToolsTab::Shaders,"Shaders"),(EngineToolsTab::VisualScript,"Visual Script")]{if ui.selectable_label(ui_state.tab==tab,label).clicked(){ui_state.tab=tab;}}});
        ui.separator();
        match ui_state.tab{EngineToolsTab::Overview=>overview(ui,project,selection,features,diagnostics,events,&editor_state),EngineToolsTab::Systems=>systems(ui,graph),EngineToolsTab::Events=>event_monitor(ui,events),EngineToolsTab::Query=>query_visualizer(ui,selection,&registry),EngineToolsTab::State=>state_editor(ui,project,&editor_state),EngineToolsTab::Animation=>animation_panel(ui,&animations),EngineToolsTab::Shaders=>shader_panel(ui,&shaders),EngineToolsTab::VisualScript=>visual_script_panel(ui,&scripts)}
    });Ok(())
}
fn overview(ui:&mut egui::Ui,project:&ProjectState,selection:&SelectionState,features:&EngineFeatureRegistry,diagnostics:&EngineDiagnostics,events:&EngineEventMonitor,editor_state:&EditorUiState){ui.label(format!("Project: {}",project.name));ui.label(format!("Mode: {:?}",project.mode));ui.label(format!("Selected: {}",selection.entities.len()));ui.separator();ui.label(format!("Frames: {}",diagnostics.frames));ui.label(format!("Entities: {}",diagnostics.entities));ui.label(format!("Drawables: {}",diagnostics.drawables));ui.label(format!("Commands: {}",diagnostics.commands_executed));ui.label(format!("Events: {}",events.len()));ui.separator();ui.label(format!("Viewport: {:?}",editor_state.viewport_mode));ui.label(format!("Gizmo: {:?}",editor_state.gizmo_mode));ui.separator();ui.strong("Capabilities");for feature in features.iter(){ui.label(format!("✓ {feature:?}"));}}
fn systems(ui:&mut egui::Ui,graph:&EngineGraphRegistry){ui.label(format!("Registered systems: {}",graph.iter().count()));egui::ScrollArea::vertical().show(ui,|ui|{for system in graph.iter(){ui.collapsing(&system.name,|ui|{ui.label(format!("Schedule: {}",system.schedule));if !system.reads.is_empty(){ui.label(format!("Reads: {}",system.reads.join(", ")));}if !system.writes.is_empty(){ui.label(format!("Writes: {}",system.writes.join(", ")));}if !system.after.is_empty(){ui.label(format!("After: {}",system.after.join(", ")));}if !system.before.is_empty(){ui.label(format!("Before: {}",system.before.join(", ")));}})}})}
fn event_monitor(ui:&mut egui::Ui,events:&EngineEventMonitor){ui.label(format!("{} buffered events",events.len()));egui::ScrollArea::vertical().stick_to_bottom(true).show(ui,|ui|{for event in events.iter().rev(){ui.monospace(format!("#{:04}  frame {:>7}  {:<20} {}",event.sequence,event.frame,event.kind,event.payload));}})}
fn query_visualizer(ui:&mut egui::Ui,selection:&SelectionState,registry:&ComponentRegistry){let Some(entity)=selection.primary()else{ui.label("Select an entity to inspect its ECS composition.");return};ui.monospace(format!("Entity {:?}",entity));ui.separator();for descriptor in registry.iter(){ui.label(format!("{} · {}",descriptor.category,descriptor.label));}}
fn state_editor(ui:&mut egui::Ui,project:&ProjectState,editor_state:&EditorUiState){ui.heading("Runtime / Editor State");ui.label(format!("Project: {:?}",project.mode));ui.label(format!("Viewport: {:?}",editor_state.viewport_mode));ui.label(format!("Gizmo: {:?}",editor_state.gizmo_mode));ui.label(format!("Transform space: {:?}",editor_state.transform_space));}
fn animation_panel(ui:&mut egui::Ui,animations:&Query<(Entity,&AnimationLibrary)>){ui.label(format!("Animation libraries: {}",animations.iter().count()));egui::ScrollArea::vertical().show(ui,|ui|{for(entity,library)in animations.iter(){ui.collapsing(format!("Entity {:?}",entity),|ui|{for(name,clip)in &library.0{ui.label(format!("{} · {:.2}s · {} tracks · {}",name,clip.duration,clip.tracks.len(),if clip.looping{"loop"}else{"once"}));}})}})}
fn shader_panel(ui:&mut egui::Ui,shaders:&ShaderGraphLibrary){ui.label(format!("Shader graphs: {}",shaders.graphs.len()));egui::ScrollArea::vertical().show(ui,|ui|{for(name,graph)in &shaders.graphs{ui.collapsing(name,|ui|{ui.label(format!("{} nodes · {} links",graph.nodes.len(),graph.links.len()));for issue in graph.validate(){ui.colored_label(egui::Color32::YELLOW,issue);}})}})}
fn visual_script_panel(ui:&mut egui::Ui,scripts:&Query<(Entity,&VisualScriptAsset)>){ui.label(format!("Visual scripts: {}",scripts.iter().count()));egui::ScrollArea::vertical().show(ui,|ui|{for(entity,script)in scripts.iter(){ui.collapsing(format!("Entity {:?}",entity),|ui|{ui.label(format!("{} nodes · {} links",script.0.nodes.len(),script.0.links.len()));for issue in script.0.validate(){ui.colored_label(egui::Color32::YELLOW,issue);}})}})}

#[allow(dead_code)]fn feature_enabled(features:&EngineFeatureRegistry,feature:EngineFeature)->bool{features.is_enabled(feature)}
