//! Editor workspace UI. Rendering, authoring actions, asset discovery and persistence
//! are isolated into focused submodules.

use bevy::ecs::system::{ParamSet, SystemParam};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::{
    assets::AssetDatabase,
    command::{EditorCommandBus, EditorCommandRegistry},
    docking::{show_dock_area, DockViewer, EditorDockState, TransformEdit},
    editor::{EditorPluginRegistry, EditorUiState},
    history::TransformHistory,
    profiler::EditorProfiler,
    project::ProjectState,
    scene_model::EditorParent,
    selection::SelectionState,
    settings::EditorSettingsState,
    viewport::EditorEntity,
};

mod actions;
mod assets;
pub mod settings;
pub mod theme;
pub mod welcome;
pub mod workspace;
mod persistence;

use actions::{apply_entity_actions, UiActions};
use persistence::save_editor_project;
use welcome::{show_welcome, WelcomeState};
use workspace::{show_app_bar, show_navigation_rail};

#[derive(SystemParam)]
pub struct EditorUiParams<'w, 's> {
    pub contexts: EguiContexts<'w, 's>,
    pub dock: ResMut<'w, EditorDockState>,
    pub state: ResMut<'w, EditorUiState>,
    pub settings: ResMut<'w, EditorSettingsState>,
    pub welcome: ResMut<'w, WelcomeState>,
    pub project: ResMut<'w, ProjectState>,
    pub selection: ResMut<'w, SelectionState>,
    pub registry: Res<'w, EditorCommandRegistry>,
    pub command_bus: ResMut<'w, EditorCommandBus>,
    pub plugins: Res<'w, EditorPluginRegistry>,
    pub assets: ResMut<'w, AssetDatabase>,
    pub profiler: Res<'w, EditorProfiler>,
    pub history: ResMut<'w, TransformHistory>,
    pub transforms: Query<'w, 's, &'static Transform, With<EditorEntity>>,
    pub names: Query<'w, 's, (Entity, Option<&'static Name>), With<EditorEntity>>,
    pub visibility: Query<'w, 's, &'static Visibility, With<EditorEntity>>,
    pub parent_queries: ParamSet<'w, 's, (
        Query<'w, 's, Option<&'static EditorParent>, With<EditorEntity>>,
        Query<'w, 's, &'static mut EditorParent, With<EditorEntity>>,
    )>,
    pub commands: Commands<'w, 's>,
}

pub fn install_editor_ui(app: &mut App) {
    app.init_resource::<WelcomeState>()
        .add_systems(bevy_egui::EguiPrimaryContextPass, editor_ui_system);
}

fn editor_ui_system(mut mut_params: EditorUiParams) -> Result {
    let ctx = mut_params.contexts.ctx_mut()?;
    theme::apply_material_theme(ctx);
    theme::apply_material_settings(ctx, &mut_params.settings.settings);

    if mut_params.welcome.visible {
        let welcome = &mut *mut_params.welcome;
        let project = &mut *mut_params.project;
        let state = &mut *mut_params.state;
        let mut welcome_ui = egui::Ui::new(
            ctx.clone(),
            "welcome_root".into(),
            egui::UiBuilder::new()
                .layer_id(egui::LayerId::background())
                .max_rect(ctx.viewport_rect()),
        );
        show_welcome(&mut welcome_ui, welcome, project, state);
        return Ok(());
    }

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

    let selected_name = mut_params
        .selection
        .primary()
        .and_then(|entity| mut_params.names.get(entity).ok().and_then(|(_, name)| name))
        .map(|name| name.as_str().to_owned());

    let selected_visible = mut_params.selection.primary().and_then(|entity| {
        mut_params.visibility.get(entity).ok().map(|visibility| {
            !matches!(visibility, Visibility::Hidden)
        })
    });

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

    let plugin_names: Vec<String> = mut_params
        .plugins
        .iter()
        .map(|(name, version)| format!("{name} v{version}"))
        .collect();

    let project_name = mut_params.project.name.clone();
    let project_dirty = mut_params.project.dirty;
    let project_mode = mut_params.project.mode;
    let status = mut_params.state.status.clone();
    let mut rendered_actions: Option<UiActions> = None;

    let mut root_ui = egui::Ui::new(
        ctx.clone(),
        "editor_root".into(),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );

    egui::CentralPanel::default().show(&mut root_ui, |ui| {
        show_app_bar(
            ui,
            &project_name,
            project_dirty,
            project_mode,
            &status,
            &mut mut_params.welcome,
            &mut mut_params.command_bus,
        );
        ui.add_space(6.0);

        let mut viewer = DockViewer {
            project: &mut mut_params.project,
            selection: &mut mut_params.selection,
            ui_state: &mut mut_params.state,
            settings: &mut mut_params.settings,
            profiler: &mut mut_params.profiler,
            assets: &mut mut_params.assets,
            entities: &entities,
            parents: &parent_map,
            selected_transform,
            selected_name,
            selected_visible,
            plugin_names: &plugin_names,
            command_count: mut_params.registry.iter().count(),
            transform_edit: None,
            name_edit: None,
            visibility_edit: None,
            viewport_focused: false,
            create_entity: false,
            delete_entity: None,
            duplicate_entity: None,
            save_requested: false,
            parent_selected: false,
            unparent_selected: false,
        };

        ui.horizontal(|ui| {
            show_navigation_rail(ui, &mut mut_params.welcome);
            ui.separator();
            ui.allocate_ui(ui.available_size(), |content| {
                show_dock_area(content, &mut mut_params.dock, &mut viewer);
            });
        });

        rendered_actions = Some(UiActions::from(&viewer));
    });

    let actions = rendered_actions.unwrap_or(UiActions {
        create_entity: false,
        delete_entity: None,
        duplicate_entity: None,
        save_requested: false,
        transform_edit: None,
        name_edit: None,
        visibility_edit: None,
        parent_selected: false,
        unparent_selected: false,
    });
    let save_requested = actions.save_requested;

    let mut parents = mut_params.parent_queries.p1();
    apply_entity_actions(
        &actions,
        &mut mut_params.commands,
        &mut mut_params.selection,
        &mut mut_params.project,
        &mut mut_params.history,
        &mut mut_params.transforms,
        &mut parents,
    );
    drop(parents);

    if save_requested {
        let parents = mut_params.parent_queries.p0();
        let scene_entities: Vec<(Entity, String, Transform, Option<Entity>, bool)> = entities
            .iter()
            .filter_map(|(entity, name)| {
                let transform = mut_params.transforms.get(*entity).ok().copied()?;
                let parent = parents
                    .get(*entity)
                    .ok()
                    .flatten()
                    .and_then(|parent| parent.0);
                let visible = mut_params
                    .visibility
                    .get(*entity)
                    .ok()
                    .is_none_or(|visibility| !matches!(visibility, Visibility::Hidden));
                Some((*entity, name.clone(), transform, parent, visible))
            })
            .collect();
        save_editor_project(&mut mut_params.project, &mut mut_params.state, &scene_entities);
    }

    Ok(())
}
