use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{fs, io, path::Path};
use thiserror::Error;

use crate::scene_model::EditorParent;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct SceneNode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneDocument {
    pub format_version: u32,
    pub entities: Vec<SceneEntity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneEntity {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub parent: Option<u64>,
    pub name: String,
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

impl SceneDocument {
    pub fn from_entities<I>(entities: I) -> Self
    where
        I: IntoIterator<Item = (String, Transform)>,
    {
        Self {
            format_version: 2,
            entities: entities
                .into_iter()
                .enumerate()
                .map(|(index, (name, transform))| SceneEntity::from_transform(index as u64 + 1, None, name, transform))
                .collect(),
        }
    }

    pub fn from_world<I>(entities: I) -> Self
    where
        I: IntoIterator<Item = (Entity, String, Transform, Option<Entity>)>,
    {
        let mut ids = std::collections::BTreeMap::new();
        let snapshot: Vec<_> = entities.into_iter().collect();
        for (index, (entity, _, _, _)) in snapshot.iter().enumerate() {
            ids.insert(*entity, index as u64 + 1);
        }
        Self {
            format_version: 2,
            entities: snapshot
                .into_iter()
                .enumerate()
                .map(|(index, (entity, name, transform, parent))| {
                    SceneEntity::from_transform(
                        index as u64 + 1,
                        parent.and_then(|value| ids.get(&value).copied()),
                        name,
                        transform,
                    )
                })
                .collect(),
        }
    }
}

impl SceneEntity {
    fn from_transform(id: u64, parent: Option<u64>, name: String, transform: Transform) -> Self {
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
    let mut spawned = Vec::with_capacity(document.entities.len());
    let mut by_id = std::collections::BTreeMap::new();
    for entity in &document.entities {
        let id = commands
            .spawn((
                Name::new(entity.name.clone()),
                entity.transform(),
                SceneNode,
                EditorParent(None),
            ))
            .id();
        spawned.push(id);
        by_id.insert(entity.id, id);
    }
    for (index, entity) in document.entities.iter().enumerate() {
        if let Some(parent_id) = entity.parent {
            if let Some(parent) = by_id.get(&parent_id) {
                commands
                    .entity(spawned[index])
                    .insert(EditorParent(Some(*parent)));
            }
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
        let document = SceneDocument::from_entities([(
            "Player".to_owned(),
            Transform {
                translation: Vec3::new(1.0, 2.0, 3.0),
                rotation: Quat::from_rotation_y(0.5),
                scale: Vec3::splat(2.0),
            },
        )]);
        let json = serde_json::to_string(&document).expect("scene serialization should succeed");
        let restored: SceneDocument = serde_json::from_str(&json).expect("scene parsing should succeed");
        assert_eq!(restored.format_version, 2);
        assert_eq!(restored.entities.len(), 1);
        assert_eq!(restored.entities[0].id, 1);
        assert_eq!(restored.entities[0].parent, None);
        assert_eq!(restored.entities[0].name, "Player");
        assert_eq!(restored.entities[0].translation, [1.0, 2.0, 3.0]);
        assert_eq!(restored.entities[0].scale, [2.0, 2.0, 2.0]);
    }
}
