use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{fs, io, path::Path};
use thiserror::Error;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct SceneNode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneDocument {
    pub format_version: u32,
    pub entities: Vec<SceneEntity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneEntity {
    pub id: u64,
    pub parent: Option<u64>,
    pub name: String,
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
    #[serde(default = "default_visible")]
    pub visible: bool,
}

fn default_visible() -> bool {
    true
}

impl SceneDocument {
    pub fn from_entities<I>(entities: I) -> Self
    where
        I: IntoIterator<Item = (Entity, String, Transform, Option<Entity>)>,
    {
        self::build_document(entities.into_iter().map(|(entity, name, transform, parent)| {
            (entity, name, transform, parent, true)
        }))
    }

    pub fn from_entities_with_visibility<I>(entities: I) -> Self
    where
        I: IntoIterator<Item = (Entity, String, Transform, Option<Entity>, bool)>,
    {
        build_document(entities)
    }
}

fn build_document<I>(entities: I) -> SceneDocument
where
    I: IntoIterator<Item = (Entity, String, Transform, Option<Entity>, bool)>,
{
    let snapshot: Vec<_> = entities.into_iter().collect();
    let mut ids = std::collections::BTreeMap::new();
    for (index, (entity, _, _, _, _)) in snapshot.iter().enumerate() {
        ids.insert(*entity, index as u64 + 1);
    }

    SceneDocument {
        format_version: 3,
        entities: snapshot
            .into_iter()
            .enumerate()
            .map(|(index, (_entity, name, transform, parent, visible))| {
                SceneEntity::from_transform(
                    index as u64 + 1,
                    parent.and_then(|value| ids.get(&value).copied()),
                    name,
                    transform,
                    visible,
                )
            })
            .collect(),
    }
}

impl SceneEntity {
    fn from_transform(
        id: u64,
        parent: Option<u64>,
        name: String,
        transform: Transform,
        visible: bool,
    ) -> Self {
        Self {
            id,
            parent,
            name,
            translation: transform.translation.to_array(),
            rotation: [
                transform.rotation.x,
                transform.rotation.y,
                transform.rotation.z,
                transform.rotation.w,
            ],
            scale: transform.scale.to_array(),
            visible,
        }
    }

    pub fn transform(&self) -> Transform {
        Transform {
            translation: Vec3::from_array(self.translation),
            rotation: Quat::from_xyzw(
                self.rotation[0],
                self.rotation[1],
                self.rotation[2],
                self.rotation[3],
            ),
            scale: Vec3::from_array(self.scale),
        }
    }
}

pub fn spawn_scene(commands: &mut Commands, document: &SceneDocument) -> Vec<Entity> {
    let mut by_id = std::collections::BTreeMap::new();
    let mut pending_parents = Vec::new();
    let mut spawned = Vec::with_capacity(document.entities.len());

    for entity in &document.entities {
        let spawned_entity = commands
            .spawn((
                Name::new(entity.name.clone()),
                entity.transform(),
                if entity.visible { Visibility::Visible } else { Visibility::Hidden },
                SceneNode,
                crate::EditorParent(None),
            ))
            .id();
        by_id.insert(entity.id, spawned_entity);
        pending_parents.push((spawned_entity, entity.parent));
        spawned.push(spawned_entity);
    }

    for (entity, parent_id) in pending_parents {
        if let Some(parent_id) = parent_id.and_then(|id| by_id.get(&id).copied()) {
            commands.entity(entity).insert(crate::EditorParent(Some(parent_id)));
        }
    }

    spawned
}

#[derive(Debug, Error)]
pub enum SceneIoError {
    #[error("failed to create scene directory: {0}")]
    CreateDirectory(#[source] io::Error),
    #[error("failed to serialize scene: {0}")]
    Serialize(#[source] serde_json::Error),
    #[error("failed to write scene: {0}")]
    Write(#[source] io::Error),
    #[error("failed to read scene: {0}")]
    Read(#[source] io::Error),
    #[error("failed to parse scene: {0}")]
    Parse(#[source] serde_json::Error),
}

pub fn save_scene(path: &Path, document: &SceneDocument) -> Result<(), SceneIoError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(SceneIoError::CreateDirectory)?;
    }
    let json = serde_json::to_string_pretty(document).map_err(SceneIoError::Serialize)?;
    fs::write(path, json).map_err(SceneIoError::Write)
}

pub fn load_scene(path: &Path) -> Result<SceneDocument, SceneIoError> {
    let json = fs::read_to_string(path).map_err(SceneIoError::Read)?;
    serde_json::from_str(&json).map_err(SceneIoError::Parse)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scene_json_round_trip() {
        let parent = Entity::from_raw(1);
        let child = Entity::from_raw(2);
        let document = SceneDocument::from_entities_with_visibility([
            (parent, "Root".to_owned(), Transform::default(), None, true),
            (
                child,
                "Player".to_owned(),
                Transform {
                    translation: Vec3::new(1.0, 2.0, 3.0),
                    rotation: Quat::from_rotation_y(0.5),
                    scale: Vec3::splat(2.0),
                },
                Some(parent),
                false,
            ),
        ]);
        let json = serde_json::to_string(&document).expect("scene serialization should succeed");
        let restored: SceneDocument =
            serde_json::from_str(&json).expect("scene parsing should succeed");
        assert_eq!(restored.format_version, 3);
        assert_eq!(restored.entities.len(), 2);
        assert_eq!(restored.entities[1].name, "Player");
        assert_eq!(restored.entities[1].parent, Some(1));
        assert!(!restored.entities[1].visible);
    }
}
