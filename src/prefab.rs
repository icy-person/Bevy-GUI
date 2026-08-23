use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::{BTreeMap, BTreeSet}, fs, io, path::{Path, PathBuf}};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PrefabDocument {
    pub format_version: u32,
    pub name: String,
    pub root_ids: Vec<u64>,
    pub nodes: Vec<PrefabNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PrefabNode {
    pub id: u64,
    pub parent: Option<u64>,
    pub name: String,
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
    pub visible: bool,
}

impl PrefabDocument {
    pub const FORMAT_VERSION: u32 = 1;

    pub fn empty(name: impl Into<String>) -> Self {
        Self {
            format_version: Self::FORMAT_VERSION,
            name: name.into(),
            root_ids: Vec::new(),
            nodes: Vec::new(),
        }
    }

    pub fn from_scene_entities<I>(name: impl Into<String>, entities: I) -> Self
    where
        I: IntoIterator<Item = (Entity, String, Transform, Option<Entity>, bool)>,
    {
        let snapshot: Vec<_> = entities.into_iter().collect();
        let mut ids = BTreeMap::<Entity, u64>::new();
        for (index, (entity, _, _, _, _)) in snapshot.iter().enumerate() {
            ids.insert(*entity, index as u64 + 1);
        }
        let mut root_ids = Vec::new();
        let mut nodes = Vec::with_capacity(snapshot.len());
        for (index, (_entity, name, transform, parent, visible)) in snapshot.into_iter().enumerate() {
            let id = index as u64 + 1;
            let parent_id = parent.and_then(|value| ids.get(&value).copied());
            if parent_id.is_none() {
                root_ids.push(id);
            }
            nodes.push(PrefabNode {
                id,
                parent: parent_id,
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
            });
        }
        Self {
            format_version: Self::FORMAT_VERSION,
            name: name.into(),
            root_ids,
            nodes,
        }
    }

    pub fn node(&self, id: u64) -> Option<&PrefabNode> {
        self.nodes.iter().find(|node| node.id == id)
    }

    pub fn children_of(&self, parent: u64) -> impl Iterator<Item = &PrefabNode> {
        self.nodes.iter().filter(move |node| node.parent == Some(parent))
    }

    pub fn roots(&self) -> impl Iterator<Item = &PrefabNode> {
        self.nodes.iter().filter(|node| node.parent.is_none())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.format_version != Self::FORMAT_VERSION {
            return Err(format!("unsupported prefab version {}", self.format_version));
        }
        let ids: BTreeSet<u64> = self.nodes.iter().map(|node| node.id).collect();
        for node in &self.nodes {
            if let Some(parent) = node.parent {
                if parent == node.id {
                    return Err(format!("node {} is its own parent", node.id));
                }
                if !ids.contains(&parent) {
                    return Err(format!("node {} references missing parent {}", node.id, parent));
                }
            }
        }
        for root in &self.root_ids {
            if !ids.contains(root) {
                return Err(format!("root id {} does not exist", root));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum PrefabIoError {
    #[error("failed to create prefab directory: {0}")]
    CreateDirectory(#[source] io::Error),
    #[error("failed to read prefab: {0}")]
    Read(#[source] io::Error),
    #[error("failed to write prefab: {0}")]
    Write(#[source] io::Error),
    #[error("failed to serialize prefab: {0}")]
    Serialize(#[source] serde_json::Error),
    #[error("failed to parse prefab: {0}")]
    Parse(#[source] serde_json::Error),
    #[error("invalid prefab: {0}")]
    Invalid(String),
}

pub fn save_prefab(path: &Path, prefab: &PrefabDocument) -> Result<(), PrefabIoError> {
    prefab.validate().map_err(PrefabIoError::Invalid)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(PrefabIoError::CreateDirectory)?;
    }
    let json = serde_json::to_string_pretty(prefab).map_err(PrefabIoError::Serialize)?;
    fs::write(path, json).map_err(PrefabIoError::Write)
}

pub fn load_prefab(path: &Path) -> Result<PrefabDocument, PrefabIoError> {
    let json = fs::read_to_string(path).map_err(PrefabIoError::Read)?;
    let prefab: PrefabDocument = serde_json::from_str(&json).map_err(PrefabIoError::Parse)?;
    prefab.validate().map_err(PrefabIoError::Invalid)?;
    Ok(prefab)
}

#[derive(Debug, Clone, Copy)]
pub struct PrefabInstanceOptions {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub visible: bool,
}

impl Default for PrefabInstanceOptions {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            visible: true,
        }
    }
}

pub fn spawn_prefab(
    commands: &mut Commands,
    prefab: &PrefabDocument,
    options: PrefabInstanceOptions,
) -> Result<Vec<Entity>, PrefabIoError> {
    prefab.validate().map_err(PrefabIoError::Invalid)?;
    let mut spawned = BTreeMap::<u64, Entity>::new();
    let mut result = Vec::with_capacity(prefab.nodes.len());

    for node in &prefab.nodes {
        let local = Transform {
            translation: Vec3::from_array(node.translation),
            rotation: Quat::from_xyzw(
                node.rotation[0],
                node.rotation[1],
                node.rotation[2],
                node.rotation[3],
            ),
            scale: Vec3::from_array(node.scale),
        };
        let transform = if node.parent.is_none() {
            Transform {
                translation: options.translation + options.rotation.mul_vec3(local.translation),
                rotation: options.rotation * local.rotation,
                scale: options.scale * local.scale,
            }
        } else {
            local
        };
        let entity = commands
            .spawn((
                Name::new(node.name.clone()),
                transform,
                crate::EditorParent(None),
                if node.visible && options.visible {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                },
            ))
            .id();
        spawned.insert(node.id, entity);
        result.push(entity);
    }

    for node in &prefab.nodes {
        if let Some(parent_id) = node.parent {
            if let (Some(child), Some(parent)) = (spawned.get(&node.id), spawned.get(&parent_id)) {
                commands
                    .entity(*child)
                    .insert(crate::EditorParent(Some(*parent)));
            }
        }
    }
    Ok(result)
}

pub fn prefab_path(project_root: &Path, name: &str) -> PathBuf {
    project_root.join("prefabs").join(format!("{name}.prefab.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefab_validates_parent_graph() {
        let mut prefab = PrefabDocument::empty("Test");
        prefab.nodes.push(PrefabNode {
            id: 1,
            parent: None,
            name: "Root".into(),
            translation: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
            visible: true,
        });
        prefab.root_ids.push(1);
        assert!(prefab.validate().is_ok());
    }

    #[test]
    fn prefab_rejects_self_parent() {
        let mut prefab = PrefabDocument::empty("Test");
        prefab.nodes.push(PrefabNode {
            id: 1,
            parent: Some(1),
            name: "Loop".into(),
            translation: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
            visible: true,
        });
        assert!(prefab.validate().is_err());
    }
}
