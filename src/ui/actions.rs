use bevy::prelude::*;
use crate::{
    docking::{DockViewer, TransformEdit},
    history::{TransformHistory, TransformSnapshot, TransformTransaction},
    project::ProjectState,
    scene_model::EditorParent,
    selection::SelectionState,
    viewport::EditorEntity,
};

#[derive(Clone, Default)]
pub struct UiActions {
    pub create_entity: bool,
    pub delete_entity: Option<Entity>,
    pub duplicate_entity: Option<Entity>,
    pub save_requested: bool,
    pub transform_edit: Option<TransformEdit>,
    pub name_edit: Option<String>,
    pub visibility_edit: Option<bool>,
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
            visibility_edit: viewer.visibility_edit,
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
        let entity = commands.spawn((
            Transform::default(),
            Name::new("Entity"),
            Visibility::Inherited,
            Pickable::default(),
            EditorEntity,
            EditorParent(None),
        )).id();
        commands.entity(entity).observe(select_clicked_entity);
        selection.select(entity);
        project.dirty = true;
    }

    if let Some(entity) = actions.duplicate_entity {
        if let Ok(transform) = transforms.get(entity) {
            let duplicate = commands.spawn((
                *transform,
                Name::new("Duplicate"),
                Visibility::Inherited,
                Pickable::default(),
                EditorEntity,
                EditorParent(None),
            )).id();
            commands.entity(duplicate).observe(select_clicked_entity);
            selection.select(duplicate);
            project.dirty = true;
        }
    }

    if let Some(entity) = actions.delete_entity {
        if selection.contains(entity) {
            commands.entity(entity).despawn();
            selection.entities.retain(|current| *current != entity);
            selection.focused = selection.entities.last().copied();
            project.dirty = true;
        }
    }

    if actions.parent_selected && selection.entities.len() >= 2 {
        if let Some(parent) = selection.primary() {
            for entity in selection.entities.clone() {
                if entity != parent {
                    if let Ok(mut value) = parents.get_mut(entity) {
                        value.0 = Some(parent);
                    }
                }
            }
            project.dirty = true;
        }
    }

    if actions.unparent_selected {
        for entity in selection.entities.clone() {
            if let Ok(mut value) = parents.get_mut(entity) {
                value.0 = None;
            }
        }
        project.dirty = true;
    }

    if let Some(name) = &actions.name_edit {
        if let Some(entity) = selection.primary() {
            commands.entity(entity).insert(Name::new(name.clone()));
            project.dirty = true;
        }
    }

    if let Some(visible) = actions.visibility_edit {
        if let Some(entity) = selection.primary() {
            commands.entity(entity).insert(if visible { Visibility::Visible } else { Visibility::Hidden });
            project.dirty = true;
        }
    }

    if let Some(edit) = actions.transform_edit {
        if let Ok(current) = transforms.get(edit.entity) {
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
            if *current != next {
                history.push_transaction(TransformTransaction::new(
                    "Transform",
                    vec![TransformSnapshot { entity: edit.entity, transform: *current }],
                    vec![TransformSnapshot { entity: edit.entity, transform: next }],
                ));
                commands.entity(edit.entity).insert(next);
                project.dirty = true;
            }
        }
    }
}
