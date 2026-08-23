//! Docking workspace subsystem.

mod state;
mod viewer2;

pub use state::{EditorDockState, EditorTab, TransformEdit};
pub use viewer2::{show_dock_area, DockViewer};
