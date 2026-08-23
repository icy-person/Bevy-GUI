use bevy::prelude::*;
use std::collections::VecDeque;

/// A single entity transform before/after an editor mutation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransformSnapshot {
    pub entity: Entity,
    pub transform: Transform,
}

/// A named editor transaction containing all affected entities. Grouping
/// multiple entities into one command makes multi-selection transforms undo as
/// one operation rather than one operation per entity.
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
}

/// Bounded undo/redo history for transform authoring.
///
/// The public legacy `push/undo/redo` API remains available for the simple
/// single-entity path, while `push_transaction/undo_transaction` provides the
/// production path for grouped editor operations.
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

    pub fn push(&mut self, snapshot: TransformSnapshot) {
        let current = self.current_transform(snapshot.entity);
        if let Some(current) = current {
            self.push_transaction(TransformTransaction::new(
                "Transform",
                vec![TransformSnapshot { entity: snapshot.entity, transform: current }],
                vec![snapshot],
            ));
        } else {
            self.push_transaction(TransformTransaction::new(
                "Transform",
                Vec::new(),
                vec![snapshot],
            ));
        }
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

    /// Records a complete before/after capture for the supplied entities.
    pub fn capture_transaction<I, F>(
        &mut self,
        label: impl Into<String>,
        before: I,
        after: I,
    ) where
        I: IntoIterator<Item = TransformSnapshot>,
        F: Fn(Entity) -> Option<Transform>,
    {
        let _ = std::marker::PhantomData::<F>;
        self.push_transaction(TransformTransaction::new(
            label,
            before.into_iter().collect(),
            after.into_iter().collect(),
        ));
    }

    fn current_transform(&self, _entity: Entity) -> Option<Transform> {
        // The legacy `push` API has no World/Query access and therefore cannot
        // know the current value by itself. Callers that need grouped history
        // should use `push_transaction` with explicit before/after snapshots.
        None
    }

    pub fn undo(&mut self, transforms: &mut Query<&mut Transform>) {
        let Some(transaction) = self.undo.pop_back() else { return; };
        let mut applied_after = Vec::with_capacity(transaction.after.len());
        for snapshot in &transaction.after {
            if let Ok(current) = transforms.get(snapshot.entity) {
                applied_after.push(TransformSnapshot { entity: snapshot.entity, transform: *current });
            }
        }
        apply_snapshots(transforms, &transaction.before);
        self.redo.push_back(TransformTransaction::new(transaction.label, transaction.before, applied_after));
    }

    pub fn redo(&mut self, transforms: &mut Query<&mut Transform>) {
        let Some(transaction) = self.redo.pop_back() else { return; };
        let mut applied_before = Vec::with_capacity(transaction.after.len());
        for snapshot in &transaction.before {
            if let Ok(current) = transforms.get(snapshot.entity) {
                applied_before.push(TransformSnapshot { entity: snapshot.entity, transform: *current });
            }
        }
        apply_snapshots(transforms, &transaction.after);
        self.undo.push_back(TransformTransaction::new(transaction.label, applied_before, transaction.after));
    }

    pub fn can_undo(&self) -> bool { !self.undo.is_empty() }
    pub fn can_redo(&self) -> bool { !self.redo.is_empty() }

    pub fn peek_undo_label(&self) -> Option<&str> { self.undo.back().map(|transaction| transaction.label.as_str()) }
    pub fn peek_redo_label(&self) -> Option<&str> { self.redo.back().map(|transaction| transaction.label.as_str()) }

    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
        self.last_label = None;
    }

    pub fn undo_len(&self) -> usize { self.undo.len() }
    pub fn redo_len(&self) -> usize { self.redo.len() }
    pub fn capacity(&self) -> usize { self.capacity }
    pub fn last_label(&self) -> Option<&str> { self.last_label.as_deref() }

    pub fn set_capacity(&mut self, capacity: usize) {
        self.capacity = capacity.max(1);
        while self.undo.len() > self.capacity { self.undo.pop_front(); }
        while self.redo.len() > self.capacity { self.redo.pop_front(); }
    }

    pub fn transactions(&self) -> impl Iterator<Item = &TransformTransaction> { self.undo.iter() }
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
        TransformSnapshot { entity: Entity::from_raw_u32(entity), transform: Transform::from_xyz(x, 0.0, 0.0) }
    }

    #[test]
    fn transaction_is_grouped() {
        let mut history = TransformHistory::with_capacity(8);
        let transaction = TransformTransaction::new(
            "Move Selection",
            vec![snapshot(1, 0.0), snapshot(2, 0.0)],
            vec![snapshot(1, 1.0), snapshot(2, 2.0)],
        );
        history.push_transaction(transaction);
        assert_eq!(history.undo_len(), 1);
        assert_eq!(history.peek_undo_label(), Some("Move Selection"));
    }

    #[test]
    fn empty_transactions_are_ignored() {
        let mut history = TransformHistory::default();
        history.push_transaction(TransformTransaction::new("Empty", Vec::new(), Vec::new()));
        assert_eq!(history.undo_len(), 0);
    }
}
