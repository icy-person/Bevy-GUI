use bevy::prelude::*;

#[derive(Resource)]
pub struct InitialSelected(pub Entity);

#[derive(Resource, Default)]
pub struct GizmoHistoryTracker {
    pub active_last_frame: bool,
}

#[derive(Component)]
pub struct EditorEntity;

#[derive(Component)]
pub struct Editor3dCamera;

#[derive(Component)]
pub struct Editor3dGrid;
