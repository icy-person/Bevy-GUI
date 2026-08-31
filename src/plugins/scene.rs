use bevy::prelude::*;
use bevy_egui::egui;
use crate::{command::{EditorCommandBus, EditorCommandId}, editor::{EditorPlugin, EditorPluginRegistry}, panel::PanelRegistry, project::{EditorMode, ProjectState}, scene_model::SceneEditorState, scene_tools::{validate_scene, SceneValidationReport}};

pub struct SceneEditorPlugin;
impl Default for SceneEditorPlugin { fn default() -> Self { Self } }
impl EditorPlugin for SceneEditorPlugin {
    fn name(&self) -> &'static str { "scene-editor" }
    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<EditorPluginRegistry>().register(self.name(), "1.3");
        app.world_mut().resource_mut::<PanelRegistry>().register(crate::panel::PanelId("scene"), "Scene", scene_panel);
    }
}

fn scene_panel(world: &mut World, ui: &mut egui::Ui) {
    let project = world.get_resource::<ProjectState>().cloned();
    let editor = world.get_resource::<SceneEditorState>().cloned();
    let Some(project) = project else { ui.label("Project state is unavailable."); return; };
    ui.horizontal(|ui| { ui.strong("Scene"); ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| mode_badge(ui, project.mode)); });
    ui.separator();
    ui.label(egui::RichText::new(&project.name).size(16.0).strong());
    ui.small(format!("Root: {}", project.root.display()));
    egui::CollapsingHeader::new("Main Scene").default_open(true).show(ui, |ui| {
        ui.monospace(project.main_scene.as_ref().map(|path| path.display().to_string()).unwrap_or_else(|| "<not configured>".into()));
        ui.horizontal_wrapped(|ui| { if ui.button("Open").clicked(){emit(world,"scene.open");} if ui.button("Save").clicked(){emit(world,"scene.save");} if ui.button("Validate").clicked(){emit(world,"scene.validate");} });
    });
    egui::CollapsingHeader::new("Create").default_open(true).show(ui, |ui| {
        ui.horizontal_wrapped(|ui| { for (label,command) in [("Entity","scene.new_entity"),("Cube","scene.new_cube"),("Plane","scene.new_plane"),("Sphere","scene.new_sphere"),("Capsule","scene.new_capsule")] { if ui.button(label).clicked(){emit(world,command);} } });
    });
    egui::CollapsingHeader::new("Selection").default_open(true).show(ui, |ui| { if ui.button("Duplicate Selected").clicked(){emit(world,"scene.duplicate");} if ui.button("Delete Selected").clicked(){emit(world,"scene.delete");} if ui.button("Create Prefab").clicked(){emit(world,"scene.prefab_create");} });
    if let Some(state)=editor { egui::CollapsingHeader::new("Authoring State").default_open(true).show(ui, |ui| { stat_row(ui,"Revision",state.revision.to_string()); stat_row(ui,"Saved",state.saved_revision.to_string()); stat_row(ui,"State",if state.dirty(){"Modified".into()}else{"Saved".into()}); stat_row(ui,"Loaded path",state.path.as_ref().map(|p|p.display().to_string()).unwrap_or_else(||"<none>".into())); }); }
    let report=current_validation(world); draw_validation(ui,report.as_ref());
}
fn current_validation(world:&mut World)->Option<SceneValidationReport>{let project=world.get_resource::<ProjectState>()?;let main_scene=project.main_scene.as_ref()?;let document=crate::scene::load_scene(&project.root.join(main_scene)).ok()?;Some(validate_scene(&document))}
fn draw_validation(ui:&mut egui::Ui,report:Option<&SceneValidationReport>){egui::CollapsingHeader::new("Validation").default_open(false).show(ui,|ui|{match report{Some(report) if report.is_valid()=>{ui.colored_label(egui::Color32::from_rgb(125,220,160),"Scene valid");stat_row(ui,"Warnings",report.warnings().to_string());},Some(report)=>{ui.colored_label(egui::Color32::from_rgb(255,125,125),"Scene has validation errors");stat_row(ui,"Errors",report.errors().to_string());stat_row(ui,"Warnings",report.warnings().to_string());for issue in report.issues.iter().take(12){ui.label(&issue.message);}},None=>{ui.label("No scene is currently available for validation.");}}});}
fn mode_badge(ui:&mut egui::Ui,mode:EditorMode){let(label,color)=match mode{EditorMode::Edit=>("EDIT",egui::Color32::from_rgb(150,195,255)),EditorMode::Play=>("PLAY",egui::Color32::from_rgb(125,220,160)),EditorMode::Paused=>("PAUSED",egui::Color32::from_rgb(255,190,96))};ui.label(egui::RichText::new(label).small().strong().color(color));}
fn stat_row(ui:&mut egui::Ui,label:&str,value:String){ui.horizontal(|ui|{ui.weak(label);ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{ui.monospace(value);});});}
fn emit(world:&mut World,id:&'static str){if let Some(mut bus)=world.get_resource_mut::<EditorCommandBus>(){bus.emit(EditorCommandId(id));}}
