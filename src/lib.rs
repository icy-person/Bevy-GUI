//! Bevy-GUI: a plugin-first editor platform for Bevy.
//! The public surface is intentionally small; implementation is split into
//! editor, viewport, UI, scene, project, command, runtime and asset subsystems.

pub mod app;
pub mod assets;
pub mod command;
pub mod command_executor;
pub mod docking;
pub mod editor;
pub mod history;
pub mod panel;
pub mod plugins;
pub mod project;
pub mod runtime;
pub mod scene;
pub mod scene_model;
pub mod selection;
pub mod ui;
pub mod viewport;

pub use app::BevyGuiPlugin;
pub use assets::{AssetDatabase, AssetEntry, AssetKind};
pub use command::{EditorCommand, EditorCommandBus, EditorCommandId, EditorCommandRegistry};
pub use command_executor::CommandExecutionState;
pub use docking::{EditorDockState, EditorTab};
pub use editor::{EditorPanel, EditorPanelContext, EditorPlugin, EditorPluginRegistry};
pub use history::{TransformHistory, TransformSnapshot};
pub use panel::PanelRegistry;
pub use project::{load_project, save_project, ProjectIoError, ProjectManifest, ProjectState};
pub use runtime::PlaySession;
pub use scene::{load_scene, save_scene, spawn_scene, SceneDocument, SceneEntity, SceneIoError, SceneNode};
pub use scene_model::{EditorParent, SceneEditorState, SceneNodeModel};
pub use selection::SelectionState;
pub use viewport::EditorEntity;
