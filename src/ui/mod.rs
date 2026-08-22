//! Editor workspace UI. Rendering, authoring actions, asset discovery and persistence
//! are isolated into focused submodules.

use bevy::ecs::system::{ParamSet, SystemParam};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::{
    assets::AssetDatabase,
    command::EditorCommandRegistry,
    docking::{show_dock_area, DockViewer, EditorDockState, TransformEdit},
    editor::{EditorPluginRegistry, EditorUiState},
    history::TransformHistory,
    project::ProjectState,
    scene_model::EditorParent,
    selection::SelectionState,
    viewport::EditorEntity,
};

mod actions;
mod assets;
mod persistence;

use actions::{apply_entity_actions, UiActions};
use persistence::save_editor_project;

#[derive(SystemParam)]
pub struct EditorUiParams<'w, 's> {
    pub contexts: EguiContexts<'w, 's>,
    pub dock: ResMut<'w, EditorDockState>,
    pub state: ResMut<'w, EditorUiState>,
    pub project: ResMut<'w, ProjectState>,
    pub selection: ResMut<'w, SelectionState>,
    pub registry: Res<'w, EditorCommandRegistry>,
    pub plugins: Res<'w, EditorPluginRegistry>,
    pub assets: Res<'w, AssetDatabase>,
    pub history: ResMut<'w, TransformHistory>,
    pub transforms: Query<'w, 's, &'static Transform, With<EditorEntity>>,
    pub names: Query<'w, 's, (Entity, Option<&'static Name>), With<EditorEntity>>,
    pub parent_queries: ParamSet<'w, 's, (
        Query<'w, 's, Option<&'static EditorParent>, With<EditorEntity>>,
        Query<'w, 's, &'static mut EditorParent, With<EditorEntity>>,
    )>,
    pub commands: Commands<'w, 's>,
}

pub fn install_editor_ui(app: &mut App) {
    app.add_systems(bevy_egui::EguiPrimaryContextPass, editor_ui_system);
}

fn editor_ui_system(mut mut_params: EditorUiParams) -> Result {
    let ctx = mut_params.contexts.ctx_mut()?;
    let entities: Vec<(Entity, String)> = mut_params
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

    let parent_map: Vec<(Entity, Option<Entity>)> = {
        let parents = mut_params.parent_queries.p0();
        entities
            .iter()
            .map(|(entity, _)| {
                (
                    *entity,
                    parents
                        .get(*entity)
                        .ok()
                        .flatten()
                        .and_then(|parent| parent.0),
                )
            })
            .collect()
    };

    let selected_transform = mut_params.selection.primary().and_then(|entity| {
        mut_params.transforms.get(entity).ok().map(|transform| {
            let (x, y, z) = transform.rotation.to_euler(EulerRot::XYZ);
            TransformEdit {
                entity,
                translation: transform.translation,
                rotation: Vec3::new(x.to_degrees(), y.to_degrees(), z.to_degrees()),
                scale: transform.scale,
            }
        })
    });

    let asset_paths: Vec<String> = mut_params
        .assets
        .entries
        .iter()
        .map(|entry| entry.path.display().to_string())
        .collect();
    let plugin_names: Vec<String> = mut_params
        .plugins
        .iter()
        .map(|(name, version)| format!("{name} v{version}"))
        .collect();

    let mut viewer = DockViewer {
        project: &mut mut_params.project,
        selection: &mut mut_params.selection,
        ui_state: &mut mut_params.state,
        entities: &entities,
        parents: &parent_map,
        selected_transform,
        assets: &asset_paths,
        plugin_names: &plugin_names,
        command_count: mut_params.registry.iter().count(),
        transform_edit: None,
        viewport_focused: false,
        create_entity: false,
        delete_entity: None,
        duplicate_entity: None,
        save_requested: false,
        parent_selected: false,
        unparent_selected: false,
    };

    let mut root_ui = egui::Ui::new(
        ctx.clone(),
        "editor_root".into(),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );

    egui::CentralPanel::default().show(&mut root_ui, |ui| {
        show_dock_area(ui, &mut mut_params.dock, &mut viewer);
    });

    let actions = UiActions::from(&viewer);
    let mut parents = mut_params.parent_queries.p1();
    apply_entity_actions(
        actions,
        &mut mut_params.commands,
        &mut mut_params.selection,
        &mut mut_params.project,
        &mut mut_params.history,
        &mut_params.transforms,
        &mut parents,
    );
    drop(parents);

    if actions.save_requested {
        let parents = mut_params.parent_queries.p0();
        let scene_entities: Vec<(Entity, String, Transform, Option<Entity>)> = entities
            .iter()
            .filter_map(|(entity, name)| {
                let transform = mut_params.transforms.get(*entity).ok().copied()?;
                let parent = parents
                    .get(*entity)
                    .ok()
                    .flatten()
                    .and_then(|parent| parent.0);
                Some((*entity, name.clone(), transform, parent))
            })
            .collect();
        save_editor_project(&mut mut_params.project, &mut mut_params.state, &scene_entities);
    }

    Ok(())
}
