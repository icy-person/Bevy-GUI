use bevy::prelude::*;
use bevy_egui::egui;
use egui_dock::{DockArea, TabViewer};

use crate::{
    editor::{EditorUiState, ViewportMode},
    profiler::EditorProfiler,
    project::{EditorMode, ProjectState},
    selection::SelectionState,
    settings::EditorSettingsState,
};

use super::state::{EditorDockState, EditorTab, TransformEdit};

pub struct DockViewer<'a>{
    pub project:&'a mut ProjectState,
    pub selection:&'a mut SelectionState,
    pub ui_state:&'a mut EditorUiState,
    pub settings:&'a mut EditorSettingsState,
    pub profiler:&'a EditorProfiler,
    pub assets:&'a mut crate::AssetDatabase,
    pub entities:&'a[(Entity,String)],
    pub parents:&'a[(Entity,Option<Entity>)],
    pub selected_transform:Option<TransformEdit>,
    pub selected_name:Option<String>,
    pub selected_visible:Option<bool>,
    pub plugin_names:&'a[String],
    pub command_count:usize,
    pub transform_edit:Option<TransformEdit>,
    pub name_edit:Option<String>,
    pub visibility_edit:Option<bool>,
    pub viewport_focused:bool,
    pub create_entity:bool,
    pub delete_entity:Option<Entity>,
    pub duplicate_entity:Option<Entity>,
    pub save_requested:bool,
    pub parent_selected:bool,
    pub unparent_selected:bool,
}

impl TabViewer for DockViewer<'_>{type Tab=EditorTab;fn title(&mut self,tab:&mut Self::Tab)->egui::WidgetText{tab.title().into()}fn ui(&mut self,ui:&mut egui::Ui,tab:&mut Self::Tab){match *tab{EditorTab::Viewport=>self.show_viewport(ui),EditorTab::Hierarchy=>self.show_hierarchy(ui),EditorTab::Inspector=>self.show_inspector(ui),EditorTab::Assets=>self.show_assets(ui),EditorTab::Console=>self.show_console(ui),EditorTab::Profiler=>self.show_profiler(ui),EditorTab::Plugins=>self.show_plugins(ui),EditorTab::Settings=>crate::ui::settings::show_settings(ui,self.settings,self.project)}}fn clear_background(&self,tab:&Self::Tab)->bool{!matches!(tab,Self::Tab::Viewport)}fn scroll_bars(&self,tab:&Self::Tab)->[bool;2]{if matches!(tab,Self::Tab::Viewport){[false,false]}else{[true,true]}}fn is_closeable(&self,tab:&Self::Tab)->bool{!matches!(tab,Self::Tab::Viewport)}}

