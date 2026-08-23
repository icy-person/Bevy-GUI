use bevy::prelude::*;
use bevy_egui::egui;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Function used to render a registered editor panel.
pub type PanelFn = fn(&mut World, &mut egui::Ui);

/// Stable identifier for a panel. The identifier is persisted in layouts and
/// should therefore never be changed after a panel has shipped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PanelId(pub &'static str);

/// Semantic category used by the workspace to group panels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PanelCategory {
    Scene,
    Assets,
    Debug,
    Project,
    Tools,
    Viewport,
    System,
}

impl Default for PanelCategory {
    fn default() -> Self {
        Self::Tools
    }
}

/// Runtime metadata for a panel.
#[derive(Debug, Clone, Copy)]
pub struct PanelDescriptor {
    pub id: PanelId,
    pub title: &'static str,
    pub category: PanelCategory,
    pub default_open: bool,
    pub closable: bool,
    pub singleton: bool,
    pub order: i32,
    pub draw: PanelFn,
}

impl PanelDescriptor {
    pub const fn new(id: PanelId, title: &'static str, draw: PanelFn) -> Self {
        Self {
            id,
            title,
            category: PanelCategory::Tools,
            default_open: true,
            closable: true,
            singleton: true,
            order: 0,
            draw,
        }
    }

    pub const fn category(mut self, category: PanelCategory) -> Self {
        self.category = category;
        self
    }

    pub const fn default_open(mut self, value: bool) -> Self {
        self.default_open = value;
        self
    }

    pub const fn closable(mut self, value: bool) -> Self {
        self.closable = value;
        self
    }

    pub const fn singleton(mut self, value: bool) -> Self {
        self.singleton = value;
        self
    }

    pub const fn order(mut self, value: i32) -> Self {
        self.order = value;
        self
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PanelState {
    pub open: bool,
    pub pinned: bool,
    pub visible: bool,
}

impl PanelState {
    pub fn from_descriptor(descriptor: &PanelDescriptor) -> Self {
        Self {
            open: descriptor.default_open,
            pinned: descriptor.default_open,
            visible: true,
        }
    }
}

#[derive(Resource, Default)]
pub struct PanelRegistry {
    panels: BTreeMap<PanelId, PanelDescriptor>,
    state: BTreeMap<PanelId, PanelState>,
    hidden_by_user: BTreeSet<PanelId>,
}

impl PanelRegistry {
    pub fn register(&mut self, id: PanelId, title: &'static str, draw: PanelFn) -> Option<PanelDescriptor> {
        self.register_descriptor(PanelDescriptor::new(id, title, draw))
    }

    pub fn register_descriptor(&mut self, descriptor: PanelDescriptor) -> Option<PanelDescriptor> {
        let previous = self.panels.insert(descriptor.id, descriptor);
        self.state
            .entry(descriptor.id)
            .or_insert_with(|| PanelState::from_descriptor(&descriptor));
        previous
    }

    pub fn get(&self, id: PanelId) -> Option<&PanelDescriptor> {
        self.panels.get(&id)
    }

    pub fn state(&self, id: PanelId) -> Option<&PanelState> {
        self.state.get(&id)
    }

    pub fn state_mut(&mut self, id: PanelId) -> Option<&mut PanelState> {
        self.state.get_mut(&id)
    }

    pub fn remove(&mut self, id: PanelId) -> Option<PanelDescriptor> {
        self.state.remove(&id);
        self.hidden_by_user.remove(&id);
        self.panels.remove(&id)
    }

    pub fn set_open(&mut self, id: PanelId, open: bool) {
        if let Some(state) = self.state.get_mut(&id) {
            state.open = open;
        }
    }

    pub fn set_visible(&mut self, id: PanelId, visible: bool) {
        if let Some(state) = self.state.get_mut(&id) {
            state.visible = visible;
        }
        if visible {
            self.hidden_by_user.remove(&id);
        } else {
            self.hidden_by_user.insert(id);
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &PanelDescriptor> {
        self.panels.values()
    }

    pub fn visible_iter(&self) -> impl Iterator<Item = &PanelDescriptor> {
        self.panels.values().filter(|descriptor| {
            self.state
                .get(&descriptor.id)
                .map(|state| state.visible)
                .unwrap_or(false)
        })
    }

    pub fn panel_count(&self) -> usize {
        self.panels.len()
    }

    pub fn visible_count(&self) -> usize {
        self.visible_iter().count()
    }

    pub fn clear(&mut self) {
        self.panels.clear();
        self.state.clear();
        self.hidden_by_user.clear();
    }
}
