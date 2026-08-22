//! Bevy-GUI: a plugin-first editor kernel for Bevy.
//!
//! Core services are intentionally small and replaceable: plugins, panels,
//! commands, selection, docking, scene serialization and transform history
//! are independent layers.

pub mod app;
pub mod command;
pub mod docking;
pub mod editor;
pub mod history;
pub mod panel;
pub mod plugins;
pub mod project;
pub mod scene;
pub mod selection;

pub use app::BevyGuiPlugin;
pub use command::{EditorCommand, EditorCommandId, EditorCommandRegistry};
pub use docking::{EditorDockState, EditorTab};
pub use editor::{EditorPanel, EditorPanelContext, EditorPlugin, EditorPluginRegistry};
pub use history::{TransformHistory, TransformSnapshot};
pub use panel::PanelRegistry;
pub use project::ProjectState;
pub use scene::{load_scene, save_scene, SceneDocument, SceneEntity, SceneIoError};
pub use selection::SelectionState;
