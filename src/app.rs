use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};

use crate::{
    command::{EditorCommand, EditorCommandId, EditorCommandRegistry},
    editor::{register_builtin_state, EditorUiState},
    project::{EditorMode, ProjectState},
    selection::SelectionState,
    PanelRegistry,
};

pub struct BevyGuiPlugin;

impl Plugin for BevyGuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .init_resource::<SelectionState>()
            .init_resource::<ProjectState>()
            .init_resource::<EditorCommandRegistry>()
            .init_resource::<PanelRegistry>();

        register_builtin_state(app);
        app.add_systems(Startup, (register_default_commands, setup_editor_scene))
            .add_systems(Update, editor_ui_system);
    }
}

fn register_default_commands(mut commands: ResMut<EditorCommandRegistry>) {
    commands.register(EditorCommand {
        id: EditorCommandId("project.save"),
        label: "Save Project",
        shortcut: Some("Ctrl+S"),
    });
    commands.register(EditorCommand {
        id: EditorCommandId("project.play"),
        label: "Play",
        shortcut: Some("F6"),
    });
    commands.register(EditorCommand {
        id: EditorCommandId("project.pause"),
        label: "Pause",
        shortcut: Some("F7"),
    });
    commands.register(EditorCommand {
        id: EditorCommandId("project.stop"),
        label: "Stop",
        shortcut: Some("F8"),
    });
}

fn setup_editor_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(6.0, 4.0, 7.0).looking_at(Vec3::ZERO, Vec3::Y),
        Name::new("Editor Camera"),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 12_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, -0.5, 0.0)),
        Name::new("Key Light"),
    ));

    let cube = commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::new(2.0, 2.0, 2.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.15, 0.55, 0.95),
                metallic: 0.05,
                perceptual_roughness: 0.32,
                ..default()
            })),
            Transform::default(),
            Name::new("Player"),
        ))
        .id();

    commands.insert_resource(InitialSelected(cube));
}

#[derive(Resource)]
struct InitialSelected(Entity);

fn editor_ui_system(
    mut contexts: EguiContexts,
    mut state: ResMut<EditorUiState>,
    mut project: ResMut<ProjectState>,
    mut selection: ResMut<SelectionState>,
    mut initial: Option<ResMut<InitialSelected>>,
    names: Query<(Entity, Option<&Name>), Without<Window>>,
    transforms: Query<&Transform>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    if let Some(initial) = initial.take() {
        selection.select(initial.0);
    }

    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("Bevy-GUI");
            ui.separator();
            for menu in ["File", "Edit", "View", "Assets", "Scene", "Entity", "Build"] {
                ui.menu_button(menu, |ui| {
                    ui.label(format!("{menu} commands are plugin-provided"));
                });
            }
            ui.separator();
            if ui.button("▶ Play").clicked() {
                project.mode = EditorMode::Play;
            }
            if ui.button("Ⅱ Pause").clicked() {
                project.mode = EditorMode::Paused;
            }
            if ui.button("■ Stop").clicked() {
                project.mode = EditorMode::Edit;
            }
            ui.separator();
            ui.label(format!("Mode: {:?}", project.mode));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("{} entities", names.iter().count()));
            });
        });
    });

    egui::TopBottomPanel::bottom("console_panel")
        .resizable(true)
        .default_height(150.0)
        .show_animated(ctx, state.show_console, |ui| {
            ui.horizontal(|ui| {
                ui.strong("Console");
                ui.separator();
                ui.label("Editor initialized successfully.");
            });
            ui.separator();
            ui.monospace("[info] plugin-first editor kernel online");
            ui.monospace("[info] scene / hierarchy / inspector / assets panels ready");
        });

    egui::SidePanel::left("hierarchy_panel")
        .resizable(true)
        .default_width(240.0)
        .show_animated(ctx, state.show_hierarchy, |ui| {
            ui.heading("Scene");
            ui.separator();
            for (entity, name) in &names {
                let label = name.map(|n| n.as_str()).unwrap_or("Entity");
                let selected = selection.entity == Some(entity);
                if ui.selectable_label(selected, label).clicked() {
                    selection.select(entity);
                }
            }
        });

    egui::SidePanel::right("inspector_panel")
        .resizable(true)
        .default_width(300.0)
        .show_animated(ctx, state.show_inspector, |ui| {
            ui.heading("Inspector");
            ui.separator();
            if let Some(entity) = selection.entity {
                ui.label(format!("Entity {:?}", entity));
                ui.separator();
                if let Ok(transform) = transforms.get(entity) {
                    ui.collapsing("Transform", |ui| {
                        ui.label(format!("Position: {:.2}, {:.2}, {:.2}", transform.translation.x, transform.translation.y, transform.translation.z));
                        ui.label(format!("Scale: {:.2}, {:.2}, {:.2}", transform.scale.x, transform.scale.y, transform.scale.z));
                        let (x, y, z) = transform.rotation.to_euler(EulerRot::XYZ);
                        ui.label(format!("Rotation: {:.1}°, {:.1}°, {:.1}°", x.to_degrees(), y.to_degrees(), z.to_degrees()));
                    });
                }
                ui.collapsing("Components", |ui| {
                    ui.label("Reflection-based component inspectors are an extension point.");
                });
            } else {
                ui.weak("Nothing selected");
            }
        });

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(80.0);
            ui.heading("3D Viewport");
            ui.label("The rendering surface is intentionally isolated from editor panels.");
            ui.add_space(18.0);
            ui.group(|ui| {
                ui.label("Viewport service ready");
                ui.small("Next layer: camera gizmos, picking, grid, transform tools and scene serialization.");
            });
        });
    });

    egui::TopBottomPanel::top("status_toolbar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label(project.name.as_str());
            ui.separator();
            for (label, open) in [
                ("Hierarchy", &mut state.show_hierarchy),
                ("Inspector", &mut state.show_inspector),
                ("Assets", &mut state.show_assets),
                ("Console", &mut state.show_console),
                ("Profiler", &mut state.show_profiler),
            ] {
                ui.checkbox(open, label);
            }
            ui.separator();
            ui.label(state.status.as_str());
        });
    });
}
