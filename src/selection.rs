use bevy::prelude::*;
use std::collections::BTreeSet;

/// Shared selection model used by the hierarchy, viewport, inspector and tools.
#[derive(Resource, Default, Debug, Clone)]
pub struct SelectionState {
    pub entities: Vec<Entity>,
    pub focused: Option<Entity>,
}

impl SelectionState {
    pub fn select(&mut self, entity: Entity) {
        self.entities.clear();
        self.entities.push(entity);
        self.focused = Some(entity);
    }

    pub fn toggle(&mut self, entity: Entity) {
        if let Some(index) = self.entities.iter().position(|current| *current == entity) {
            self.entities.remove(index);
        } else {
            self.entities.push(entity);
        }
        self.focused = self.entities.last().copied();
    }

    pub fn set_many<I>(&mut self, entities: I)
    where
        I: IntoIterator<Item = Entity>,
    {
        self.entities = entities.into_iter().collect::<BTreeSet<_>>().into_iter().collect();
        self.focused = self.entities.last().copied();
    }

    pub fn contains(&self, entity: Entity) -> bool {
        self.entities.contains(&entity)
    }

    pub fn primary(&self) -> Option<Entity> {
        self.focused.or_else(|| self.entities.first().copied())
    }

    pub fn clear(&mut self) {
        self.entities.clear();
        self.focused = None;
    }
}
