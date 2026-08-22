use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditorParent(pub Option<Entity>);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneNodeModel {
    pub id: u64,
    pub parent: Option<u64>,
    pub name: String,
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

impl SceneNodeModel {
    pub fn from_transform(id: u64, parent: Option<u64>, name: String, transform: Transform) -> Self {
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

#[derive(Resource, Default, Debug, Clone)]
pub struct SceneEditorState {
    pub path: Option<std::path::PathBuf>,
    pub revision: u64,
    pub saved_revision: u64,
}

impl SceneEditorState {
    pub fn mark_dirty(&mut self) {
        self.revision = self.revision.saturating_add(1);
    }

    pub fn mark_saved(&mut self) {
        self.saved_revision = self.revision;
    }

    pub fn dirty(&self) -> bool {
        self.revision != self.saved_revision
    }
}
