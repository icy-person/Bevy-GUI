//! Bevy-GUI: a plugin-first editor platform for Bevy.
//! Core services are intentionally replaceable: plugins, panels, commands,
//! selection, docking, scene serialization, runtime sessions and history.

pub mod app;
pub mod command;
pub mod docking;
pub mod editor;
pub mod history;
pub mod panel;
pub mod plugins;
pub mod project;
pub mod runtime;
pub mod scene;
pub mod selection;

pub use app::BevyGuiPlugin;
pub use command::{EditorCommand, EditorCommandId, EditorCommandRegistry};
pub use docking::{EditorDockState, EditorTab};
pub use editor::{EditorPanel, EditorPanelContext, EditorPlugin, EditorPluginRegistry};
pub use history::{TransformHistory, TransformSnapshot};
pub use panel::PanelRegistry;
pub use project::{load_project, save_project, ProjectIoError, ProjectManifest, ProjectState};
pub use runtime::PlaySession;
pub use scene::{load_scene, save_scene, SceneDocument, SceneEntity, SceneIoError};
pub use selection::SelectionState;
