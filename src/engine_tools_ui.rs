use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::{
    component_registry::ComponentRegistry,
    editor::EditorUiState,
    engine_features::{EngineDiagnostics, EngineEventMonitor, EngineFeature, EngineFeatureRegistry, EngineGraphRegistry},
    project::ProjectState,
    selection::SelectionState,
};

#[derive(Resource, Debug, Clone, Copy)]
pub struct EngineToolsUiState {
    pub visible: bool,
    pub tab: EngineToolsTab,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineToolsTab {
    Overview,
    Systems,
    Events,
    Query,
    State,
}

impl Default for EngineToolsUiState {
    fn default() -> Self {
        Self { visible: true, tab: EngineToolsTab::Overview }
    }
}

pub struct EngineToolsUiPlugin;

impl Plugin for EngineToolsUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EngineToolsUiState>()
            .add_systems(bevy_egui::EguiPrimaryContextPass, draw_engine_tools_ui);
    }
}

fn draw_engine_tools_ui(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<EngineToolsUiState>,
    editor_state: Res<EditorUiState>,
    project: Res<ProjectState>,
    selection: Res<SelectionState>,
    features: Res<EngineFeatureRegistry>,
    graph: Res<EngineGraphRegistry>,
    events: Res<EngineEventMonitor>,
    diagnostics: Res<EngineDiagnostics>,
    registry: Res<ComponentRegistry>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    if !ui_state.visible {
        return Ok(());
    }

    egui::Window::new("Bevy-GUI Engine")
        .default_width(360.0)
        .default_height(460.0)
        .resizable(true)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Engine Tools");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("×").clicked() {
                        ui_state.visible = false;
                    }
                });
            });

            ui.horizontal_wrapped(|ui| {
                for (tab, label) in [
                    (EngineToolsTab::Overview, "Overview"),
                    (EngineToolsTab::Systems, "Systems"),
                    (EngineToolsTab::Events, "Events"),
                    (EngineToolsTab::Query, "Query"),
                    (EngineToolsTab::State, "State"),
                ] {
                    if ui.selectable_label(ui_state.tab == tab, label).clicked() {
                        ui_state.tab = tab;
                    }
                }
            });
            ui.separator();

            match ui_state.tab {
                EngineToolsTab::Overview => overview(ui, project, selection, features, diagnostics, events, &editor_state),
                EngineToolsTab::Systems => systems(ui, graph),
                EngineToolsTab::Events => event_monitor(ui, events),
                EngineToolsTab::Query => query_visualizer(ui, selection, &registry),
                EngineToolsTab::State => state_editor(ui, project, &editor_state),
            }
        });

    Ok(())
}

fn overview(
    ui: &mut egui::Ui,
    project: &ProjectState,
    selection: &SelectionState,
    features: &EngineFeatureRegistry,
    diagnostics: &EngineDiagnostics,
    events: &EngineEventMonitor,
    editor_state: &EditorUiState,
) {
    ui.label(format!("Project: {}", project.name));
    ui.label(format!("Mode: {:?}", project.mode));
    ui.label(format!("Selected: {}", selection.entities.len()));
    ui.separator();
    ui.label(format!("Engine frames: {}", diagnostics.frames));
    ui.label(format!("Editor entities: {}", diagnostics.entities));
    ui.label(format!("Commands executed: {}", diagnostics.commands_executed));
    ui.label(format!("Event records: {}", events.len()));
    ui.separator();
    ui.label(format!("Viewport: {:?}", editor_state.viewport_mode));
    ui.label(format!("Gizmo: {:?}", editor_state.gizmo_mode));
    ui.separator();
    ui.strong("Capabilities");
    for feature in features.iter() {
        ui.label(format!("✓ {feature:?}"));
    }
}

fn systems(ui: &mut egui::Ui, graph: &EngineGraphRegistry) {
    ui.label(format!("Registered systems: {}", graph.iter().count()));
    egui::ScrollArea::vertical().show(ui, |ui| {
        for system in graph.iter() {
            ui.collapsing(&system.name, |ui| {
                ui.label(format!("Schedule: {}", system.schedule));
                if !system.reads.is_empty() {
                    ui.label(format!("Reads: {}", system.reads.join(", ")));
                }
                if !system.writes.is_empty() {
                    ui.label(format!("Writes: {}", system.writes.join(", ")));
                }
                if !system.after.is_empty() {
                    ui.label(format!("After: {}", system.after.join(", ")));
                }
                if !system.before.is_empty() {
                    ui.label(format!("Before: {}", system.before.join(", ")));
                }
            });
        }
    });
}

fn event_monitor(ui: &mut egui::Ui, events: &EngineEventMonitor) {
    ui.horizontal(|ui| {
        ui.label(format!("{} buffered events", events.len()));
    });
    egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
        for event in events.iter().rev() {
            ui.monospace(format!("#{:04}  frame {:>7}  {:<20} {}", event.sequence, event.frame, event.kind, event.payload));
        }
    });
}

fn query_visualizer(ui: &mut egui::Ui, selection: &SelectionState, registry: &ComponentRegistry) {
    let Some(entity) = selection.primary() else {
        ui.label("Select an entity to inspect its ECS composition.");
        return;
    };
    ui.monospace(format!("Entity {:?}", entity));
    ui.separator();
    ui.label("Registered component classes:");
    for descriptor in registry.iter() {
        ui.label(format!("{} · {}", descriptor.category, descriptor.label));
    }
    ui.separator();
    ui.label("Use the Inspector for editable values; this view focuses on ECS metadata and query-oriented diagnostics.");
}

fn state_editor(ui: &mut egui::Ui, project: &ProjectState, editor_state: &EditorUiState) {
    ui.heading("Runtime / Editor State");
    ui.label(format!("Project mode: {:?}", project.mode));
    ui.label(format!("Viewport mode: {:?}", editor_state.viewport_mode));
    ui.label(format!("Gizmo mode: {:?}", editor_state.gizmo_mode));
    ui.label(format!("Transform space: {:?}", editor_state.transform_space));
    ui.separator();
    ui.label("Play controls remain on the main toolbar (F6/F7/F8). State changes are driven through the shared command bus.");
}

#[allow(dead_code)]
fn feature_enabled(features: &EngineFeatureRegistry, feature: EngineFeature) -> bool {
    features.is_enabled(feature)
}
