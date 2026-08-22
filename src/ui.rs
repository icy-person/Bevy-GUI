use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use std::{fs, path::{Path, PathBuf}};

use crate::{
    command::EditorCommandRegistry,
    docking::{show_dock_area, DockViewer, EditorDockState, TransformEdit},
    editor::{EditorPluginRegistry, EditorUiState},
    history::{TransformHistory, TransformSnapshot},
    project::{save_project, ProjectState},
    scene::{save_scene, SceneDocument},
    selection::SelectionState,
    viewport::EditorEntity,
};

#[derive(SystemParam)]
pub struct EditorUiParams<'w, 's> {
    pub contexts: EguiContexts<'w, 's>,
    pub dock: ResMut<'w, EditorDockState>,
    pub state: ResMut<'w, EditorUiState>,
    pub project: ResMut<'w, ProjectState>,
    pub selection: ResMut<'w, SelectionState>,
    pub registry: Res<'w, EditorCommandRegistry>,
    pub plugins: Res<'w, EditorPluginRegistry>,
    pub history: ResMut<'w, TransformHistory>,
    pub transforms: Query<'w, 's, &'static Transform, With<EditorEntity>>,
    pub names: Query<'w, 's, (Entity, Option<&'static Name>), With<EditorEntity>>,
    pub commands: Commands<'w, 's>,
}

pub fn install_editor_ui(app: &mut App) {
    app.add_systems(bevy_egui::EguiPrimaryContextPass, editor_ui_system);
}

fn editor_ui_system(mut params: EditorUiParams) -> Result {
    let ctx = params.contexts.ctx_mut()?;
    let entities: Vec<(Entity, String)> = params
        .names
        .iter()
        .map(|(entity, name)| {
            (
                entity,
                name.map(|value| value.as_str().to_owned())
                    .unwrap_or_else(|| format!("Entity {entity:?}")),
            )
        })
        .collect();

    let selected_transform = params.selection.primary().and_then(|entity| {
        params.transforms.get(entity).ok().map(|transform| {
            let (x, y, z) = transform.rotation.to_euler(EulerRot::XYZ);
            TransformEdit {
                entity,
                translation: transform.translation,
                rotation: Vec3::new(x.to_degrees(), y.to_degrees(), z.to_degrees()),
                scale: transform.scale,
            }
        })
    });

    let assets = scan_assets(&params.project.root, 5, 1000);
    let plugin_names: Vec<String> = params
        .plugins
        .iter()
        .map(|(name, version)| format!("{name} v{version}"))
        .collect();

    let mut viewer = DockViewer {
        project: &mut params.project,
        selection: &mut params.selection,
        ui_state: &mut params.state,
        entities: &entities,
        selected_transform,
        assets: &assets,
        plugin_names: &plugin_names,
        command_count: params.registry.iter().count(),
        transform_edit: None,
        viewport_focused: false,
        create_entity: false,
        delete_entity: None,
        duplicate_entity: None,
        save_requested: false,
    };

    let mut root_ui = egui::Ui::new(
        ctx.clone(),
        "editor_root".into(),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );

    egui::CentralPanel::default().show(&mut root_ui, |ui| {
        show_dock_area(ui, &mut params.dock, &mut viewer);
    });

    let UiActions {
        create_entity,
        delete_entity,
        duplicate_entity,
        save_requested,
        transform_edit,
    } = UiActions::from(&viewer);

    if create_entity {
        let entity = params
            .commands
            .spawn((
                Transform::default(),
                Name::new("Entity"),
                Pickable::default(),
                EditorEntity,
            ))
            .id();
        params.commands.entity(entity).observe(select_clicked_entity);
        params.selection.select(entity);
        params.project.dirty = true;
    }

    if let Some(entity) = duplicate_entity
        && let Ok(current) = params.transforms.get(entity)
    {
        let new_entity = params
            .commands
            .spawn((
                *current,
                Name::new("Duplicate"),
                Pickable::default(),
                EditorEntity,
            ))
            .id();
        params
            .commands
            .entity(new_entity)
            .observe(select_clicked_entity);
        params.selection.select(new_entity);
        params.project.dirty = true;
    }

    if let Some(entity) = delete_entity && params.selection.contains(entity) {
        params.commands.entity(entity).despawn();
        params.selection.entities.retain(|current| *current != entity);
        params.selection.focused = params.selection.entities.last().copied();
        params.project.dirty = true;
    }

    if let Some(edit) = transform_edit
        && let Ok(current) = params.transforms.get(edit.entity)
    {
        apply_transform_edit(
            &mut params.history,
            &mut params.commands,
            &mut params.project,
            edit,
            *current,
        );
    }

    if save_requested {
        save_editor_project(
            &mut params.project,
            &mut params.state,
            &entities,
            &params.transforms,
        );
    }

    Ok(())
}

#[derive(Clone, Copy)]
struct UiActions {
    create_entity: bool,
    delete_entity: Option<Entity>,
    duplicate_entity: Option<Entity>,
    save_requested: bool,
    transform_edit: Option<TransformEdit>,
}

impl UiActions {
    fn from(viewer: &DockViewer<'_>) -> Self {
        Self {
            create_entity: viewer.create_entity,
            delete_entity: viewer.delete_entity,
            duplicate_entity: viewer.duplicate_entity,
            save_requested: viewer.save_requested,
            transform_edit: viewer.transform_edit,
        }
    }
}

fn select_clicked_entity(event: On<Pointer<Click>>, mut selection: ResMut<SelectionState>) {
    selection.select(event.entity);
}

fn apply_transform_edit(
    history: &mut TransformHistory,
    commands: &mut Commands,
    project: &mut ProjectState,
    edit: TransformEdit,
    current: Transform,
) {
    let next = Transform {
        translation: edit.translation,
        rotation: Quat::from_euler(
            EulerRot::XYZ,
            edit.rotation.x.to_radians(),
            edit.rotation.y.to_radians(),
            edit.rotation.z.to_radians(),
        ),
        scale: edit.scale,
    };
    if current != next {
        history.push(TransformSnapshot {
            entity: edit.entity,
            transform: current,
        });
        commands.entity(edit.entity).insert(next);
        project.dirty = true;
    }
}

fn save_editor_project(
    project: &mut ProjectState,
    state: &mut EditorUiState,
    entities: &[(Entity, String)],
    transforms: &Query<&Transform, With<EditorEntity>>,
) {
    let items = entities.iter().filter_map(|(entity, name)| {
        transforms
            .get(*entity)
            .ok()
            .map(|transform| (name.clone(), *transform))
    });
    let document = SceneDocument::from_entities(items);
    let relative = project
        .main_scene
        .clone()
        .unwrap_or_else(|| PathBuf::from(".bevy-gui/untitled.scene.json"));
    let root = project.root.clone();
    let path = root.join(relative);

    match save_scene(&path, &document) {
        Ok(()) => {
            project.main_scene = path.strip_prefix(&root).ok().map(PathBuf::from);
            match save_project(&root, project) {
                Ok(()) => {
                    project.dirty = false;
                    state.status = format!("Saved {} entities", document.entities.len());
                }
                Err(error) => {
                    state.status = format!("Scene saved; project manifest failed: {error}");
                }
            }
        }
        Err(error) => state.status = format!("Save failed: {error}"),
    }
}

fn scan_assets(root: &Path, max_depth: usize, max_files: usize) -> Vec<String> {
    let mut output = Vec::new();
    visit_assets(root, root, 0, max_depth, max_files, &mut output);
    output.sort();
    output
}

fn visit_assets(
    root: &Path,
    current: &Path,
    depth: usize,
    max_depth: usize,
    max_files: usize,
    output: &mut Vec<String>,
) {
    if depth > max_depth || output.len() >= max_files {
        return;
    }
    let Ok(entries) = fs::read_dir(current) else {
        return;
    };

    for entry in entries.flatten() {
        if output.len() >= max_files {
            break;
        }
        let path = entry.path();
        if path
            .file_name()
            .is_some_and(|name| name == "target" || name == ".git" || name == ".bevy-gui")
        {
            continue;
        }
        if path.is_dir() {
            visit_assets(root, &path, depth + 1, max_depth, max_files, output);
        } else if path.is_file()
            && let Ok(relative) = path.strip_prefix(root)
        {
            output.push(relative.display().to_string());
        }
    }
}
