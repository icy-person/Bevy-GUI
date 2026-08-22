//! Docking workspace subsystem.

mod state;
mod viewer;

pub use state::{EditorDockState, EditorTab, TransformEdit};
pub use viewer::{show_dock_area, DockViewer};
