use bevy::prelude::*;
use bevy_egui::egui;
use std::collections::{BTreeMap, BTreeSet};

use crate::{EditorCommandRegistry, ProjectState, SelectionState};

/// Common interface implemented by editor-owned panels. Panels operate on the
/// Bevy World so they can inspect and mutate editor resources without being
/// coupled to the application shell.
pub trait EditorPanel: Send + Sync + 'static {
    fn id(&self) -> &'static str;
    fn title(&self) -> &'static str;
    fn draw(&mut self, world: &mut World, ui: &mut egui::Ui);
    fn reset(&mut self) {}
}

/// Extension point for editor subsystems. A plugin owns registration and
/// startup systems; the central editor only controls lifecycle/order.
pub trait EditorPlugin: Send + Sync + 'static {
    fn name(&self) -> &'static str;
    fn build(&self, app: &mut App);
    fn shutdown(&self, _app: &mut App) {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EditorPluginState {
    Registered,
    Enabled,
    Disabled,
}

#[derive(Debug, Clone, Copy)]
pub struct EditorPluginInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub state: EditorPluginState,
    pub order: i32,
}

impl EditorPluginInfo {
    pub const fn new(name: &'static str, version: &'static str) -> Self {
        Self { name, version, state: EditorPluginState::Registered, order: 0 }
    }

    pub const fn order(mut self, order: i32) -> Self {
        self.order = order;
        self
    }
}

/// Plugin registry used by the editor shell and diagnostics UI. It tracks
/// state and ordering in addition to the legacy `(name, version)` iterator.
#[derive(Resource, Default)]
pub struct EditorPluginRegistry {
    plugins: BTreeMap<&'static str, EditorPluginInfo>,
    disabled: BTreeSet<&'static str>,
}

impl EditorPluginRegistry {
    pub fn register(&mut self, name: &'static str, version: &'static str) {
        self.register_info(EditorPluginInfo::new(name, version));
    }

    pub fn register_info(&mut self, info: EditorPluginInfo) {
        self.plugins.insert(info.name, info);
        self.disabled.remove(info.name);
    }

    pub fn unregister(&mut self, name: &'static str) -> Option<EditorPluginInfo> {
        self.disabled.remove(name);
        self.plugins.remove(name)
    }

    pub fn enable(&mut self, name: &'static str) -> bool {
        let Some(info) = self.plugins.get_mut(name) else { return false; };
        info.state = EditorPluginState::Enabled;
        self.disabled.remove(name);
        true
    }

    pub fn disable(&mut self, name: &'static str) -> bool {
        let Some(info) = self.plugins.get_mut(name) else { return false; };
        info.state = EditorPluginState::Disabled;
        self.disabled.insert(name);
        true
    }

    pub fn contains(&self, name: &'static str) -> bool { self.plugins.contains_key(name) }
    pub fn is_enabled(&self, name: &'static str) -> bool { self.plugins.get(name).is_some_and(|info| info.state == EditorPluginState::Enabled) }
    pub fn get(&self, name: &'static str) -> Option<&EditorPluginInfo> { self.plugins.get(name) }

    pub fn iter(&self) -> impl Iterator<Item = (&'static str, &'static str)> + '_ {
        self.plugins.values().map(|info| (info.name, info.version))
    }

    pub fn infos(&self) -> impl Iterator<Item = &EditorPluginInfo> {
        self.plugins.values()
    }

    pub fn enabled_infos(&self) -> Vec<EditorPluginInfo> {
        let mut values: Vec<_> = self
            .plugins
            .values()
            .copied()
            .filter(|info| info.state == EditorPluginState::Enabled)
            .collect();
        values.sort_by_key(|info| (info.order, info.name));
        values
    }

    pub fn count(&self) -> usize { self.plugins.len() }
    pub fn enabled_count(&self) -> usize { self.plugins.values().filter(|info| info.state == EditorPluginState::Enabled).count() }
    pub fn disabled_count(&self) -> usize { self.plugins.values().filter(|info| info.state == EditorPluginState::Disabled).count() }

    pub fn set_order(&mut self, name: &'static str, order: i32) -> bool {
        let Some(info) = self.plugins.get_mut(name) else { return false; };
        info.order = order;
        true
    }

    pub fn reset_states(&mut self) {
        for info in self.plugins.values_mut() {
            info.state = EditorPluginState::Registered;
        }
        self.disabled.clear();
    }
}

/// Context exposed to custom panels. It deliberately contains immutable
/// references to shared editor services so a panel has to use explicit Bevy
/// resources/events for mutations, keeping ownership predictable.
pub struct EditorPanelContext<'a> {
    pub project: &'a ProjectState,
    pub selection: &'a SelectionState,
    pub commands: &'a EditorCommandRegistry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewportMode {
    TwoD,
    ThreeD,
}

impl Default for ViewportMode {
    fn default() -> Self { Self::ThreeD }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoMode {
    Select,
    Move,
    Rotate,
    Scale,
}

impl Default for GizmoMode {
    fn default() -> Self { Self::Select }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformSpace {
    Local,
    World,
}

impl Default for TransformSpace {
    fn default() -> Self { Self::World }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BottomPanel {
    Console,
    Profiler,
    Output,
    Diagnostics,
}

impl Default for BottomPanel {
    fn default() -> Self { Self::Console }
}

#[derive(Resource, Debug, Clone)]
pub struct EditorUiState {
    pub show_hierarchy: bool,
    pub show_inspector: bool,
    pub show_assets: bool,
    pub show_console: bool,
    pub show_profiler: bool,
    pub show_output: bool,
    pub show_diagnostics: bool,
    pub show_viewport: bool,
    pub viewport_mode: ViewportMode,
    pub gizmo_mode: GizmoMode,
    pub transform_space: TransformSpace,
    pub bottom_panel: BottomPanel,
    pub status: String,
    pub dirty_layout: bool,
    pub command_palette_open: bool,
    pub settings_open: bool,
    pub search_focus: bool,
}

impl Default for EditorUiState {
    fn default() -> Self {
        Self {
            show_hierarchy: true,
            show_inspector: true,
            show_assets: true,
            show_console: true,
            show_profiler: false,
            show_output: false,
            show_diagnostics: false,
            show_viewport: true,
            viewport_mode: ViewportMode::ThreeD,
            gizmo_mode: GizmoMode::Select,
            transform_space: TransformSpace::World,
            bottom_panel: BottomPanel::Console,
            status: "Ready".into(),
            dirty_layout: false,
            command_palette_open: false,
            settings_open: false,
            search_focus: false,
        }
    }
}

impl EditorUiState {
    pub fn set_viewport_mode(&mut self, mode: ViewportMode) {
        if self.viewport_mode != mode {
            self.viewport_mode = mode;
            self.dirty_layout = true;
        }
    }

