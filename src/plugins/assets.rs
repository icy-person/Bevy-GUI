use bevy::prelude::*;
use bevy_egui::egui;

use crate::{
    assets::AssetDatabase,
    editor::{EditorPlugin, EditorPluginRegistry},
    panel::PanelRegistry,
};

pub struct AssetBrowserPlugin;
impl Default for AssetBrowserPlugin { fn default() -> Self { Self } }

impl EditorPlugin for AssetBrowserPlugin {
    fn name(&self) -> &'static str { "asset-browser" }

    fn build(&self, app: &mut App) {
        app.world_mut().resource_mut::<EditorPluginRegistry>().register(self.name(), "1.1");
        app.world_mut().resource_mut::<PanelRegistry>().register(
            crate::panel::PanelId("assets"), "Assets", asset_panel,
        );
    }
}

fn asset_panel(world: &mut World, ui: &mut egui::Ui) {
    // Snapshot all data needed to render before mutating the ECS resource.
    // This keeps the UI callback borrow-safe and also prevents iterator lifetimes
    // from crossing the mutable resource updates below.
    let (mut search, refresh_requested, selected, counts, entries) = {
        let Some(database) = world.get_resource::<AssetDatabase>() else {
            ui.label("Asset database is not initialized.");
            return;
        };
        let counts = database.counts();
        let entries = database
            .filtered()
            .take(500)
            .map(|entry| (entry.path.clone(), entry.kind, entry.bytes))
            .collect::<Vec<_>>();
        (
            database.search.clone(),
            database.refresh_requested,
            database.selected.clone(),
            counts,
            entries,
        )
    };

    let mut selected = selected;
    let mut refresh = false;
    let mut search_changed = false;

    ui.horizontal(|ui| {
        ui.strong("Assets");
        ui.weak(format!("{} files", entries.len()));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("↻").on_hover_text("Refresh asset index").clicked() {
                refresh = true;
            }
        });
    });

    search_changed = ui
        .add(
            egui::TextEdit::singleline(&mut search)
                .hint_text("Search assets…")
                .desired_width(ui.available_width()),
        )
        .changed();

    ui.add_space(6.0);
    ui.horizontal_wrapped(|ui| {
        asset_count_chip(ui, "Scenes", counts[0]);
        asset_count_chip(ui, "Textures", counts[1]);
        asset_count_chip(ui, "Meshes", counts[2]);
        asset_count_chip(ui, "Materials", counts[3]);
        asset_count_chip(ui, "Audio", counts[4]);
        asset_count_chip(ui, "Scripts", counts[5]);
        asset_count_chip(ui, "Data", counts[6]);
    });
    ui.separator();

    egui::ScrollArea::vertical()
        .id_salt("asset_browser_list")
        .show(ui, |ui| {
            if entries.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label("No assets match the current filter.");
                });
                return;
            }
            for (path, kind, bytes) in entries {
                let is_selected = selected.as_ref() == Some(&path);
                let label = path.display().to_string();
                let size = format_size(bytes);
                let response = ui.selectable_label(
                    is_selected,
                    format!("{}  {}  ·  {}", kind_icon(kind.label()), label, size),
                );
                if response.clicked() {
                    selected = Some(path.clone());
                }
                response.on_hover_text(path.display().to_string());
            }
        });

    if search_changed || selected != world.get_resource::<AssetDatabase>().and_then(|db| db.selected.clone()) || refresh {
        if let Some(mut db) = world.get_resource_mut::<AssetDatabase>() {
            if search_changed { db.search = search; }
            if refresh { db.refresh_requested = true; }
            if selected != db.selected { db.selected = selected.clone(); }
        }
    } else if refresh_requested {
        // Preserve an already pending refresh request without borrowing the resource during UI drawing.
        if let Some(mut db) = world.get_resource_mut::<AssetDatabase>() {
            db.refresh_requested = true;
        }
    }

    if let Some(selected_path) = selected {
        ui.separator();
        ui.small("Selected asset");
        ui.monospace(selected_path.display().to_string());
        if ui.button("Open in system file manager").clicked() {
            tracing::info!(path = %selected_path.display(), "asset browser open requested");
        }
    }
}

fn asset_count_chip(ui: &mut egui::Ui, label: &str, count: usize) { ui.small(format!("{label}: {count}")); }

fn kind_icon(label: &str) -> &'static str {
    match label {
        "Scene" => "▱", "Texture" => "▧", "Mesh" => "◇", "Material" => "◈",
        "Audio" => "◉", "Script" => "λ", "Data" => "≡", _ => "•",
    }
}

fn format_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    let value = bytes as f64;
    if value >= GB { format!("{:.1} GB", value / GB) }
    else if value >= MB { format!("{:.1} MB", value / MB) }
    else if value >= KB { format!("{:.1} KB", value / KB) }
    else { format!("{} B", bytes) }
}