impl DockViewer<'_>{
fn section(ui:&mut egui::Ui,title:&str,body:impl FnOnce(&mut egui::Ui)){egui::Frame::group(ui.style()).inner_margin(egui::Margin::same(10)).show(ui,|ui|{ui.label(egui::RichText::new(title).strong());ui.add_space(5.0);body(ui);});}
fn show_viewport(&mut self,ui:&mut egui::Ui){self.viewport_focused=true;ui.horizontal(|ui|{ui.label("Scene View");ui.separator();if ui.selectable_label(self.ui_state.viewport_mode==ViewportMode::TwoD,"2D").clicked(){self.ui_state.viewport_mode=ViewportMode::TwoD;}if ui.selectable_label(self.ui_state.viewport_mode==ViewportMode::ThreeD,"3D").clicked(){self.ui_state.viewport_mode=ViewportMode::ThreeD;}ui.separator();if ui.selectable_label(self.project.mode==EditorMode::Edit,"Edit").clicked(){self.project.mode=EditorMode::Edit;}if ui.selectable_label(self.project.mode==EditorMode::Play,"Play").clicked(){self.project.mode=EditorMode::Play;}if ui.selectable_label(self.project.mode==EditorMode::Paused,"Pause").clicked(){self.project.mode=EditorMode::Paused;}});ui.separator();ui.centered_and_justified(|ui|{ui.vertical_centered(|ui|{ui.heading(if self.ui_state.viewport_mode==ViewportMode::TwoD{"2D Viewport"}else{"3D Viewport"});ui.label("Viewport rendering is provided by the live Bevy world behind the UI.");ui.label(format!("{} entities",self.entities.len()));});});}
fn show_hierarchy(&mut self,ui:&mut egui::Ui){ui.label("Scene Hierarchy");ui.horizontal(|ui|{if ui.button("＋ Entity").clicked(){self.create_entity=true;}if ui.button("Duplicate").clicked(){self.duplicate_entity=self.selection.primary();}if ui.button("Delete").clicked(){self.delete_entity=self.selection.primary();}});ui.separator();for(entity,name)in self.entities{let selected=self.selection.contains(*entity);let response=ui.selectable_label(selected,name);if response.clicked(){self.selection.select(*entity);}}}
pub fn show_inspector(&mut self,ui:&mut egui::Ui){ui.label("Inspector");ui.separator();let Some(mut edit)=self.selected_transform else{ui.centered_and_justified(|ui|{ui.weak("Select an entity");});return;};Self::section(ui,"Identity",|ui|{ui.label(format!("Entity {:?}",edit.entity));if let Some(name)=&mut self.selected_name{if ui.text_edit_singleline(name).lost_focus() {self.name_edit=Some(name.clone());}}});Self::section(ui,"Visibility",|ui|{let mut value=self.selected_visible.unwrap_or(true);if ui.checkbox(&mut value,"Visible").changed(){self.visibility_edit=Some(value);}});Self::section(ui,"Transform",|ui|{ui.label("Position");ui.horizontal(|ui|{ui.add(egui::DragValue::new(&mut edit.translation.x).speed(0.05));ui.add(egui::DragValue::new(&mut edit.translation.y).speed(0.05));ui.add(egui::DragValue::new(&mut edit.translation.z).speed(0.05));});ui.label("Rotation");ui.horizontal(|ui|{ui.add(egui::DragValue::new(&mut edit.rotation.x).speed(0.5));ui.add(egui::DragValue::new(&mut edit.rotation.y).speed(0.5));ui.add(egui::DragValue::new(&mut edit.rotation.z).speed(0.5));});ui.label("Scale");ui.horizontal(|ui|{ui.add(egui::DragValue::new(&mut edit.scale.x).speed(0.05));ui.add(egui::DragValue::new(&mut edit.scale.y).speed(0.05));ui.add(egui::DragValue::new(&mut edit.scale.z).speed(0.05));});if ui.button("Apply Transform").clicked(){self.transform_edit=Some(edit);}});Self::section(ui,"Components",|ui|{ui.label("Transform");ui.label("Name");ui.label("Visibility");ui.label("EditorEntity");});}
fn show_assets(&mut self,ui:&mut egui::Ui){ui.label("Asset Browser");ui.label(format!("{} assets",self.assets.entries.len()));for entry in self.assets.entries.iter().take(300){ui.label(entry.path.display().to_string());}}
fn show_console(&mut self,ui:&mut egui::Ui){ui.label("Console");ui.monospace("[editor] workspace online");ui.monospace(format!("[editor] commands {}",self.command_count));ui.monospace(format!("[editor] plugins {}",self.plugin_names.len()));ui.monospace(format!("[editor] selected {}",self.selection.entities.len()));}
fn show_profiler(&mut self,ui:&mut egui::Ui){ui.label("Profiler");ui.horizontal(|ui|{ui.label(format!("FPS {:.1}",self.profiler.fps));ui.label(format!("Frame {:.2} ms",self.profiler.frame_time_ms));});ui.label(format!("Min {:.2} ms",self.profiler.min_frame_ms));ui.label(format!("Max {:.2} ms",self.profiler.max_frame_ms));}
fn show_plugins(&mut self,ui:&mut egui::Ui){ui.label("Plugins");for plugin in self.plugin_names{ui.horizontal(|ui|{ui.label("✦");ui.label(plugin);});}}
}

pub fn show_dock_area(ui:&mut egui::Ui,dock:&mut EditorDockState,viewer:&mut DockViewer<'_>){DockArea::new(&mut dock.state).show_add_buttons(true).show_add_popup(true).show_close_buttons(true).show_inside(ui,viewer);}
