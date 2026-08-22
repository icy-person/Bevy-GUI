use bevy::prelude::*;

use crate::{project::EditorMode, scene::SceneDocument};

#[derive(Resource, Default)]
pub struct PlaySession {
    pub snapshot: Option<SceneDocument>,
    pub mode: EditorMode,
}

impl PlaySession {
    pub fn start(&mut self, snapshot: SceneDocument) {
        self.snapshot = Some(snapshot);
        self.mode = EditorMode::Play;
    }

    pub fn pause(&mut self) {
        self.mode = EditorMode::Paused;
    }

    pub fn resume(&mut self) {
        self.mode = EditorMode::Play;
    }

    pub fn stop(&mut self) -> Option<SceneDocument> {
        self.mode = EditorMode::Edit;
        self.snapshot.take()
    }
}
