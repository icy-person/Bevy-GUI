use bevy::prelude::*;

use crate::{
    docking::{DockViewer, TransformEdit},
    history::{TransformHistory, TransformSnapshot},
    project::ProjectState,
    scene_model::EditorParent,
    selection::SelectionState,
    viewport::EditorEntity,
};

#[derive(Clone)]
pub struct UiActions {
    pub create_entity: bool,
    pub delete_entity: Option<Entity>,
    pub duplicate_entity: Option<Entity>,
    pub save_requested: bool,
    pub transform_edit: Option<TransformEdit>,
    pub name_edit: Option<String>,
    pub parent_selected: bool,
    pub unparent_selected: bool,
}

impl UiActions {
    pub fn from(viewer: &DockViewer<'_>) -> Self {
        Self {
            create_entity: viewer.create_entity,
            delete_entity: viewer.delete_entity,
            duplicate_entity: viewer.duplicate_entity,
            save_requested: viewer.save_requested,
            transform_edit: viewer.transform_edit,
            name_edit: viewer.name_edit.clone(),
            parent_selected: viewer.parent_selected,
            unparent_selected: viewer.unparent_selected,
        }
    }
}

pub fn select_clicked_entity(event: On<Pointer<Click>>, mut selection: ResMut<SelectionState>) {
    selection.select(event.entity);
}

pub fn apply_entity_actions(
    actions: &UiActions,
    commands: &mut Commands,
    selection: &mut SelectionState,
    project: &mut ProjectState,
    history: &mut TransformHistory,
    transforms: &Query<&Transform, With<EditorEntity>>,
    parents: &mut Query<&mut EditorParent, With<EditorEntity>>,
) {
    if actions.create_entity {
        let entity = commands
            .spawn((
                Transform::default(),
                Name::new("Entity"),
                Pickable::default(),
                EditorEntity,
                EditorParent(None),
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
                EditorParent(None),
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

    if actions.parent_selected && selection.entities.len() >= 2
        && let Some(parent) = selection.primary()
    {
        let selected = selection.entities.clone();
        for entity in selected {
            if entity != parent
                && let Ok(mut relation) = parents.get_mut(entity)
            {
                relation.0 = Some(parent);
            }
        }
        project.dirty = true;
    }

    if actions.unparent_selected {
        for entity in selection.entities.iter().copied() {
            if let Ok(mut relation) = parents.get_mut(entity) {
                relation.0 = None;
            }
        }
        project.dirty = true;
    }

    if let Some(name) = &actions.name_edit
        && let Some(entity) = selection.primary()
    {
        commands.entity(entity).insert(Name::new(name.clone()));
        project.dirty = true;
    }

    if let Some(edit) = actions.transform_edit
        && let Ok(current) = transforms.get(edit.entity)
    {
        apply_transform_edit(history, commands, project, *edit, *current);
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
