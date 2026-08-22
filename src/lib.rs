//! Bevy-GUI: a plugin-first editor shell for Bevy.
//!
//! The editor is split into small plugins and registries so new panels, tools,
//! commands and services can be added without changing the editor kernel.

pub mod app;
pub mod command;
pub mod editor;
pub mod panel;
pub mod plugins;
pub mod project;
pub mod selection;

pub use app::BevyGuiPlugin;
pub use command::{EditorCommand, EditorCommandId, EditorCommandRegistry};
pub use editor::{EditorPanel, EditorPanelContext, EditorPlugin, EditorPluginRegistry};
pub use panel::PanelRegistry;
pub use project::ProjectState;
pub use selection::SelectionState;
