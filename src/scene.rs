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
            format_version: 1,
            entities: entities
                .into_iter()
                .map(|(name, transform)| SceneEntity {
                    name,
                    translation: transform.translation.to_array(),
                    rotation: [
                        transform.rotation.x,
                        transform.rotation.y,
                        transform.rotation.z,
                        transform.rotation.w,
                    ],
                    scale: transform.scale.to_array(),
                })
                .collect(),
        }
    }
}

pub fn spawn_scene(commands: &mut Commands, document: &SceneDocument) -> Vec<Entity> {
    document
        .entities
        .iter()
        .map(|entity| {
            commands
                .spawn((
                    Name::new(entity.name.clone()),
                    Transform {
                        translation: Vec3::from_array(entity.translation),
                        rotation: Quat::from_xyzw(
                            entity.rotation[0],
                            entity.rotation[1],
                            entity.rotation[2],
                            entity.rotation[3],
                        ),
                        scale: Vec3::from_array(entity.scale),
                    },
                    SceneNode,
                ))
                .id()
        })
        .collect()
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
        assert_eq!(restored.format_version, 1);
        assert_eq!(restored.entities.len(), 1);
        assert_eq!(restored.entities[0].name, "Player");
        assert_eq!(restored.entities[0].translation, [1.0, 2.0, 3.0]);
        assert_eq!(restored.entities[0].scale, [2.0, 2.0, 2.0]);
    }
}
