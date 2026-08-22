use bevy::prelude::*;

use crate::{
    assets::AssetDatabase,
    command::{EditorCommandBus, EditorCommandId},
    export::{default_profile, export_project},
    project::{save_project, EditorMode, ProjectState},
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
            _ => {}
        }
    }
}
