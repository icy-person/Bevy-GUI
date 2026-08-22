use bevy::prelude::*;

use crate::{
    command::EditorCommandBus,
    history::{TransformHistory, TransformSnapshot},
    project::ProjectState,
    selection::SelectionState,
    scene::SceneDocument,
};
use crate::docking::{DockViewer, TransformEdit};
use crate::viewport::EditorEntity;

#[derive(Clone, Copy)]
pub struct UiActions {
    pub create_entity: bool,
    pub delete_entity: Option<Entity>,
    pub duplicate_entity: Option<Entity>,
    pub save_requested: bool,
    pub transform_edit: Option<TransformEdit>,
}

impl UiActions {
    pub fn from(viewer: &DockViewer<'_>) -> Self {
        Self {
            create_entity: viewer.create_entity,
            delete_entity: viewer.delete_entity,
            duplicate_entity: viewer.duplicate_entity,
            save_requested: viewer.save_requested,
            transform_edit: viewer.transform_edit,
        }
    }
}

pub fn select_clicked_entity(event: On<Pointer<Click>>, mut selection: ResMut<SelectionState>) {
    selection.select(event.entity);
}

pub fn apply_entity_actions(
    actions: UiActions,
    commands: &mut Commands,
    selection: &mut SelectionState,
    project: &mut ProjectState,
    history: &mut TransformHistory,
    transforms: &Query<&Transform, With<EditorEntity>>,
) {
    if actions.create_entity {
        let entity = commands
            .spawn((
                Transform::default(),
                Name::new("Entity"),
                Pickable::default(),
                EditorEntity,
            ))
            .id();
        commands.entity(entity).observe(select_clicked_entity);
        selection.select(entity);
        project.dirty = true;
    }

    if let Some(entity) = actions.duplicate_entity
        && let Ok(current) = transforms.get(entity)
    {
        let new_entity = commands
            .spawn((
                *current,
                Name::new("Duplicate"),
                Pickable::default(),
                EditorEntity,
            ))
            .id();
        commands.entity(new_entity).observe(select_clicked_entity);
        selection.select(new_entity);
        project.dirty = true;
    }

    if let Some(entity) = actions.delete_entity && selection.contains(entity) {
        commands.entity(entity).despawn();
        selection.entities.retain(|current| *current != entity);
        selection.focused = selection.entities.last().copied();
        project.dirty = true;
    }

    if let Some(edit) = actions.transform_edit
        && let Ok(current) = transforms.get(edit.entity)
    {
        apply_transform_edit(history, commands, project, edit, *current);
    }
}

fn apply_transform_edit(
    history: &mut TransformHistory,
    commands: &mut Commands,
    project: &mut ProjectState,
    edit: TransformEdit,
    current: Transform,
) {
    let next = Transform {
        translation: edit.translation,
        rotation: Quat::from_euler(
            EulerRot::XYZ,
            edit.rotation.x.to_radians(),
            edit.rotation.y.to_radians(),
            edit.rotation.z.to_radians(),
        ),
        scale: edit.scale,
    };
    if current != next {
        history.push(TransformSnapshot {
            entity: edit.entity,
            transform: current,
        });
        commands.entity(edit.entity).insert(next);
        project.dirty = true;
    }
}
