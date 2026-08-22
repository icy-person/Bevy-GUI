//! 3D editor viewport subsystem.

use bevy::camera_controller::free_camera::FreeCameraPlugin;
use bevy::dev_tools::infinite_grid::InfiniteGridPlugin;
use bevy::prelude::*;

mod components;
mod gizmo;
mod input;
mod runtime;
mod scene;

pub use components::{EditorEntity, GizmoHistoryTracker, InitialSelected};
pub use input::editor_input;

use crate::history::TransformHistory;
use crate::runtime::PlaySession;
use crate::selection::SelectionState;

pub fn install_viewport(app: &mut App) {
    app.add_plugins((
        TransformGizmoPlugin,
        FreeCameraPlugin,
        InfiniteGridPlugin,
    ))
    .init_resource::<SelectionState>()
    .init_resource::<PlaySession>()
    .insert_resource(TransformHistory::with_capacity(256))
    .init_resource::<GizmoHistoryTracker>()
    .add_systems(Startup, scene::setup_editor_scene)
    .add_systems(
        Update,
        (
            select_initial_entity,
            input::editor_input,
            runtime::apply_runtime_mode,
            gizmo::sync_focus,
        )
            .chain(),
    )
    .add_systems(
        PostUpdate,
        gizmo::record_finished_drag.after(TransformGizmoSystems),
    );
}

fn select_initial_entity(
    initial: Option<Res<InitialSelected>>,
    mut selection: ResMut<SelectionState>,
) {
    if selection.primary().is_none() && let Some(initial) = initial {
        selection.select(initial.0);
    }
}