    pub fn set_gizmo_mode(&mut self, mode: GizmoMode) {
        if self.gizmo_mode != mode {
            self.gizmo_mode = mode;
            self.status = format!("Gizmo: {mode:?}");
        }
    }

    pub fn toggle_transform_space(&mut self) {
        self.transform_space = match self.transform_space {
            TransformSpace::Local => TransformSpace::World,
            TransformSpace::World => TransformSpace::Local,
        };
        self.status = format!("Transform space: {:?}", self.transform_space);
    }

    pub fn open_bottom_panel(&mut self, panel: BottomPanel) {
        self.bottom_panel = panel;
        self.show_console = panel == BottomPanel::Console;
        self.show_profiler = panel == BottomPanel::Profiler;
        self.show_output = panel == BottomPanel::Output;
        self.show_diagnostics = panel == BottomPanel::Diagnostics;
    }

    pub fn toggle_panel(&mut self, id: &str) {
        match id {
            "hierarchy" => self.show_hierarchy = !self.show_hierarchy,
            "inspector" => self.show_inspector = !self.show_inspector,
            "assets" => self.show_assets = !self.show_assets,
            "console" => self.show_console = !self.show_console,
            "profiler" => self.show_profiler = !self.show_profiler,
            "output" => self.show_output = !self.show_output,
            "diagnostics" => self.show_diagnostics = !self.show_diagnostics,
            "viewport" => self.show_viewport = !self.show_viewport,
            _ => return,
        }
        self.dirty_layout = true;
    }

    pub fn reset_layout(&mut self) {
        *self = Self::default();
    }

    pub fn set_status(&mut self, status: impl Into<String>) {
        self.status = status.into();
    }
}

pub fn register_builtin_state(app: &mut App) {
    app.init_resource::<EditorPluginRegistry>()
        .init_resource::<EditorUiState>();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_registry_tracks_state_and_order() {
        let mut registry = EditorPluginRegistry::default();
        registry.register_info(EditorPluginInfo::new("scene", "1.0").order(20));
        registry.register_info(EditorPluginInfo::new("viewport", "1.0").order(10));
        registry.enable("scene");
        registry.enable("viewport");
        let names: Vec<_> = registry.enabled_infos().into_iter().map(|info| info.name).collect();
        assert_eq!(names, vec!["viewport", "scene"]);
    }

    #[test]
    fn ui_state_panel_and_viewport_transitions_are_real() {
        let mut state = EditorUiState::default();
        state.set_viewport_mode(ViewportMode::TwoD);
        state.set_gizmo_mode(GizmoMode::Move);
        state.toggle_transform_space();
        state.open_bottom_panel(BottomPanel::Profiler);
        assert_eq!(state.viewport_mode, ViewportMode::TwoD);
        assert_eq!(state.gizmo_mode, GizmoMode::Move);
        assert_eq!(state.transform_space, TransformSpace::Local);
        assert!(state.show_profiler);
        assert!(!state.show_console);
    }
}
