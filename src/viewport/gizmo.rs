use bevy::prelude::*;

use crate::{
    history::{TransformHistory, TransformSnapshot},
    project::ProjectState,
    selection::SelectionState,
};

use super::components::GizmoHistoryTracker;

pub fn sync_focus(
    mut commands: Commands,
    selection: Res<SelectionState>,
    query: Query<(Entity, Option<&TransformGizmoFocus>)>,
) {
    for (entity, focus) in &query {
        match (selection.contains(entity), focus.is_some()) {
            (true, false) => {
                commands.entity(entity).insert(TransformGizmoFocus);
            }
            (false, true) => {
                commands.entity(entity).remove::<TransformGizmoFocus>();
            }
            _ => {}
        }
    }
}

pub fn record_finished_drag(
    mut tracker: ResMut<GizmoHistoryTracker>,
    gizmo_state: Res<TransformGizmoState>,
    mut history: ResMut<TransformHistory>,
    mut project: ResMut<ProjectState>,
) {
    if tracker.active_last_frame
        && !gizmo_state.active
        && let Some(entity) = gizmo_state.entity
    {
        history.push(TransformSnapshot {
            entity,
            transform: gizmo_state.start_transform,
        });
        project.dirty = true;
    }
    tracker.active_last_frame = gizmo_state.active;
}
