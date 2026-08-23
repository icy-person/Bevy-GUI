use bevy::prelude::*;
use std::collections::BTreeMap;

use crate::{
    project::{EditorMode, ProjectState},
    runtime::PlaySession,
    scene::SceneDocument,
    scene_model::EditorParent,
};

use super::components::EditorEntity;

pub fn apply_runtime_mode(
    project: Res<ProjectState>,
    mut session: ResMut<PlaySession>,
    mut query: Query<
        (
            Entity,
            Option<&Name>,
            &mut Transform,
            Option<&EditorParent>,
            Option<&Visibility>,
        ),
        With<EditorEntity>,
    >,
) {
    match project.mode {
        EditorMode::Play if session.snapshot.is_none() => {
            let snapshot = SceneDocument::from_entities_with_visibility(query.iter_mut().map(
                |(entity, name, transform, parent, visibility)| {
                    (
                        entity,
                        name.map(|value| value.as_str().to_owned())
                            .unwrap_or_else(|| "Entity".into()),
                        *transform,
                        parent.and_then(|value| value.0),
                        visibility.is_none_or(|value| !matches!(value, Visibility::Hidden)),
                    )
                },
            ));
            session.start(snapshot);
        }
        EditorMode::Paused => session.pause(),
        EditorMode::Play if session.snapshot.is_some() => session.resume(),
        EditorMode::Edit if session.snapshot.is_some() => {
            if let Some(snapshot) = session.stop() {
                let saved: BTreeMap<_, _> = snapshot
                    .entities
                    .into_iter()
                    .map(|entity| (entity.name.clone(), entity))
                    .collect();
                for (_, name, mut transform, _, mut visibility) in &mut query {
                    if let Some(saved) = name.and_then(|value| saved.get(value.as_str())) {
                        transform.translation = Vec3::from_array(saved.translation);
                        transform.rotation = Quat::from_xyzw(
                            saved.rotation[0],
                            saved.rotation[1],
                            saved.rotation[2],
                            saved.rotation[3],
                        );
                        transform.scale = Vec3::from_array(saved.scale);
                        *visibility = if saved.visible {
                            Visibility::Visible
                        } else {
                            Visibility::Hidden
                        };
                    }
                }
            }
        }
        _ => {}
    }
}
