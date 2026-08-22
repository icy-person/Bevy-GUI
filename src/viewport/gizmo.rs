use bevy::prelude::*;

use crate::{
    history::{TransformHistory, TransformSnapshot},
    project::ProjectState,
    selection::SelectionState,
    settings::EditorSettingsState,
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
    settings: Res<EditorSettingsState>,
    mut history: ResMut<TransformHistory>,
    mut project: ResMut<ProjectState>,
    mut transforms: Query<&mut Transform>,
) {
    if tracker.active_last_frame
        && !gizmo_state.active
        && let Some(entity) = gizmo_state.entity
    {
        history.push(TransformSnapshot {
            entity,
            transform: gizmo_state.start_transform,
        });
        if settings.settings.viewport.snap_enabled {
            if let Ok(mut transform) = transforms.get_mut(entity) {
                let t = settings.settings.viewport.snap_translation.max(0.001);
                let r = settings
                    .settings
                    .viewport
                    .snap_rotation_degrees
                    .max(0.1)
                    .to_radians();
                let s = settings.settings.viewport.snap_scale.max(0.001);
                transform.translation = snap_vec3(transform.translation, t);
                transform.rotation = Quat::from_euler(
                    EulerRot::XYZ,
                    snap_angle(transform.rotation.to_euler(EulerRot::XYZ).0, r),
                    snap_angle(transform.rotation.to_euler(EulerRot::XYZ).1, r),
                    snap_angle(transform.rotation.to_euler(EulerRot::XYZ).2, r),
                );
                transform.scale = snap_vec3(transform.scale, s);
            }
        }
        project.dirty = true;
    }
    tracker.active_last_frame = gizmo_state.active;
}

fn snap_vec3(value: Vec3, step: f32) -> Vec3 {
    value.map(|component| (component / step).round() * step)
}

fn snap_angle(value: f32, step: f32) -> f32 {
    (value / step).round() * step
}
