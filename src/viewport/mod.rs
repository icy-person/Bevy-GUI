//! 3D editor viewport subsystem.

use bevy::camera_controller::free_camera::FreeCameraPlugin;
use bevy::dev_tools::infinite_grid::InfiniteGridPlugin;
use bevy::prelude::*;

mod components;
mod gizmo;
mod input;
mod runtime;
mod scene;

pub use components::{Editor3dCamera, Editor3dGrid, EditorEntity, GizmoHistoryTracker, InitialSelected};
pub use input::editor_input;

use crate::editor::{EditorUiState, ViewportMode};
use crate::history::TransformHistory;
use crate::runtime::PlaySession;
use crate::selection::SelectionState;
use crate::settings::EditorSettingsState;

pub fn install_viewport(app: &mut App) {
    app.add_plugins((TransformGizmoPlugin, FreeCameraPlugin, InfiniteGridPlugin))
        .init_resource::<SelectionState>()
        .init_resource::<PlaySession>()
        .insert_resource(TransformHistory::with_capacity(256))
        .init_resource::<GizmoHistoryTracker>()
        .add_systems(Startup, scene::setup_editor_scene)
        .add_systems(
            Update,
            (
                sync_3d_visibility,
                select_initial_entity,
                input::editor_input,
                runtime::apply_runtime_mode,
                gizmo::sync_focus,
            )
                .chain(),
        )
        .add_systems(PostUpdate, gizmo::record_finished_drag.after(TransformGizmoSystems));
}

fn sync_3d_visibility(
    editor: Res<EditorUiState>,
    settings: Res<EditorSettingsState>,
    mut camera_query: Query<&mut Visibility, With<Editor3dCamera>>,
    mut grid_query: Query<&mut Visibility, (With<Editor3dGrid>, Without<Editor3dCamera>)>,
) {
    let visible = editor.viewport_mode == ViewportMode::ThreeD;
    for mut visibility in &mut camera_query {
        *visibility = if visible { Visibility::Inherited } else { Visibility::Hidden };
    }
    let grid_visible = visible && settings.settings.viewport.grid_3d;
    for mut visibility in &mut grid_query {
        *visibility = if grid_visible { Visibility::Inherited } else { Visibility::Hidden };
    }
}

fn select_initial_entity(
    initial: Option<Res<InitialSelected>>,
    mut selection: ResMut<SelectionState>,
) {
    if selection.primary().is_none() && let Some(initial) = initial {
        selection.select(initial.0);
    }
}
