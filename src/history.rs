use bevy::prelude::*;
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy)]
pub struct TransformSnapshot {
    pub entity: Entity,
    pub transform: Transform,
}

#[derive(Resource, Default)]
pub struct TransformHistory {
    undo: VecDeque<TransformSnapshot>,
    redo: VecDeque<TransformSnapshot>,
    capacity: usize,
}

impl TransformHistory {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            capacity,
            ..Default::default()
        }
    }

    pub fn push(&mut self, snapshot: TransformSnapshot) {
        if self.capacity == 0 {
            self.capacity = 128;
        }
        if self.undo.back().is_none_or(|last| {
            last.entity == snapshot.entity && last.transform == snapshot.transform
        }) {
            return;
        }
        self.undo.push_back(snapshot);
        while self.undo.len() > self.capacity {
            self.undo.pop_front();
        }
        self.redo.clear();
    }

    pub fn undo(&mut self, transforms: &mut Query<&mut Transform>) {
        if let Some(snapshot) = self.undo.pop_back() {
            if let Ok(mut current) = transforms.get_mut(snapshot.entity) {
                let current_snapshot = TransformSnapshot {
                    entity: snapshot.entity,
                    transform: *current,
                };
                *current = snapshot.transform;
                self.redo.push_back(current_snapshot);
            }
        }
    }

    pub fn redo(&mut self, transforms: &mut Query<&mut Transform>) {
        if let Some(snapshot) = self.redo.pop_back() {
            if let Ok(mut current) = transforms.get_mut(snapshot.entity) {
                let current_snapshot = TransformSnapshot {
                    entity: snapshot.entity,
                    transform: *current,
                };
                *current = snapshot.transform;
                self.undo.push_back(current_snapshot);
            }
        }
    }

    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }

    pub fn undo_len(&self) -> usize {
        self.undo.len()
    }

    pub fn redo_len(&self) -> usize {
        self.redo.len()
    }
}
