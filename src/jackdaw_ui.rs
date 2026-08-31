use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use bevy::text::{EditableText, TextFont};
use jackdaw_feathers::text_edit::{
    self, set_text_input_value, TextEditCommitEvent, TextEditDragging, TextEditProps,
    TextEditWrapper, TextEditValue,
};

use crate::selection::SelectionState;

/// Small integration layer that brings Jackdaw's Bevy-native Feathers widgets
/// into Bevy-GUI without replacing the existing egui editor shell.
///
/// The bridge deliberately starts with inspector-grade text inputs. This lets
/// Bevy-GUI adopt Jackdaw's widget/event model incrementally while preserving
/// the existing docking and egui panels.
pub struct JackdawUiPlugin;

impl Plugin for JackdawUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(jackdaw_feathers::EditorFeathersPlugin)
            .add_systems(Startup, spawn_jackdaw_bridge_inspector)
            .add_systems(Update, sync_jackdaw_bridge_selection)
            .add_observer(on_jackdaw_text_commit);
    }
}

#[derive(Component)]
struct JackdawBridgeInspector;

#[derive(Component, Clone, Copy)]
enum JackdawField {
    Name,
    PositionX,
}

#[derive(Component, Clone, Copy)]
struct JackdawFieldBinding {
    field: JackdawField,
}

fn spawn_jackdaw_bridge_inspector(mut commands: Commands) {
    let root = commands
        .spawn((
            JackdawBridgeInspector,
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(16.0),
                top: Val::Px(16.0),
                width: Val::Px(320.0),
                padding: UiRect::all(Val::Px(10.0)),
                row_gap: Val::Px(8.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.055, 0.06, 0.07, 0.96)),
            BorderRadius::all(Val::Px(10.0)),
        ))
        .id();

    let header = commands
        .spawn((
            Text::new("Jackdaw Inspector"),
            TextFont {
                font_size: 15.0,
                ..default()
            },
        ))
        .id();
    commands.entity(header).insert(ChildOf(root));

    let name = commands
        .spawn((
            text_edit::text_edit(
                TextEditProps::default()
                    .with_label("Name")
                    .with_placeholder("Select an entity"),
            ),
            JackdawFieldBinding {
                field: JackdawField::Name,
            },
        ))
        .id();
    commands.entity(name).insert(ChildOf(root));

    let position_x = commands
        .spawn((
            text_edit::text_edit(
                TextEditProps::default()
                    .with_label("Position X")
                    .with_placeholder("0.0")
                    .numeric_f32(),
            ),
            JackdawFieldBinding {
                field: JackdawField::PositionX,
            },
        ))
        .id();
    commands.entity(position_x).insert(ChildOf(root));
}

fn sync_jackdaw_bridge_selection(
    selection: Res<SelectionState>,
    mut fields: Query<(&JackdawFieldBinding, &Children, &TextEditValue)>,
    mut text_inputs: Query<&mut EditableText>,
    wrappers: Query<&TextEditWrapper>,
    names: Query<&Name>,
    transforms: Query<&Transform>,
) {
    if !selection.is_changed() {
        return;
    }

    let Some(entity) = selection.primary() else {
        return;
    };

    let name_value = names.get(entity).map(|n| n.as_str().to_owned()).unwrap_or_default();
    let position_x = transforms.get(entity).map(|t| t.translation.x).unwrap_or_default();

    for (binding, children, current) in &mut fields {
        let desired = match binding.field {
            JackdawField::Name => name_value.clone(),
            JackdawField::PositionX => position_x.to_string(),
        };
        if current.0 == desired {
            continue;
        }

        for child in children.iter() {
            let Some(wrapper) = wrappers.get(child).ok() else {
                continue;
            };
            if let Ok(mut editable) = text_inputs.get_mut(wrapper.0) {
                set_text_input_value(&mut editable, desired.clone());
                break;
            }
        }
    }
}

fn on_jackdaw_text_commit(
    event: On<TextEditCommitEvent>,
    bindings: Query<&JackdawFieldBinding>,
    parents: Query<&ChildOf>,
    mut names: Query<&mut Name>,
    mut transforms: Query<&mut Transform>,
    mut selection: ResMut<SelectionState>,
) {
    let mut current = event.entity;
    let mut binding = None;
    let mut source_ui = None;

    for _ in 0..4 {
        if let Ok(value) = bindings.get(current) {
            binding = Some(*value);
            source_ui = Some(current);
            break;
        }
        let Ok(parent) = parents.get(current) else {
            break;
        };
        current = parent.0;
    }

    let Some(binding) = binding else {
        return;
    };
    let Some(source_ui) = source_ui else {
        return;
    };

    // The binding identifies the editor field; its target is always the
    // currently focused entity. This mirrors the focused-entity model used by
    // Jackdaw's inspector while keeping mutations inside Bevy-GUI's selection
    // resource.
    let Some(target) = selection.primary() else {
        return;
    };

    match binding.field {
        JackdawField::Name => {
            if let Ok(mut name) = names.get_mut(target) {
                *name = Name::new(event.text.clone());
            } else {
                return;
            }
        }
        JackdawField::PositionX => {
            let Ok(value) = event.text.trim().parse::<f32>() else {
                return;
            };
            let Ok(mut transform) = transforms.get_mut(target) else {
                return;
            };
            transform.translation.x = value;
        }
    }

    // Keep the focused entity stable after the field mutation so the next UI
    // refresh updates the widget instead of dropping the inspector target.
    selection.focus(target);
    let _ = source_ui;
}

#[allow(dead_code)]
fn _keep_imports_used(_: &InputFocus, _: &TextEditDragging) {}
