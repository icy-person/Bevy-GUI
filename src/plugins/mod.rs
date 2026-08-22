//! Built-in editor plugins. Each capability is isolated in its own module so
//! the plugin registry remains orchestration-only.

use bevy::prelude::*;

use crate::EditorPlugin;

pub mod assets;
pub mod console;
pub mod inspector;
pub mod scene;
pub mod viewport;

pub use assets::AssetBrowserPlugin;
pub use console::ConsolePlugin;
pub use inspector::InspectorEditorPlugin;
pub use scene::SceneEditorPlugin;
pub use viewport::ViewportEditorPlugin;

pub fn install_builtin_editor_plugins(app: &mut App) {
    let plugins: [Box<dyn EditorPlugin>; 5] = [
        Box::new(SceneEditorPlugin),
        Box::new(ViewportEditorPlugin),
        Box::new(InspectorEditorPlugin),
        Box::new(AssetBrowserPlugin),
        Box::new(ConsolePlugin),
    ];

    for plugin in plugins {
        plugin.build(app);
    }
}
