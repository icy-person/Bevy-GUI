use bevy::prelude::*;

use crate::{
    assets::AssetDatabase,
    command::{EditorCommandBus, EditorCommandId},
    export::{default_profile, export_project},
    project::{save_project, EditorMode, ProjectState},
    selection::SelectionState,
    viewport::EditorEntity,
};

#[derive(Resource, Default, Debug)]
pub struct CommandExecutionState {
    pub executed: u64,
    pub last: Option<EditorCommandId>,
    pub last_error: Option<String>,
    pub last_message: Option<String>,
}

pub fn execute_editor_commands(
    mut bus: ResMut<EditorCommandBus>,
    mut project: ResMut<ProjectState>,
    mut assets: ResMut<AssetDatabase>,
    mut state: ResMut<CommandExecutionState>,
    mut selection: ResMut<SelectionState>,
    mut commands: Commands,
    transforms: Query<&Transform, With<EditorEntity>>,
) {
    for id in bus.drain() {
        state.executed = state.executed.saturating_add(1);
        state.last = Some(id);
        state.last_error = None;
        state.last_message = None;

        match id.0 {
            "project.save" => {
                if let Err(error) = save_project(&project.root, &project) {
                    state.last_error = Some(error.to_string());
                } else {
                    project.dirty = false;
                    state.last_message = Some("Project saved".into());
                }
            }
            "project.play" => project.mode = EditorMode::Play,
            "project.pause" => project.mode = EditorMode::Paused,
            "project.stop" => project.mode = EditorMode::Edit,
            "project.export" => {
                let profile = default_profile(&project);
                match export_project(&project, &profile) {
                    Ok(report) => {
                        state.last_message = Some(format!(
                            "Exported {} files ({} bytes) to {}",
                            report.files,
                            report.bytes,
                            report.output.display()
                        ));
                    }
                    Err(error) => state.last_error = Some(error.to_string()),
                }
            }
            "assets.refresh" => {
                assets.refresh_requested = true;
                state.last_message = Some("Asset scan requested".into());
            }
            "scene.new_entity" => {
                let entity = commands
                    .spawn((
                        Transform::default(),
                        Name::new("Entity"),
                        crate::viewport::EditorEntity,
                        crate::scene_model::EditorParent(None),
                        Pickable::default(),
                    ))
                    .id();
                selection.select(entity);
                project.dirty = true;
                state.last_message = Some("Entity created".into());
            }
            "scene.duplicate" => {
                if let Some(source) = selection.primary() {
                    match transforms.get(source) {
                        Ok(transform) => {
                            let entity = commands
                                .spawn((
                                    *transform,
                                    Name::new("Duplicate"),
                                    EditorEntity,
                                    crate::scene_model::EditorParent(None),
                                    Pickable::default(),
                                ))
                                .id();
                            selection.select(entity);
                            project.dirty = true;
                            state.last_message = Some("Entity duplicated".into());
                        }
                        Err(_) => {
                            state.last_message = Some("Selected entity is no longer available".into());
                        }
                    }
                } else {
                    state.last_message = Some("Select an entity first".into());
                }
            }
            "scene.delete" => {
                if let Some(entity) = selection.primary() {
                    commands.entity(entity).despawn();
                    selection.entities.retain(|current| *current != entity);
                    selection.focused = selection.entities.last().copied();
                    project.dirty = true;
                    state.last_message = Some("Entity deleted".into());
                } else {
                    state.last_message = Some("Select an entity first".into());
                }
            }
            "edit.undo" | "edit.redo" => {
                state.last_message = Some(if id.0 == "edit.undo" {
                    "Undo requested".into()
                } else {
                    "Redo requested".into()
                });
            }
            _ => {}
        }
    }
}
