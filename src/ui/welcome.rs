use std::path::PathBuf;
use bevy::prelude::*;
use bevy_egui::egui;

use crate::{
    command::{EditorCommandBus, EditorCommandId},
    editor::EditorUiState,
    project::{create_project, load_project, ProjectState},
};

#[derive(Resource, Debug, Clone)]
pub struct WelcomeState { pub visible: bool, pub project_path: String, pub project_name: String }
impl Default for WelcomeState { fn default()->Self{Self{visible:true,project_path:".".into(),project_name:"My Game".into()}} }

pub fn show_welcome(ui:&mut egui::Ui,welcome:&mut WelcomeState,project:&mut ProjectState,editor:&mut EditorUiState,commands:&mut EditorCommandBus){
    let available=ui.available_size();
    egui::Frame::new().fill(egui::Color32::from_rgb(18,18,22)).show(ui,|ui|{
        ui.set_min_size(available);
        ui.vertical_centered(|ui|{
            ui.add_space(54.0);
            ui.heading(egui::RichText::new("Bevy-GUI").size(38.0).strong());
            ui.label(egui::RichText::new("Material 3 editor • Scene authoring • Runtime • Build").size(16.0));
            ui.add_space(28.0);
            ui.horizontal(|ui|{
                egui::Frame::group(ui.style()).fill(egui::Color32::from_rgb(31,31,38)).inner_margin(egui::Margin::same(20)).show(ui,|ui|{
                    ui.set_width(310.0);ui.heading("New Project");ui.label("Create a persistent project with manifest, assets and main scene.");ui.add_space(10.0);ui.label("Project name");ui.text_edit_singleline(&mut welcome.project_name);ui.label("Parent folder");ui.text_edit_singleline(&mut welcome.project_path);ui.add_space(10.0);
                    if ui.button("Create Project").clicked(){let parent=PathBuf::from(&welcome.project_path);let name=welcome.project_name.trim();if name.is_empty(){editor.status="Project name cannot be empty".into();}else{let root=parent.join(name);match create_project(&root,name){Ok(created)=>{*project=created;editor.status=format!("Created project at {}",project.root.display());welcome.visible=false;commands.emit(EditorCommandId("scene.open"));},Err(error)=>editor.status=format!("Create failed: {error}")}}}
                });
                egui::Frame::group(ui.style()).fill(egui::Color32::from_rgb(31,31,38)).inner_margin(egui::Margin::same(20)).show(ui,|ui|{
                    ui.set_width(310.0);ui.heading("Open Project");ui.label("Open a folder containing project.godot-rs.json.");ui.add_space(10.0);ui.text_edit_singleline(&mut welcome.project_path);
                    if ui.button("Open Project").clicked(){match load_project(PathBuf::from(&welcome.project_path).as_path()){Ok(loaded)=>{*project=loaded;editor.status=format!("Loaded {}",project.name);welcome.visible=false;commands.emit(EditorCommandId("scene.open"));},Err(error)=>editor.status=format!("Open failed: {error}")}}
                });
            });
            ui.add_space(24.0);
            egui::Frame::group(ui.style()).fill(egui::Color32::from_rgb(24,24,30)).inner_margin(egui::Margin::same(16)).show(ui,|ui|{ui.set_width(640.0);ui.heading("Editor workflow");ui.label("2D / 3D viewport • Hierarchy • Inspector • Assets • Console • Profiler • Engine Tools");ui.add_space(6.0);ui.small("Projects are written to disk immediately, so opening the same directory later restores the manifest and main scene.");});
            ui.add_space(20.0);ui.label(egui::RichText::new("Plugin-first • Bevy 0.19 • Vulkan / OpenGL / native windowing").weak());
        });
    });
}
