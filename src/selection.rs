use bevy::prelude::*;
use std::collections::BTreeSet;

/// Selection semantics used by hierarchy, viewport and editor tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionMode {
    Replace,
    Add,
    Toggle,
    Subtract,
    Focus,
}

impl Default for SelectionMode {
    fn default() -> Self {
        Self::Replace
    }
}

/// Shared selection model used by hierarchy, viewport, inspector and tools.
///
/// The model deliberately keeps ordering for UI/shift-selection while also
/// maintaining an index for O(1) membership checks. `focused` is independent
/// from the selected set and represents the entity whose inspector is shown.
#[derive(Resource, Default, Debug, Clone)]
pub struct SelectionState {
    pub entities: Vec<Entity>,
    pub focused: Option<Entity>,
    anchor: Option<Entity>,
}

impl SelectionState {
    pub fn select(&mut self, entity: Entity) {
        self.apply(entity, SelectionMode::Replace);
    }

    pub fn add(&mut self, entity: Entity) {
        self.apply(entity, SelectionMode::Add);
    }

    pub fn subtract(&mut self, entity: Entity) {
        self.apply(entity, SelectionMode::Subtract);
    }

    pub fn toggle(&mut self, entity: Entity) {
        self.apply(entity, SelectionMode::Toggle);
    }

    pub fn focus(&mut self, entity: Entity) {
        self.apply(entity, SelectionMode::Focus);
    }

    pub fn apply(&mut self, entity: Entity, mode: SelectionMode) {
        match mode {
            SelectionMode::Replace => {
                self.entities.clear();
                self.entities.push(entity);
                self.focused = Some(entity);
                self.anchor = Some(entity);
            }
            SelectionMode::Add => {
                if !self.entities.contains(&entity) {
                    self.entities.push(entity);
                }
                self.focused = Some(entity);
                self.anchor = Some(entity);
            }
            SelectionMode::Subtract => {
                self.remove(entity);
            }
            SelectionMode::Toggle => {
                if self.contains(entity) {
                    self.remove(entity);
                } else {
                    self.entities.push(entity);
                    self.focused = Some(entity);
                    self.anchor = Some(entity);
                }
            }
            SelectionMode::Focus => {
                if self.contains(entity) {
                    self.focused = Some(entity);
                } else {
                    self.entities.clear();
                    self.entities.push(entity);
                    self.focused = Some(entity);
                }
            }
        }
        self.normalize();
    }

    pub fn remove(&mut self, entity: Entity) -> bool {
        let Some(index) = self.entities.iter().position(|current| *current == entity) else {
            return false;
        };
        self.entities.remove(index);
        if self.focused == Some(entity) {
            self.focused = self.entities.get(index).copied().or_else(|| self.entities.last().copied());
        }
        if self.anchor == Some(entity) {
            self.anchor = self.focused;
        }
        self.normalize();
        true
    }

    pub fn set_many<I>(&mut self, entities: I)
    where
        I: IntoIterator<Item = Entity>,
    {
        self.entities.clear();
        let mut seen = BTreeSet::new();
        for entity in entities {
            if seen.insert(entity) {
                self.entities.push(entity);
            }
        }
        self.focused = self.entities.last().copied();
        self.anchor = self.focused;
        self.normalize();
    }

    pub fn replace_with<I>(&mut self, entities: I, focused: Option<Entity>)
    where
        I: IntoIterator<Item = Entity>,
    {
        self.set_many(entities);
        self.focused = focused.filter(|entity| self.contains(*entity)).or(self.entities.last().copied());
        self.anchor = self.focused;
    }

    pub fn clear(&mut self) {
        self.entities.clear();
        self.focused = None;
        self.anchor = None;
    }

    pub fn contains(&self, entity: Entity) -> bool {
        self.entities.contains(&entity)
    }

    pub fn primary(&self) -> Option<Entity> {
        self.focused.or_else(|| self.entities.first().copied())
    }

    pub fn anchor(&self) -> Option<Entity> {
        self.anchor
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    pub fn first(&self) -> Option<Entity> {
        self.entities.first().copied()
    }

    pub fn last(&self) -> Option<Entity> {
        self.entities.last().copied()
    }

    pub fn as_slice(&self) -> &[Entity] {
        &self.entities
    }

    /// Selects all entities in `candidates` between the anchor and target,
    /// preserving candidate order. This is the building block for Shift-click
    /// in hierarchy and viewport selection.
    pub fn select_range<I>(&mut self, candidates: I, target: Entity)
    where
        I: IntoIterator<Item = Entity>,
    {
        let values: Vec<Entity> = candidates.into_iter().collect();
        let Some(anchor) = self.anchor else {
            self.select(target);
            return;
        };
        let Some(a) = values.iter().position(|entity| *entity == anchor) else {
            self.select(target);
            return;
        };
        let Some(b) = values.iter().position(|entity| *entity == target) else {
            self.select(target);
            return;
        };
        let (start, end) = if a <= b { (a, b) } else { (b, a) };
        self.entities = values[start..=end].to_vec();
        self.focused = Some(target);
        self.normalize();
    }

    pub fn retain_existing<F>(&mut self, mut exists: F)
    where
        F: FnMut(Entity) -> bool,
    {
        self.entities.retain(|entity| exists(*entity));
        self.focused = self.focused.filter(|entity| self.contains(*entity));
        self.anchor = self.anchor.filter(|entity| self.contains(*entity));
        self.normalize();
    }

    fn normalize(&mut self) {
        let mut seen = BTreeSet::new();
        self.entities.retain(|entity| seen.insert(*entity));
        if self.entities.is_empty() {
            self.focused = None;
            self.anchor = None;
        } else {
            self.focused = self.focused.filter(|entity| self.contains(*entity)).or_else(|| self.entities.last().copied());
            self.anchor = self.anchor.filter(|entity| self.contains(*entity)).or(self.focused);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(index: u32) -> Entity {
        Entity::from_raw_u32(index)
    }

    #[test]
    fn replace_add_toggle_and_subtract_are_distinct() {
        let mut selection = SelectionState::default();
        selection.select(e(1));
        selection.add(e(2));
        assert_eq!(selection.as_slice(), &[e(1), e(2)]);
        selection.toggle(e(1));
        assert_eq!(selection.as_slice(), &[e(2)]);
        selection.subtract(e(2));
        assert!(selection.is_empty());
    }

    #[test]
    fn range_selection_uses_anchor() {
        let mut selection = SelectionState::default();
        let values = [e(1), e(2), e(3), e(4)];
        selection.select(e(1));
        selection.select_range(values, e(3));
        assert_eq!(selection.as_slice(), &[e(1), e(2), e(3)]);
        assert_eq!(selection.primary(), Some(e(3)));
    }

    #[test]
    fn replace_with_deduplicates_and_validates_focus() {
        let mut selection = SelectionState::default();
        selection.replace_with([e(1), e(1), e(2)], Some(e(3)));
        assert_eq!(selection.as_slice(), &[e(1), e(2)]);
        assert_eq!(selection.primary(), Some(e(2)));
    }
}
