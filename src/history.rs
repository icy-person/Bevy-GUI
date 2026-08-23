use bevy::prelude::*;
use std::collections::VecDeque;

/// A single entity transform captured before or after an editor mutation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransformSnapshot {
    pub entity: Entity,
    pub transform: Transform,
}

/// One atomic editor transaction. Multi-selection edits should be represented
/// by one transaction so Undo/Redo matches what the user perceives as one
/// operation.
#[derive(Debug, Clone, PartialEq)]
pub struct TransformTransaction {
    pub label: String,
    pub before: Vec<TransformSnapshot>,
    pub after: Vec<TransformSnapshot>,
}

impl TransformTransaction {
    pub fn new(label: impl Into<String>, before: Vec<TransformSnapshot>, after: Vec<TransformSnapshot>) -> Self {
        Self { label: label.into(), before, after }
    }

    pub fn is_empty(&self) -> bool {
        self.before.is_empty() && self.after.is_empty()
    }

    pub fn affected_entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.after.iter().map(|snapshot| snapshot.entity)
    }

    pub fn changed_count(&self) -> usize {
        self.before
            .iter()
            .zip(self.after.iter())
            .filter(|(before, after)| before.transform != after.transform || before.entity != after.entity)
            .count()
    }
}

/// Bounded editor history for transform authoring.
#[derive(Resource)]
pub struct TransformHistory {
    undo: VecDeque<TransformTransaction>,
    redo: VecDeque<TransformTransaction>,
    capacity: usize,
    last_label: Option<String>,
}

impl Default for TransformHistory {
    fn default() -> Self {
        Self::with_capacity(256)
    }
}

impl TransformHistory {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            undo: VecDeque::new(),
            redo: VecDeque::new(),
            capacity: capacity.max(1),
            last_label: None,
        }
    }

    /// Backward-compatible single-snapshot insertion. This is intended for
    /// callers that have already captured the previous transform elsewhere;
    /// use `push_transaction` for editor code where both sides are known.
    pub fn push(&mut self, snapshot: TransformSnapshot) {
        self.push_transaction(TransformTransaction::new("Transform", vec![snapshot], vec![snapshot]));
    }

    pub fn push_transaction(&mut self, transaction: TransformTransaction) {
        if transaction.is_empty() || transaction.before == transaction.after {
            return;
        }
        self.last_label = Some(transaction.label.clone());
        self.undo.push_back(transaction);
        self.redo.clear();
        while self.undo.len() > self.capacity {
            self.undo.pop_front();
        }
    }

    pub fn undo(&mut self, transforms: &mut Query<&mut Transform>) {
        let Some(transaction) = self.undo.pop_back() else { return; };
        let mut current_after = Vec::with_capacity(transaction.after.len());
        for snapshot in &transaction.after {
            if let Ok(current) = transforms.get(snapshot.entity) {
                current_after.push(TransformSnapshot { entity: snapshot.entity, transform: *current });
            }
        }
        apply_snapshots(transforms, &transaction.before);
        self.redo.push_back(TransformTransaction::new(
            transaction.label,
            transaction.before,
            current_after,
        ));
    }

    pub fn redo(&mut self, transforms: &mut Query<&mut Transform>) {
        let Some(transaction) = self.redo.pop_back() else { return; };
        let mut current_before = Vec::with_capacity(transaction.before.len());
        for snapshot in &transaction.before {
            if let Ok(current) = transforms.get(snapshot.entity) {
                current_before.push(TransformSnapshot { entity: snapshot.entity, transform: *current });
            }
        }
        apply_snapshots(transforms, &transaction.after);
        self.undo.push_back(TransformTransaction::new(
            transaction.label,
            current_before,
            transaction.after,
        ));
    }

    pub fn can_undo(&self) -> bool { !self.undo.is_empty() }
    pub fn can_redo(&self) -> bool { !self.redo.is_empty() }
    pub fn peek_undo_label(&self) -> Option<&str> { self.undo.back().map(|transaction| transaction.label.as_str()) }
    pub fn peek_redo_label(&self) -> Option<&str> { self.redo.back().map(|transaction| transaction.label.as_str()) }
    pub fn clear(&mut self) { self.undo.clear(); self.redo.clear(); self.last_label = None; }
    pub fn undo_len(&self) -> usize { self.undo.len() }
    pub fn redo_len(&self) -> usize { self.redo.len() }
    pub fn capacity(&self) -> usize { self.capacity }
    pub fn last_label(&self) -> Option<&str> { self.last_label.as_deref() }

    pub fn set_capacity(&mut self, capacity: usize) {
        self.capacity = capacity.max(1);
        while self.undo.len() > self.capacity { self.undo.pop_front(); }
        while self.redo.len() > self.capacity { self.redo.pop_front(); }
    }

    pub fn transactions(&self) -> impl Iterator<Item = &TransformTransaction> {
        self.undo.iter()
    }
}

fn apply_snapshots(transforms: &mut Query<&mut Transform>, snapshots: &[TransformSnapshot]) {
    for snapshot in snapshots {
        if let Ok(mut transform) = transforms.get_mut(snapshot.entity) {
            *transform = snapshot.transform;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot(entity: u32, x: f32) -> TransformSnapshot {
        TransformSnapshot {
            entity: Entity::from_raw_u32(entity),
            transform: Transform::from_xyz(x, 0.0, 0.0),
        }
    }

    #[test]
    fn transaction_is_grouped() {
        let mut history = TransformHistory::with_capacity(8);
        history.push_transaction(TransformTransaction::new(
            "Move Selection",
            vec![snapshot(1, 0.0), snapshot(2, 0.0)],
            vec![snapshot(1, 1.0), snapshot(2, 2.0)],
        ));
        assert_eq!(history.undo_len(), 1);
        assert_eq!(history.peek_undo_label(), Some("Move Selection"));
        assert_eq!(history.transactions().next().unwrap().changed_count(), 2);
    }

    #[test]
    fn identical_transactions_are_ignored() {
        let mut history = TransformHistory::default();
        let value = snapshot(1, 0.0);
        history.push_transaction(TransformTransaction::new("Same", vec![value], vec![value]));
        assert_eq!(history.undo_len(), 0);
    }

    #[test]
    fn capacity_is_bounded() {
        let mut history = TransformHistory::with_capacity(2);
        for index in 0..4 {
            history.push_transaction(TransformTransaction::new(
                format!("T{index}"),
                vec![snapshot(index, 0.0)],
                vec![snapshot(index, 1.0)],
            ));
        }
        assert_eq!(history.undo_len(), 2);
    }
}
