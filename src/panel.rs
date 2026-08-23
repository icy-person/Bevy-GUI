use bevy::prelude::*;
use bevy_egui::egui;
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

/// Runtime metadata for a panel. This intentionally contains no UI state so
/// a panel can be registered once and the state can be persisted separately.
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

/// Persistable runtime state for panels. The registry owns capabilities;
/// this resource owns user choices.
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

/// Central panel registry. Registration is idempotent and duplicate ids are
/// rejected by returning the previous descriptor so plugin authors can detect
/// accidental collisions instead of silently losing a panel.
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
        if descriptor.default_open {
            self.hidden_by_user.remove(&descriptor.id);
        }
        previous
    }

    pub fn unregister(&mut self, id: PanelId) -> Option<PanelDescriptor> {
        self.state.remove(&id);
        self.hidden_by_user.remove(&id);
        self.panels.remove(&id)
    }

    pub fn contains(&self, id: PanelId) -> bool {
        self.panels.contains_key(&id)
    }

    pub fn get(&self, id: PanelId) -> Option<&PanelDescriptor> {
        self.panels.get(&id)
    }

    pub fn get_mut(&mut self, id: PanelId) -> Option<&mut PanelDescriptor> {
        self.panels.get_mut(&id)
    }

    pub fn state(&self, id: PanelId) -> Option<&PanelState> {
        self.state.get(&id)
    }

    pub fn state_mut(&mut self, id: PanelId) -> Option<&mut PanelState> {
        self.state.get_mut(&id)
    }

    pub fn set_visible(&mut self, id: PanelId, visible: bool) -> bool {
        let Some(state) = self.state.get_mut(&id) else {
            return false;
        };
        state.visible = visible;
        if visible {
            self.hidden_by_user.remove(&id);
        } else {
            self.hidden_by_user.insert(id);
        }
        true
    }

    pub fn toggle_visible(&mut self, id: PanelId) -> bool {
        let current = self.state(id).map(|state| state.visible).unwrap_or(false);
        self.set_visible(id, !current)
    }

    pub fn set_open(&mut self, id: PanelId, open: bool) -> bool {
        let Some(state) = self.state.get_mut(&id) else {
            return false;
        };
        state.open = open;
        true
    }

    pub fn set_pinned(&mut self, id: PanelId, pinned: bool) -> bool {
        let Some(state) = self.state.get_mut(&id) else {
            return false;
        };
        state.pinned = pinned;
        true
    }

    /// Iterates in deterministic UI order: category, explicit order and id.
    pub fn iter(&self) -> impl Iterator<Item = (&PanelId, &PanelDescriptor)> {
        self.panels.iter()
    }

    pub fn iter_visible(&self) -> impl Iterator<Item = &PanelDescriptor> {
        let mut values: Vec<_> = self
            .panels
            .values()
            .filter(|descriptor| {
                self.state
                    .get(&descriptor.id)
                    .map(|state| state.visible)
                    .unwrap_or(descriptor.default_open)
            })
            .collect();
        values.sort_by_key(|descriptor| (descriptor.category, descriptor.order, descriptor.id.0));
        values.into_iter()
    }

    pub fn iter_open(&self) -> impl Iterator<Item = &PanelDescriptor> {
        let mut values: Vec<_> = self
            .panels
            .values()
            .filter(|descriptor| {
                self.state
                    .get(&descriptor.id)
                    .map(|state| state.open && state.visible)
                    .unwrap_or(descriptor.default_open)
            })
            .collect();
        values.sort_by_key(|descriptor| (descriptor.category, descriptor.order, descriptor.id.0));
        values.into_iter()
    }

    pub fn by_category(&self, category: PanelCategory) -> Vec<&PanelDescriptor> {
        let mut values: Vec<_> = self
            .panels
            .values()
            .filter(|descriptor| descriptor.category == category)
            .collect();
        values.sort_by_key(|descriptor| (descriptor.order, descriptor.id.0));
        values
    }

    pub fn visible_ids(&self) -> Vec<PanelId> {
        self.iter_visible().map(|descriptor| descriptor.id).collect()
    }

    pub fn open_ids(&self) -> Vec<PanelId> {
        self.iter_open().map(|descriptor| descriptor.id).collect()
    }

    pub fn reset_layout(&mut self) {
        for descriptor in self.panels.values() {
            self.state
                .insert(descriptor.id, PanelState::from_descriptor(descriptor));
        }
        self.hidden_by_user.clear();
    }

    pub fn hidden_by_user(&self, id: PanelId) -> bool {
        self.hidden_by_user.contains(&id)
    }

    pub fn visible_count(&self) -> usize {
        self.iter_visible().count()
    }

    pub fn open_count(&self) -> usize {
        self.iter_open().count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draw(_: &mut World, _: &mut egui::Ui) {}

    #[test]
    fn registration_creates_default_state() {
        let mut registry = PanelRegistry::default();
        registry.register_descriptor(
            PanelDescriptor::new(PanelId("scene"), "Scene", draw)
                .category(PanelCategory::Scene)
                .order(10),
        );
        assert!(registry.contains(PanelId("scene")));
        assert_eq!(registry.visible_count(), 1);
        assert_eq!(registry.open_count(), 1);
    }

    #[test]
    fn duplicate_registration_returns_previous_descriptor() {
        let mut registry = PanelRegistry::default();
        assert!(registry.register(PanelId("scene"), "Scene", draw).is_none());
        let previous = registry.register(PanelId("scene"), "Scene 2", draw);
        assert_eq!(previous.unwrap().title, "Scene");
        assert_eq!(registry.get(PanelId("scene")).unwrap().title, "Scene 2");
    }

    #[test]
    fn visibility_and_layout_reset_work() {
        let mut registry = PanelRegistry::default();
        registry.register_descriptor(
            PanelDescriptor::new(PanelId("scene"), "Scene", draw)
                .default_open(true),
        );
        registry.set_visible(PanelId("scene"), false);
        assert_eq!(registry.visible_count(), 0);
        registry.reset_layout();
        assert_eq!(registry.visible_count(), 1);
        assert_eq!(registry.open_count(), 1);
    }
}
