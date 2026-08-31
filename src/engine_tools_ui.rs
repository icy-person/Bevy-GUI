use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::{engine_features::{EngineDiagnostics, EngineEventMonitor, EngineFeatureRegistry}, project::ProjectState};

#[derive(Resource, Debug, Clone, Copy)]
pub struct EngineToolsUiState { pub visible: bool, pub tab: EngineToolsTab }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineToolsTab { Overview, Systems, Events, Query, State, Animation, Shaders, VisualScript }
impl Default for EngineToolsUiState { fn default() -> Self { Self { visible: true, tab: EngineToolsTab::Overview } } }

pub struct EngineToolsUiPlugin;
impl Plugin for EngineToolsUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EngineToolsUiState>()
            .add_systems(bevy_egui::EguiPrimaryContextPass, draw_engine_tools_ui);
    }
}

fn draw_engine_tools_ui(
    mut contexts: EguiContexts,
    mut state: ResMut<EngineToolsUiState>,
    project: Res<ProjectState>,
    features: Res<EngineFeatureRegistry>,
    diagnostics: Res<EngineDiagnostics>,
    mut events: ResMut<EngineEventMonitor>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    if !state.visible { return Ok(()); }

    egui::Window::new("Bevy-GUI Engine")
        .default_width(460.0)
        .default_height(520.0)
        .resizable(true)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Engine Tools");
                if ui.small_button("×").clicked() { state.visible = false; }
            });
            ui.separator();
            ui.horizontal_wrapped(|ui| {
                for (tab, label) in [
                    (EngineToolsTab::Overview, "Overview"),
                    (EngineToolsTab::Systems, "Systems"),
                    (EngineToolsTab::Events, "Events"),
                    (EngineToolsTab::Query, "Query"),
                    (EngineToolsTab::State, "State"),
                    (EngineToolsTab::Animation, "Animation"),
                    (EngineToolsTab::Shaders, "Shaders"),
                    (EngineToolsTab::VisualScript, "Visual Script"),
                ] {
                    if ui.selectable_label(state.tab == tab, label).clicked() { state.tab = tab; }
                }
            });
            ui.separator();
            match state.tab {
                EngineToolsTab::Overview => {
                    ui.label(format!("Project: {}", project.name));
                    ui.label(format!("Mode: {:?}", project.mode));
                    ui.label(format!("Frames: {}", diagnostics.frames));
                    ui.label(format!("Entities: {}", diagnostics.entities));
                    ui.label(format!("Drawables: {}", diagnostics.drawables));
                    ui.label(format!("Commands: {}", diagnostics.commands_executed));
                    ui.label(format!("Buffered events: {}", events.len()));
                    ui.separator();
                    ui.strong("Enabled features");
                    for feature in features.iter() { ui.label(format!("✓ {feature:?}")); }
                }
                EngineToolsTab::Events => {
                    ui.horizontal(|ui| {
                        ui.label(format!("{} buffered events", events.len()));
                        if ui.small_button("Clear").clicked() { events.clear(); }
                    });
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for event in events.iter().rev() {
                            ui.monospace(format!("#{:04} frame {:>7} {:<20} {}", event.sequence, event.frame, event.kind, event.payload));
                        }
                    });
                }
                EngineToolsTab::Systems => {
                    ui.label("System graph is available through the engine diagnostics registry.");
                    ui.label("Startup → Update → PostUpdate");
                }
                EngineToolsTab::Query => {
                    ui.label("Select an entity in the hierarchy to inspect its ECS components.");
                }
                EngineToolsTab::State => {
                    ui.label(format!("Editor mode: {:?}", project.mode));
                }
                EngineToolsTab::Animation => ui.label("Animation runtime is enabled."),
                EngineToolsTab::Shaders => ui.label("Shader graph runtime is enabled."),
                EngineToolsTab::VisualScript => ui.label("Visual scripting runtime is enabled."),
            }
        });
    Ok(())
}
