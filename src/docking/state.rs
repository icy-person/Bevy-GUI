use bevy::prelude::*;
use egui_dock::dock_state::tree::NodeIndex;
use egui_dock::DockState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EditorTab {
    Viewport,
    Hierarchy,
    Inspector,
    Assets,
    Console,
    Profiler,
    Plugins,
    Settings,
}

impl EditorTab {
    pub fn title(self) -> &'static str {
        match self {
            Self::Viewport => "Viewport",
            Self::Hierarchy => "Hierarchy",
            Self::Inspector => "Inspector",
            Self::Assets => "Asset Browser",
            Self::Console => "Console",
            Self::Profiler => "Profiler",
            Self::Plugins => "Plugins",
            Self::Settings => "Settings",
        }
    }
}

#[derive(Resource)]
pub struct EditorDockState {
    pub state: DockState<EditorTab>,
}

impl Default for EditorDockState {
    fn default() -> Self {
        let mut state = DockState::new(vec![EditorTab::Viewport]);
        let tree = state.main_surface_mut();
        let root = NodeIndex::root();
        let [_old, left] = tree.split_left(root, 0.20, vec![EditorTab::Hierarchy]);
        let [_old, right] = tree.split_right(root, 0.20, vec![EditorTab::Inspector]);
        tree.split_below(root, 0.74, vec![EditorTab::Console, EditorTab::Profiler]);
        tree.split_below(left, 0.65, vec![EditorTab::Assets]);
        tree.split_below(right, 0.65, vec![EditorTab::Plugins, EditorTab::Settings]);
        Self { state }
    }
}

#[derive(Clone, Copy)]
pub struct TransformEdit {
    pub entity: Entity,
    pub translation: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
}
