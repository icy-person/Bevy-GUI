use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::command::{EditorCommandBus, EditorCommandRegistry, EditorCommandId};

#[derive(Resource, Debug, Clone)]
pub struct CommandPaletteState {
    pub open: bool,
    pub query: String,
    pub selected: usize,
}

impl Default for CommandPaletteState {
    fn default() -> Self {
        Self {
            open: false,
            query: String::new(),
            selected: 0,
        }
    }
}

impl CommandPaletteState {
    pub fn toggle(&mut self) {
        self.open = !self.open;
        if !self.open {
            self.query.clear();
            self.selected = 0;
        }
    }

    pub fn close(&mut self) {
        self.open = false;
        self.query.clear();
        self.selected = 0;
    }
}

pub fn install_command_palette(app: &mut App) {
    app.init_resource::<CommandPaletteState>()
        .add_systems(bevy_egui::EguiPrimaryContextPass, command_palette_system);
}

fn command_palette_system(
    mut contexts: EguiContexts,
    mut state: ResMut<CommandPaletteState>,
    registry: Res<EditorCommandRegistry>,
    mut bus: ResMut<EditorCommandBus>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    if !state.open {
        return Ok(());
    }

    let mut filtered = registry
        .iter()
        .filter(|command| {
            if state.query.trim().is_empty() {
                return true;
            }
            let query = state.query.to_ascii_lowercase();
            command.label.to_ascii_lowercase().contains(&query)
                || command.id.0.to_ascii_lowercase().contains(&query)
                || command
                    .shortcut
                    .map(|shortcut| shortcut.to_ascii_lowercase().contains(&query))
                    .unwrap_or(false)
        })
        .cloned()
        .collect::<Vec<_>>();

    filtered.sort_by(|a, b| a.label.cmp(b.label));
    if state.selected >= filtered.len() {
        state.selected = filtered.len().saturating_sub(1);
    }

    let mut execute_index = None;
    let mut close = false;
    egui::Area::new(egui::Id::new("bevy_gui_command_palette"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 74.0))
        .show(ctx, |ui| {
            egui::Frame::window(ui.style()).show(ui, |ui| {
                ui.set_width(620.0);
                ui.vertical(|ui| {
                    ui.heading("Command Palette");
                    ui.add_space(6.0);
                    let response = ui.add_sized(
                        [ui.available_width(), 34.0],
                        egui::TextEdit::singleline(&mut state.query)
                            .hint_text("Search commands, shortcuts, or command ids…"),
                    );
                    if response.changed() {
                        state.selected = 0;
                    }

                    let enter = ui.input(|input| input.key_pressed(egui::Key::Enter));
                    let escape = ui.input(|input| input.key_pressed(egui::Key::Escape));
                    if escape {
                        close = true;
                    }
                    if enter && !filtered.is_empty() {
                        execute_index = Some(state.selected);
                    }

                    ui.add_space(8.0);
                    if filtered.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(16.0);
                            ui.weak("No commands match the current search.");
                            ui.add_space(16.0);
                        });
                    } else {
                        egui::ScrollArea::vertical()
                            .max_height(420.0)
                            .show(ui, |ui| {
                                for (index, command) in filtered.iter().enumerate() {
                                    let selected = index == state.selected;
                                    let row = ui.add(
                                        egui::Button::new(
                                            egui::RichText::new(format!(
                                                "{}    {}",
                                                command.label,
                                                command.shortcut.unwrap_or("")
                                            ))
                                            .size(14.0),
                                        )
                                        .selected(selected)
                                        .min_size(egui::vec2(ui.available_width(), 32.0)),
                                    );
                                    if row.clicked() {
                                        execute_index = Some(index);
                                    }
                                    if row.double_clicked() {
                                        execute_index = Some(index);
                                    }
                                }
                            });
                    }

                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.small("Enter: execute");
                        ui.small("↑↓: select");
                        ui.small("Esc: close");
                    });
                });
            });
        });

    let navigation = ctx.input(|input| {
        (
            input.key_pressed(egui::Key::ArrowUp),
            input.key_pressed(egui::Key::ArrowDown),
        )
    });
    if navigation.0 && state.selected > 0 {
        state.selected -= 1;
    }
    if navigation.1 && state.selected + 1 < filtered.len() {
        state.selected += 1;
    }

    if let Some(index) = execute_index {
        if let Some(command) = filtered.get(index) {
            bus.emit(EditorCommandId(command.id.0));
        }
        state.close();
    } else if close {
        state.close();
    }
    Ok(())
}
