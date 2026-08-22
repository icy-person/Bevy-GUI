use bevy::prelude::*;
use bevy_egui::egui;
use std::collections::BTreeMap;

pub type PanelFn = fn(&mut World, &mut egui::Ui);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PanelId(pub &'static str);

#[derive(Resource, Default)]
pub struct PanelRegistry {
    panels: BTreeMap<PanelId, (&'static str, PanelFn)>,
}

impl PanelRegistry {
    pub fn register(&mut self, id: PanelId, title: &'static str, draw: PanelFn) {
        self.panels.insert(id, (title, draw));
    }

    pub fn iter(&self) -> impl Iterator<Item = (&PanelId, &(&'static str, PanelFn))> {
        self.panels.iter()
    }
}
