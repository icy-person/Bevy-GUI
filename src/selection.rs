use bevy::prelude::*;

/// Central selection state shared by panels and tools.
#[derive(Resource, Default, Debug)]
pub struct SelectionState {
    pub entity: Option<Entity>,
}

impl SelectionState {
    pub fn select(&mut self, entity: Entity) {
        self.entity = Some(entity);
    }

    pub fn clear(&mut self) {
        self.entity = None;
    }
}
