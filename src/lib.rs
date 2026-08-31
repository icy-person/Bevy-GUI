//! Bevy-GUI: a plugin-first editor platform for Bevy.
//! The editor is split into project, scene, prefab, asset, component, viewport,
//! runtime, command and UI subsystems. Each subsystem owns real state and
//! persistence APIs so editor features can be extended without a monolithic UI.

pub mod app;
pub mod asset_pipeline;
pub mod assets;
pub mod command;
pub mod command_executor;
pub mod component_registry;
pub mod docking;
pub mod editor;
pub mod export;
pub mod history;
pub mod jackdaw_ui;
pub mod panel;
pub mod plugins;
pub mod prefab;
pub mod profiler;
pub mod project;
pub mod runtime;
pub mod scene;
pub mod scene_model;
pub mod scene_tools;
pub mod selection;
pub mod settings;
pub mod ui;
pub mod viewport;
pub mod viewport2d;

pub use app::BevyGuiPlugin;
pub use asset_pipeline::{ImportDatabase, ImportedAsset, ImportKind, ImportReport, ImportSettings, ImportStatus};
pub use assets::{AssetDatabase, AssetEntry, AssetKind};
pub use command::{EditorCommand, EditorCommandBus, EditorCommandId, EditorCommandRegistry};
pub use command_executor::CommandExecutionState;
pub use component_registry::{ComponentDescriptor, ComponentKind, ComponentRegistry, PropertyDescriptor, PropertyKind};
pub use docking::{EditorDockState, EditorTab};
pub use editor::{EditorPanel, EditorPanelContext, EditorPlugin, EditorPluginRegistry, ViewportMode};
pub use export::{default_profile, export_project, ExportError, ExportProfile, ExportReport};
pub use history::{TransformHistory, TransformSnapshot};
pub use jackdaw_ui::JackdawUiPlugin;
pub use panel::{PanelId, PanelRegistry};
pub use prefab::{load_prefab, prefab_path, save_prefab, spawn_prefab, PrefabDocument, PrefabInstanceOptions, PrefabIoError, PrefabNode};
pub use profiler::EditorProfiler;
pub use project::{create_project, load_project, project_file, save_project, ProjectIoError, ProjectManifest, ProjectState};
pub use runtime::PlaySession;
pub use scene::{load_scene, save_scene, spawn_scene, SceneDocument, SceneEntity, SceneIoError, SceneNode};
pub use scene_model::{EditorParent, SceneEditorState, SceneNodeModel};
pub use scene_tools::{delete_subtree, duplicate_subtree, reparent_entity, validate_scene, SceneSelectionSet, SceneTool, SceneValidationReport};
pub use selection::SelectionState;
pub use settings::{load_settings, save_settings, EditorSettings, EditorSettingsState};
pub use viewport::EditorEntity;
