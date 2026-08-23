//! 3D editor viewport subsystem.

use bevy::camera_controller::free_camera::{FreeCamera, FreeCameraPlugin};
use bevy::dev_tools::infinite_grid::InfiniteGridPlugin;
use bevy::prelude::*;

mod components;
mod gizmo;
mod input;
mod picking;
mod runtime;
mod scene;

pub use components::{Editor3dCamera, Editor3dGrid, EditorEntity, GizmoHistoryTracker, InitialSelected};
pub use input::editor_input;
pub use picking::{PlacementAxis, PlacementSettings, ViewportCursor};

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
        .init_resource::<ViewportCursor>()
        .init_resource::<PlacementSettings>()
        .add_systems(Startup, scene::setup_editor_scene)
        .add_systems(
            Update,
            (
                sync_3d_visibility,
                apply_camera_settings,
                picking::update_viewport_cursor,
                select_initial_entity,
                input::editor_input,
                runtime::apply_runtime_mode,
                gizmo::sync_focus,
            )
                .chain(),
        )
        .add_systems(PostUpdate, gizmo::record_finished_drag.after(TransformGizmoSystems))
        .add_systems(PostUpdate, picking::draw_viewport_cursor.after(picking::update_viewport_cursor));
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

fn apply_camera_settings(
    settings: Res<EditorSettingsState>,
    mut cameras: Query<&mut FreeCamera, With<Editor3dCamera>>,
) {
    if !settings.is_changed() {
        return;
    }
    for mut camera in &mut cameras {
        camera.walk_speed = settings.settings.viewport.camera_move_speed.max(0.1);
        camera.run_speed = (camera.walk_speed * 2.0).max(0.2);
        camera.sensitivity = settings.settings.viewport.camera_orbit_speed.max(0.05);
        camera.scroll_factor = settings.settings.viewport.camera_zoom_speed.max(0.001).ln().max(0.001);
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
