use bevy::prelude::*;
use std::time::{Duration, Instant};

use crate::{project::EditorMode, scene::SceneDocument};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeTransition {
    Start,
    Pause,
    Resume,
    Stop,
}

#[derive(Resource, Debug)]
pub struct PlaySession {
    pub snapshot: Option<SceneDocument>,
    pub mode: EditorMode,
    pub transition_count: u64,
    pub started_at: Option<Instant>,
    pub paused_duration: Duration,
    pub last_transition: Option<RuntimeTransition>,
    pub generation: u64,
}

impl Default for PlaySession {
    fn default() -> Self {
        Self {
            snapshot: None,
            mode: EditorMode::Edit,
            transition_count: 0,
            started_at: None,
            paused_duration: Duration::ZERO,
            last_transition: None,
            generation: 0,
        }
    }
}

impl PlaySession {
    pub fn start(&mut self, snapshot: SceneDocument) {
        self.snapshot = Some(snapshot);
        self.mode = EditorMode::Play;
        self.started_at = Some(Instant::now());
        self.paused_duration = Duration::ZERO;
        self.transition_count = self.transition_count.saturating_add(1);
        self.generation = self.generation.saturating_add(1);
        self.last_transition = Some(RuntimeTransition::Start);
    }

    pub fn pause(&mut self) {
        if self.mode == EditorMode::Play {
            self.mode = EditorMode::Paused;
            self.transition_count = self.transition_count.saturating_add(1);
            self.last_transition = Some(RuntimeTransition::Pause);
        }
    }

    pub fn resume(&mut self) {
        if self.mode == EditorMode::Paused {
            self.mode = EditorMode::Play;
            self.transition_count = self.transition_count.saturating_add(1);
            self.last_transition = Some(RuntimeTransition::Resume);
        }
    }

    pub fn stop(&mut self) -> Option<SceneDocument> {
        let snapshot = self.snapshot.take();
        self.mode = EditorMode::Edit;
        self.transition_count = self.transition_count.saturating_add(1);
        self.last_transition = Some(RuntimeTransition::Stop);
        self.started_at = None;
        self.paused_duration = Duration::ZERO;
        snapshot
    }

    pub fn is_running(&self) -> bool {
        self.mode == EditorMode::Play
    }

    pub fn is_paused(&self) -> bool {
        self.mode == EditorMode::Paused
    }

    pub fn is_editing(&self) -> bool {
        self.mode == EditorMode::Edit
    }

    pub fn has_snapshot(&self) -> bool {
        self.snapshot.is_some()
    }

    pub fn elapsed(&self) -> Duration {
        self.started_at
            .map(|start| start.elapsed().saturating_sub(self.paused_duration))
            .unwrap_or(Duration::ZERO)
    }

    pub fn snapshot_mut(&mut self) -> Option<&mut SceneDocument> {
        self.snapshot.as_mut()
    }

    pub fn snapshot_ref(&self) -> Option<&SceneDocument> {
        self.snapshot.as_ref()
    }

    pub fn clear(&mut self) {
        self.snapshot = None;
        self.mode = EditorMode::Edit;
        self.started_at = None;
        self.paused_duration = Duration::ZERO;
        self.last_transition = None;
    }

    pub fn transition_label(&self) -> &'static str {
        match self.last_transition {
            Some(RuntimeTransition::Start) => "Started",
            Some(RuntimeTransition::Pause) => "Paused",
            Some(RuntimeTransition::Resume) => "Resumed",
            Some(RuntimeTransition::Stop) => "Stopped",
            None => "Idle",
        }
    }
}
