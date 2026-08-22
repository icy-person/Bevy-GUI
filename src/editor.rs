use bevy::prelude::*;
use bevy_egui::egui;
use std::collections::BTreeMap;

use crate::{EditorCommandRegistry, PanelRegistry, ProjectState, SelectionState};

pub trait EditorPanel: Send + Sync + 'static {
    fn id(&self) -> &'static str;
    fn title(&self) -> &'static str;
    fn draw(&mut self, world: &mut World, ui: &mut egui::Ui);
}

pub trait EditorPlugin: Send + Sync + 'static {
    fn name(&self) -> &'static str;
    fn build(&self, app: &mut App);
}

#[derive(Resource, Default)]
pub struct EditorPluginRegistry {
    plugins: BTreeMap<&'static str, &'static str>,
}

impl EditorPluginRegistry {
    pub fn register(&mut self, name: &'static str, version: &'static str) {
        self.plugins.insert(name, version);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&'static str, &'static str)> + '_ {
        self.plugins.iter().map(|(k, v)| (*k, *v))
    }
}

pub struct EditorPanelContext<'a> {
    pub project: &'a ProjectState,
    pub selection: &'a SelectionState,
    pub commands: &'a EditorCommandRegistry,
}

#[derive(Resource, Debug, Clone)]
pub struct EditorUiState {
    pub show_hierarchy: bool,
    pub show_inspector: bool,
    pub show_assets: bool,
    pub show_console: bool,
    pub show_profiler: bool,
    pub show_viewport: bool,
    pub status: String,
}

impl Default for EditorUiState {
    fn default() -> Self {
        Self {
            show_hierarchy: true,
            show_inspector: true,
            show_assets: true,
            show_console: true,
            show_profiler: false,
            show_viewport: true,
            status: "Ready".into(),
        }
    }
}

pub fn register_builtin_state(app: &mut App) {
    app.init_resource::<EditorPluginRegistry>()
        .init_resource::<EditorUiState>();
}
