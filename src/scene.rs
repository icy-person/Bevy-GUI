use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{fs, io, path::Path};
use thiserror::Error;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct SceneNode;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ScenePrimitive {
    Cube,
    Plane,
    Sphere,
    Capsule,
    None,
}

impl Default for ScenePrimitive {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SceneVisual {
    #[serde(default)]
    pub primitive: ScenePrimitive,
    #[serde(default)]
    pub mesh_asset: Option<String>,
    #[serde(default)]
    pub material_asset: Option<String>,
    #[serde(default = "default_color")]
    pub base_color: [f32; 4],
    #[serde(default = "default_metallic")]
    pub metallic: f32,
    #[serde(default = "default_roughness")]
    pub roughness: f32,
}

fn default_color() -> [f32; 4] {
    [0.2, 0.55, 0.95, 1.0]
}

fn default_metallic() -> f32 {
    0.0
}

fn default_roughness() -> f32 {
    0.5
}

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
    #[serde(default)]
    pub visual: SceneVisual,
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
            (entity, name, transform, parent, true, SceneVisual::default())
        }))
    }

    pub fn from_entities_with_visibility<I>(entities: I) -> Self
    where
        I: IntoIterator<Item = (Entity, String, Transform, Option<Entity>, bool)>,
    {
        build_document(entities.into_iter().map(|(entity, name, transform, parent, visible)| {
            (entity, name, transform, parent, visible, SceneVisual::default())
        }))
    }

    pub fn from_entities_with_visuals<I>(
        entities: I,
    ) -> Self
    where
        I: IntoIterator<Item = (Entity, String, Transform, Option<Entity>, bool, SceneVisual)>,
    {
        build_document(entities)
    }
}

fn build_document<I>(entities: I) -> SceneDocument
where
    I: IntoIterator<Item = (Entity, String, Transform, Option<Entity>, bool, SceneVisual)>,
{
    let snapshot: Vec<_> = entities.into_iter().collect();
    let mut ids = std::collections::BTreeMap::new();
    for (index, (entity, _, _, _, _, _)) in snapshot.iter().enumerate() {
        ids.insert(*entity, index as u64 + 1);
    }

    SceneDocument {
        format_version: 4,
        entities: snapshot
            .into_iter()
            .enumerate()
            .map(|(index, (_entity, name, transform, parent, visible, visual))| {
                SceneEntity::from_transform(
                    index as u64 + 1,
                    parent.and_then(|value| ids.get(&value).copied()),
                    name,
                    transform,
                    visible,
                    visual,
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
        visual: SceneVisual,
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
            visual,
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
        let entity_id = commands
            .spawn((
                Name::new(entity.name.clone()),
                entity.transform(),
                if entity.visible { Visibility::Visible } else { Visibility::Hidden },
                SceneNode,
                crate::EditorParent(None),
            ))
            .id();
        by_id.insert(entity.id, entity_id);
        pending_parents.push((entity_id, entity.parent));
        spawned.push(entity_id);
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
        let parent = Entity::from_bits(1);
        let child = Entity::from_bits(2);
        let document = SceneDocument::from_entities_with_visuals([
            (
                parent,
                "Root".to_owned(),
                Transform::default(),
                None,
                true,
                SceneVisual {
                    primitive: ScenePrimitive::Cube,
                    ..default()
                },
            ),
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
                SceneVisual::default(),
            ),
        ]);
        let json = serde_json::to_string(&document).expect("scene serialization should succeed");
        let restored: SceneDocument =
            serde_json::from_str(&json).expect("scene parsing should succeed");
        assert_eq!(restored.format_version, 4);
        assert_eq!(restored.entities.len(), 2);
        assert_eq!(restored.entities[1].name, "Player");
        assert_eq!(restored.entities[1].parent, Some(1));
        assert!(!restored.entities[1].visible);
        assert_eq!(restored.entities[0].visual.primitive, ScenePrimitive::Cube);
    }
}
