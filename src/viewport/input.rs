use bevy::prelude::*;

use crate::{
    command::{EditorCommandBus, EditorCommandId},
    editor::{EditorUiState, ViewportMode},
    history::TransformHistory,
    project::{EditorMode, ProjectState},
};

pub fn editor_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut project: ResMut<ProjectState>,
    mut editor: ResMut<EditorUiState>,
    mut gizmo: ResMut<TransformGizmoSettings>,
    mut history: ResMut<TransformHistory>,
    mut transforms: Query<&mut Transform>,
    mut bus: ResMut<EditorCommandBus>,
) {
    if keys.just_pressed(KeyCode::Digit1) {
        editor.viewport_mode = ViewportMode::TwoD;
    }
    if keys.just_pressed(KeyCode::Digit2) {
        editor.viewport_mode = ViewportMode::ThreeD;
    }

    if keys.just_pressed(KeyCode::KeyW) {
        gizmo.mode = TransformGizmoMode::Translate;
    }
    if keys.just_pressed(KeyCode::KeyE) {
        gizmo.mode = TransformGizmoMode::Rotate;
    }
    if keys.just_pressed(KeyCode::KeyR) {
        gizmo.mode = TransformGizmoMode::Scale;
    }
    if keys.just_pressed(KeyCode::KeyX) {
        gizmo.space = match gizmo.space {
            TransformGizmoSpace::World => TransformGizmoSpace::Local,
            TransformGizmoSpace::Local => TransformGizmoSpace::World,
        };
    }

    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    let alt = keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight);

    if ctrl && keys.just_pressed(KeyCode::KeyZ) && !shift {
        history.undo(&mut transforms);
        project.dirty = true;
        bus.emit(EditorCommandId("edit.undo"));
    }
    if ctrl && keys.just_pressed(KeyCode::KeyY) {
        history.redo(&mut transforms);
        project.dirty = true;
        bus.emit(EditorCommandId("edit.redo"));
    }
    if ctrl && keys.just_pressed(KeyCode::KeyS) && shift {
        bus.emit(EditorCommandId("scene.save"));
    } else if ctrl && keys.just_pressed(KeyCode::KeyS) {
        bus.emit(EditorCommandId("project.save"));
    }
    if ctrl && keys.just_pressed(KeyCode::KeyD) {
        bus.emit(EditorCommandId("scene.duplicate"));
    }
    if ctrl && keys.just_pressed(KeyCode::KeyA) && shift {
        bus.emit(EditorCommandId("scene.new_entity"));
    }
    if ctrl && keys.just_pressed(KeyCode::KeyB) && shift {
        bus.emit(EditorCommandId("project.export"));
    }
    if ctrl && keys.just_pressed(KeyCode::KeyO) {
        bus.emit(EditorCommandId("scene.open"));
    }
    if ctrl && keys.just_pressed(KeyCode::KeyP) {
        bus.emit(EditorCommandId("scene.prefab_create"));
    }
    if ctrl && keys.just_pressed(KeyCode::KeyI) && shift {
        bus.emit(EditorCommandId("assets.import"));
    }
    if ctrl && keys.just_pressed(KeyCode::KeyV) && shift {
        bus.emit(EditorCommandId("scene.validate"));
    }
    if keys.just_pressed(KeyCode::F5) {
        bus.emit(EditorCommandId("assets.refresh"));
    }
    if keys.just_pressed(KeyCode::Delete) && !alt {
        bus.emit(EditorCommandId("scene.delete"));
    }
    if keys.just_pressed(KeyCode::F6) {
        project.mode = EditorMode::Play;
        bus.emit(EditorCommandId("project.play"));
    }
    if keys.just_pressed(KeyCode::F7) {
        project.mode = EditorMode::Paused;
        bus.emit(EditorCommandId("project.pause"));
    }
    if keys.just_pressed(KeyCode::F8) {
        project.mode = EditorMode::Edit;
        bus.emit(EditorCommandId("project.stop"));
    }
}
